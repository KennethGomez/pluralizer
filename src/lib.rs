// Copyright 2025 pluralizer Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

/*!
Rust package to pluralize or singularize any word based on a count, inspired by the
[pluralize](https://www.npmjs.com/package/pluralize) NPM package.

A count of exactly `1` produces the singular form; any other count (including `0` and
negative counts) produces the plural form. Words that are already in the requested form are
kept as they are.

# Example

```rust
use pluralizer::pluralize;

// It can convert to plural
assert_eq!(pluralize("House", 2, true), "2 Houses");

// But also can convert to singular
assert_eq!(pluralize("Houses", 1, true), "1 House");

// And keep singularization if needed
assert_eq!(pluralize("House", 1, false), "House");

// Or keep pluralization
assert_eq!(pluralize("Houses", 2, false), "Houses");
```

# Custom rules

The rule tables can be extended at runtime with [`add_irregular_rule`], [`add_plural_rule`],
[`add_singular_rule`] and [`add_uncountable_rule`]. The tables are global: a rule added from
any thread applies to every later call, on every thread.

Irregular words and uncountable words (built-in, or added with [`UncountableRule::String`])
are checked first. After that, regex rules are tried from the most recently added one to the
oldest, so a custom regex rule takes precedence over the built-in ones.

# Differences from the NPM package

* The built-in rules mirror the `plurals/pluralize` GitHub `master` branch at commit `36f03cd`,
  which is newer than the last NPM release (`pluralize@8.0.0`). For example `my`/`our` and
  `canvas`/`canvases` are irregular, and `species` is not uncountable.
* Text that a rule captures from the word keeps its case, and the rest of the replacement takes
  its case from the whole word rather than from the matched text, so results for mixed-case
  words can differ: `"fooURL"` becomes `"fooURLs"` (NPM: `"fooURLS"`) and `"the Lives"` becomes
  `"the Life"` (NPM: `"the life"`).
* Rules use the `regex` crate's Unicode-aware `\b` and `\w`.
*/

pub(crate) mod constants;

#[cfg(test)]
mod test;

#[cfg(doctest)]
#[doc = include_str!("../README.md")]
struct ReadmeDoctests;

use once_cell::sync::Lazy;
use regex::{Captures, Regex, RegexSet};
use std::collections::{HashMap, HashSet};
use std::ops::Range;
use std::sync::{PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard};

#[derive(Debug, Clone)]
struct WordRule {
    rule: Regex,
    placement: String,
    // The most frequently applied rules (`s?$` -> "s", `s$` -> "") have no `$` in their
    // placement, so they can use `find()` instead of the slower `captures()`.
    has_dollar: bool,
    // The placement is all lowercase and does not start with `$`, so in a Title-case word it
    // takes the capital letter the matched text started with (`Man` -> `Men`).
    takes_capital: bool,
}

impl WordRule {
    fn new(rule: Regex, placement: String) -> Self {
        let has_dollar = placement.contains('$');
        let takes_capital = !placement.starts_with('$') && placement == placement.to_lowercase();
        WordRule {
            rule,
            placement,
            has_dollar,
            takes_capital,
        }
    }
}

// Words longer than this skip the `RegexSet`. The set makes one forward pass over the whole
// word, while the linear scan runs each `$`-anchored rule from the end of it, at a fixed cost
// per rule. On real text the set pass is cheaper up to roughly 130-150 bytes.
const REGEX_SET_MAX_LEN: usize = 128;

// The built-in regex rules never change, so they live outside the lock. They are matched with
// a single `RegexSet` pass instead of trying every rule one by one.
struct BuiltinRules {
    rules: Vec<WordRule>,
    set: RegexSet,
    // Index into `rules` of each pattern in `set`.
    set_rules: Vec<usize>,
    // Rules using Unicode `\b`, in descending order. They are matched individually because the
    // `RegexSet` would otherwise fall back to a much slower engine on non-ASCII words.
    boundary_rules: Vec<usize>,
}

impl BuiltinRules {
    fn new(rules: &[(&str, &str)]) -> Self {
        let patterns: Vec<(&str, &str)> = rules
            .iter()
            .copied()
            .chain(
                constants::UNCOUNTABLE_REGEX_RULES
                    .iter()
                    .map(|&rule| (rule, "$0")),
            )
            .collect();

        let (boundary_rules, set_rules): (Vec<usize>, Vec<usize>) =
            (0..patterns.len()).partition(|&i| patterns[i].0.contains(r"\b"));

        BuiltinRules {
            rules: patterns
                .iter()
                .map(|&(rule, placement)| {
                    WordRule::new(
                        Regex::new(rule).expect("Invalid regular expression"),
                        placement.to_string(),
                    )
                })
                .collect(),
            set: RegexSet::new(set_rules.iter().map(|&i| patterns[i].0))
                .expect("Invalid regular expression"),
            set_rules,
            boundary_rules: boundary_rules.into_iter().rev().collect(),
        }
    }

    // Returns the last rule that matches `word`.
    fn find(&self, word: &str) -> Option<&WordRule> {
        if word.len() > REGEX_SET_MAX_LEN {
            return self
                .rules
                .iter()
                .rev()
                .find(|rule| rule.rule.is_match(word));
        }

        let last_in_set = self
            .set
            .matches(word)
            .iter()
            .next_back()
            .map(|i| self.set_rules[i]);
        let last_boundary = self
            .boundary_rules
            .iter()
            .copied()
            .take_while(|&i| last_in_set.map_or(true, |last| i > last))
            .find(|&i| self.rules[i].rule.is_match(word));

        last_boundary.or(last_in_set).map(|i| &self.rules[i])
    }
}

static BUILTIN_PLURAL_RULES: Lazy<BuiltinRules> =
    Lazy::new(|| BuiltinRules::new(constants::PLURAL_RULES));

static BUILTIN_SINGULAR_RULES: Lazy<BuiltinRules> =
    Lazy::new(|| BuiltinRules::new(constants::SINGULAR_RULES));

// Every mutable table lives behind a single lock, so each call takes it once and every
// `add_*` function updates its tables atomically.
struct RuleTables {
    // Keyed by the lowercase word, which is what lookups use
    irregular_singles: HashMap<String, String>,
    irregular_plurals: HashMap<String, String>,
    // Regex rules added at runtime; the built-in ones are in `BUILTIN_*_RULES`.
    plural_rules: Vec<WordRule>,
    singular_rules: Vec<WordRule>,
    uncountable: HashSet<String>,
}

static RULES: Lazy<RwLock<RuleTables>> = Lazy::new(|| {
    RwLock::new(RuleTables {
        irregular_singles: constants::IRREGULAR_RULES
            .iter()
            .map(|&(singular, plural)| (singular.to_lowercase(), plural.to_string()))
            .collect(),
        irregular_plurals: constants::IRREGULAR_RULES
            .iter()
            .map(|&(singular, plural)| (plural.to_lowercase(), singular.to_string()))
            .collect(),
        plural_rules: Vec::new(),
        singular_rules: Vec::new(),
        uncountable: constants::UNCOUNTABLE_RULES
            .iter()
            .map(|&word| word.to_string())
            .collect(),
    })
});

// The write sections only insert into or push onto the tables and do not panic, so a poisoned
// lock still holds valid tables and can be used as is. Keep anything that may panic out of them.
fn read_rules() -> RwLockReadGuard<'static, RuleTables> {
    RULES.read().unwrap_or_else(PoisonError::into_inner)
}

fn write_rules() -> RwLockWriteGuard<'static, RuleTables> {
    RULES.write().unwrap_or_else(PoisonError::into_inner)
}

/// Add an irregular word definition.
///
/// Irregular words are checked before any regex rule, and are matched case-insensitively. The
/// replacement is lowercased for lowercase words and uppercased for UPPERCASE words. For other
/// words, a replacement written with capitals of its own (such as `McDonalds`) is kept as
/// written; otherwise it takes the word's capitals on the letters the two start with
/// (`"Octopus"` becomes `"Octopodes"`). A word that is already in the requested form is returned
/// as it is.
///
/// # Examples
/// ```
/// use pluralizer::{add_irregular_rule, pluralize};
///
/// assert_eq!(pluralize("irregular", 2, false), "irregulars");
///
/// add_irregular_rule("irregular".to_string(), "regular".to_string());
///
/// assert_eq!(pluralize("irregular", 2, false), "regular");
/// assert_eq!(pluralize("Regular", 1, false), "Irregular");
/// ```
pub fn add_irregular_rule(singular: String, plural: String) {
    let singular_key = singular.to_lowercase();
    let plural_key = plural.to_lowercase();

    let mut rules = write_rules();
    rules.irregular_singles.insert(singular_key, plural);
    rules.irregular_plurals.insert(plural_key, singular);
}

/// Add a pluralization rule to the collection.
///
/// The `rule` is matched against the word as given, so add `(?i)` to make it case-insensitive.
/// Rules added later take precedence over earlier and built-in rules.
///
/// The first match is replaced with `placement`, where `$N` is replaced with capture group `N`
/// as it appears in the word (`$0` is the whole match), or with nothing if that group did not
/// take part in the match. Two digits are read as one group number only if the rule has that
/// many groups, so with three groups `$12` is group 1 followed by `2`. Any other `$` is kept as
/// is. This is not the `regex` crate's replacement syntax: write `$1ices`, not `${1}ices`.
///
/// The rest of the placement is re-cased to match the word: it is uppercased for UPPERCASE words
/// and when it replaces an UPPERCASE run of two or more letters (`myPERSON` becomes `myPEOPLE`),
/// kept as written for other words that start with a capital letter (`Man`, `SuperMan`), and
/// lowercased for all other words (`man`, `myMan`). Then, if the replaced text starts with a
/// capital letter, so does the replacement, unless the placement starts with `$` or is written
/// with capitals in a word that starts with a capital letter (`Man` becomes `Men`).
///
/// # Examples
/// ```
/// use pluralizer::{add_plural_rule, pluralize};
/// use regex::Regex;
///
/// assert_eq!(pluralize("regex", 2, false), "regexes");
///
/// add_plural_rule(Regex::new("(?i)(ge)x$").unwrap(), "$1xii".to_string());
///
/// assert_eq!(pluralize("Regex", 2, false), "Regexii");
/// ```
pub fn add_plural_rule(rule: Regex, placement: String) {
    let rule = WordRule::new(rule, placement);
    write_rules().plural_rules.push(rule);
}

/// Add a singularization rule to the collection.
///
/// The `rule` is matched against the word as given, so add `(?i)` to make it case-insensitive.
/// Rules added later take precedence over earlier and built-in rules.
///
/// The first match is replaced with `placement`, where `$N` is replaced with capture group `N`
/// as it appears in the word (`$0` is the whole match), or with nothing if that group did not
/// take part in the match. Two digits are read as one group number only if the rule has that
/// many groups, so with three groups `$12` is group 1 followed by `2`. Any other `$` is kept as
/// is. This is not the `regex` crate's replacement syntax: write `$1ices`, not `${1}ices`.
///
/// The rest of the placement is re-cased to match the word: it is uppercased for UPPERCASE words
/// and when it replaces an UPPERCASE run of two or more letters (`myPERSON` becomes `myPEOPLE`),
/// kept as written for other words that start with a capital letter (`Man`, `SuperMan`), and
/// lowercased for all other words (`man`, `myMan`). Then, if the replaced text starts with a
/// capital letter, so does the replacement, unless the placement starts with `$` or is written
/// with capitals in a word that starts with a capital letter (`Man` becomes `Men`).
///
/// # Examples
/// ```
/// use pluralizer::{add_singular_rule, pluralize};
/// use regex::Regex;
///
/// assert_eq!(pluralize("singles", 1, false), "single");
///
/// add_singular_rule(Regex::new("(?i)singles$").unwrap(), "singular".to_string());
///
/// assert_eq!(pluralize("singles", 1, false), "singular");
/// ```
pub fn add_singular_rule(rule: Regex, placement: String) {
    let rule = WordRule::new(rule, placement);
    write_rules().singular_rules.push(rule);
}

/// An uncountable word rule, passed to [`add_uncountable_rule`].
///
/// Irregular words are checked first, so neither variant has any effect on a word that has an
/// irregular form (for example `goose`/`geese`).
#[derive(Debug, Clone)]
pub enum UncountableRule {
    /// The text this regex matches is kept as it is. It is matched against the word as given,
    /// so add `(?i)` to make it case-insensitive. Regex rules added later take precedence.
    Regex(Regex),
    /// This word, compared case-insensitively, is left unchanged.
    String(String),
}

/// Add an uncountable word rule.
///
/// The rule can be either a word or a regex, see [`UncountableRule`].
///
/// # Examples
/// ```
/// use pluralizer::{add_uncountable_rule, pluralize, UncountableRule};
/// use regex::Regex;
///
/// assert_eq!(pluralize("paper", 2, false), "papers");
/// assert_eq!(pluralize("widgetz", 2, false), "widgetzs");
///
/// add_uncountable_rule(UncountableRule::String("paper".to_string()));
/// add_uncountable_rule(UncountableRule::Regex(Regex::new("(?i)widgetz$").unwrap()));
///
/// assert_eq!(pluralize("Paper", 2, false), "Paper");
/// assert_eq!(pluralize("widgetz", 2, false), "widgetz");
/// ```
pub fn add_uncountable_rule(rule: UncountableRule) {
    match rule {
        UncountableRule::Regex(regex_rule) => {
            // We add it as both plural and singular rules with the same placement
            let plural_rule = WordRule::new(regex_rule.clone(), "$0".to_string());
            let singular_rule = WordRule::new(regex_rule, "$0".to_string());

            let mut rules = write_rules();
            rules.plural_rules.push(plural_rule);
            rules.singular_rules.push(singular_rule);
        }
        UncountableRule::String(word) => {
            let word = word.to_lowercase();
            write_rules().uncountable.insert(word);
        }
    }
}

/// Pluralize or singularize a word based on the passed in count.
///
/// A `count` of exactly `1` produces the singular form; any other count produces the plural
/// form. When `include_count` is `true`, the count is prepended to the result.
///
/// # Examples
/// ```
/// use pluralizer::pluralize;
///
/// assert_eq!(pluralize("House", 2, true), "2 Houses");
/// assert_eq!(pluralize("Houses", 1, true), "1 House");
/// assert_eq!(pluralize("House", 1, false), "House");
/// assert_eq!(pluralize("Houses", 2, false), "Houses");
/// assert_eq!(pluralize("House", 0, true), "0 Houses");
/// ```
pub fn pluralize(word: &str, count: isize, include_count: bool) -> String {
    let pluralized = if count == 1 {
        to_singular(word)
    } else {
        to_plural(word)
    };
    if include_count {
        format!("{count} {pluralized}")
    } else {
        pluralized
    }
}

fn to_singular(word: &str) -> String {
    // Forced before taking the lock, so compiling the built-in rules never blocks `add_*` calls
    let builtin_rules = &*BUILTIN_SINGULAR_RULES;
    let rules = read_rules();

    replace_word(
        &rules.irregular_plurals,
        &rules.irregular_singles,
        builtin_rules,
        &rules.singular_rules,
        &rules.uncountable,
        word,
    )
}

fn to_plural(word: &str) -> String {
    // Forced before taking the lock, so compiling the built-in rules never blocks `add_*` calls
    let builtin_rules = &*BUILTIN_PLURAL_RULES;
    let rules = read_rules();

    replace_word(
        &rules.irregular_singles,
        &rules.irregular_plurals,
        builtin_rules,
        &rules.plural_rules,
        &rules.uncountable,
        word,
    )
}

// This function tries to replace the given word by looking at:
// 1. The "keep_map" (words that are already in the requested form)
// 2. The "replace_map" (e.g., known irregular singular->plural or vice versa)
// 3. The list of uncountable words
// 4. The custom and built-in regex rules
fn replace_word(
    replace_map: &HashMap<String, String>,
    keep_map: &HashMap<String, String>,
    builtin_rules: &BuiltinRules,
    custom_rules: &[WordRule],
    uncountable: &HashSet<String>,
    word: &str,
) -> String {
    let token = word.to_lowercase();
    let case = Case::of(word);

    // Check against the keep map: the word is already in the requested form
    if keep_map.contains_key(&token) {
        return word.to_string();
    }

    // Check if there's a direct replacement in the replace_map
    if let Some(replacement) = replace_map.get(&token) {
        return restore_word_case(word, case, replacement);
    }

    // Finally, check rules or see if it's uncountable
    sanitize_word(&token, word, case, builtin_rules, custom_rules, uncountable)
}

// This performs the main logic for applying regex-based transformations,
// taking into account "uncountable" words
fn sanitize_word(
    token: &str,
    word: &str,
    case: Case,
    builtin_rules: &BuiltinRules,
    custom_rules: &[WordRule],
    uncountable: &HashSet<String>,
) -> String {
    // If empty or uncountable, return as-is
    if token.is_empty() || uncountable.contains(token) {
        return word.to_string();
    }

    // Custom rules come after the built-in ones, so they are tried first, newest first
    let rule = custom_rules
        .iter()
        .rev()
        .find(|rule| rule.rule.is_match(word))
        .or_else(|| builtin_rules.find(word));

    match rule {
        Some(rule) => apply_rule(rule, word, case),
        None => word.to_string(),
    }
}

// Replaces the first match of `rule` in `word` with its case-restored placement
fn apply_rule(rule: &WordRule, word: &str, case: Case) -> String {
    let captures = if rule.has_dollar {
        Some(rule.rule.captures(word).expect("rule matched the word"))
    } else {
        None
    };
    let range = match &captures {
        Some(captures) => captures.get(0).expect("group 0 is the whole match").range(),
        None => rule.rule.find(word).expect("rule matched the word").range(),
    };
    let matched = &word[range.clone()];

    // Text that replaces an UPPERCASE run inside a mixed-case word stays uppercase (`myPEOPLE`)
    let case = if matches!(case, Case::Title | Case::Other) && is_uppercase_run(matched) {
        Case::Upper
    } else {
        case
    };
    let placement = restore_case(word, case, &rule.placement);
    let mut replacement = match &captures {
        Some(captures) => interpolate(&placement, captures),
        None => placement,
    };

    // Keep the capital letter the replaced text started with
    let takes_capital = match case {
        Case::Title => rule.takes_capital,
        // The placement has just been lowercased, however it was written
        Case::Other => !rule.placement.starts_with('$'),
        Case::Lower | Case::Upper => false,
    };
    if takes_capital && starts_with_uppercase(matched) {
        replacement = capitalize(&replacement);
    }

    splice(word, range, &replacement)
}

fn splice(word: &str, range: Range<usize>, replacement: &str) -> String {
    let mut result = String::with_capacity(word.len() - range.len() + replacement.len());
    result.push_str(&word[..range.start]);
    result.push_str(replacement);
    result.push_str(&word[range.end..]);
    result
}

// Replaces `$N` and `$NN` in `template` with the matching capture group as it appears in the
// word, or with nothing if the group did not participate in the match. `$NN` is only read as a
// two-digit group when it does not start with `0` and the rule has that many groups, so `$01` is
// `$0` followed by `1`. Any other `$` is kept as is, and inserted text is never rescanned.
fn interpolate(template: &str, captures: &Captures<'_>) -> String {
    let bytes = template.as_bytes();
    let mut result = String::with_capacity(template.len() + 16);
    let mut literal_start = 0;
    let mut i = 0;

    while i < bytes.len() {
        let first_digit = match bytes.get(i + 1) {
            Some(digit) if bytes[i] == b'$' && digit.is_ascii_digit() => usize::from(digit - b'0'),
            _ => {
                i += 1;
                continue;
            }
        };

        let (group, end) = match bytes.get(i + 2) {
            Some(digit)
                if first_digit != 0
                    && digit.is_ascii_digit()
                    && first_digit * 10 + usize::from(digit - b'0') < captures.len() =>
            {
                (first_digit * 10 + usize::from(digit - b'0'), i + 3)
            }
            _ => (first_digit, i + 2),
        };

        result.push_str(&template[literal_start..i]);
        if let Some(found) = captures.get(group) {
            result.push_str(found.as_str());
        }
        i = end;
        literal_start = end;
    }

    result.push_str(&template[literal_start..]);
    result
}

// The casing of the input word, which is applied to replacement text
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Case {
    Lower,
    Upper,
    Title,
    Other,
}

impl Case {
    fn of(word: &str) -> Case {
        if word.is_ascii() {
            let bytes = word.as_bytes();
            if !bytes.iter().any(u8::is_ascii_uppercase) {
                Case::Lower
            } else if !bytes.iter().any(u8::is_ascii_lowercase) {
                Case::Upper
            } else if bytes[0].is_ascii_uppercase() {
                Case::Title
            } else {
                Case::Other
            }
        } else if word == word.to_lowercase() {
            Case::Lower
        } else if word == word.to_uppercase() {
            Case::Upper
        } else if starts_with_uppercase(word) {
            Case::Title
        } else {
            Case::Other
        }
    }
}

// Restores the case from `word` into `token`
fn restore_case(word: &str, case: Case, token: &str) -> String {
    if word == token {
        return token.to_string();
    }
    match case {
        Case::Lower | Case::Other => token.to_lowercase(),
        Case::Upper => token.to_uppercase(),
        Case::Title => token.to_string(),
    }
}

// Restores the case from `word` into the irregular word that replaces it. Lowercase and
// UPPERCASE words work like `restore_case`. For other words, a replacement with capitals of its
// own is kept as written (`iPhones`), and an all-lowercase one takes the word's capitals on the
// start the two share (`McDonalds` -> `McDonald`), or else the word's leading capital
// (`Is` -> `Are`).
fn restore_word_case(word: &str, case: Case, token: &str) -> String {
    if matches!(case, Case::Lower | Case::Upper) {
        return restore_case(word, case, token);
    }
    if token != token.to_lowercase() {
        return token.to_string();
    }

    let mut word_end = 0;
    let mut token_end = 0;
    for (word_char, token_char) in word.chars().zip(token.chars()) {
        if !word_char.to_lowercase().eq(std::iter::once(token_char)) {
            break;
        }
        word_end += word_char.len_utf8();
        token_end += token_char.len_utf8();
    }

    if word_end > 0 {
        format!("{}{}", &word[..word_end], &token[token_end..])
    } else if case == Case::Title {
        capitalize(token)
    } else {
        token.to_string()
    }
}

fn starts_with_uppercase(text: &str) -> bool {
    text.chars().next().map_or(false, char::is_uppercase)
}

// At least two uppercase characters and no lowercase ones
fn is_uppercase_run(text: &str) -> bool {
    !text.chars().any(char::is_lowercase)
        && text.chars().filter(|c| c.is_uppercase()).nth(1).is_some()
}

// Uppercases the first character if it is lowercase
fn capitalize(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) if first.is_lowercase() => first.to_uppercase().chain(chars).collect(),
        _ => text.to_string(),
    }
}
