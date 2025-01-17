// Copyright 2025 pluralizer Developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

/*!
Rust package to pluralize or singularize any word based on a count inspired on pluralize NPM package.

It will keep plurals are plurals if the count given is not 1, either way, it is going to keep the  singular form if the count given is 1

# Example

```rust
use pluralizer::pluralize;

fn main() {
    // It can convert to plural
    println!("{}", pluralize("House", 2, true)); // 2 Houses

    // But also can convert to singular
    println!("{}", pluralize("Houses", 1, true)); // 1 House

    // And keep singularization if needed
    println!("{}", pluralize("House", 1, false)); // House

    // Or keep pluralization
    println!("{}", pluralize("Houses", 2, false)); // Houses
}
```

 */

pub(crate) mod constants;

#[cfg(test)]
mod test;

use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;
use std::sync::RwLock;

#[derive(Debug, Clone)]
struct WordRule {
    rule: Regex,
    placement: String,
}

// Macros to load data
macro_rules! load_regex_vec {
    ($rules: expr, $uncountable: expr) => {{
        let mut vec = $rules
            .iter()
            .map(|(k, v)| WordRule {
                rule: Regex::new(k).expect("Invalid regular expression"),
                placement: v.to_string(),
            })
            .collect::<Vec<WordRule>>();

        vec.append(
            &mut $uncountable
                .iter()
                .map(|s| WordRule {
                    rule: Regex::new(s).expect("Invalid regular expression"),
                    placement: "$0".to_string(),
                })
                .collect::<Vec<WordRule>>(),
        );

        vec
    }};
}

macro_rules! load_irregular_map {
    ($rules: expr, $map: expr) => {
        $rules.iter().map($map).collect()
    };
}

// Static references with RwLock
static IRREGULAR_SINGLES: Lazy<RwLock<HashMap<String, String>>> = Lazy::new(|| {
    RwLock::new(load_irregular_map!(constants::IRREGULAR_RULES, |(k, v)| (
        k.to_string(),
        v.to_string()
    )))
});

static IRREGULAR_PLURALS: Lazy<RwLock<HashMap<String, String>>> = Lazy::new(|| {
    RwLock::new(load_irregular_map!(constants::IRREGULAR_RULES, |(k, v)| (
        v.to_string(),
        k.to_string()
    )))
});

static PLURAL_RULES: Lazy<RwLock<Vec<WordRule>>> = Lazy::new(|| {
    RwLock::new(load_regex_vec!(
        constants::PLURAL_RULES,
        constants::UNCOUNTABLE_REGEX_RULES
    ))
});

static SINGULAR_RULES: Lazy<RwLock<Vec<WordRule>>> = Lazy::new(|| {
    RwLock::new(load_regex_vec!(
        constants::SINGULAR_RULES,
        constants::UNCOUNTABLE_REGEX_RULES
    ))
});

static UNCOUNTABLE_RULES: Lazy<RwLock<Vec<String>>> = Lazy::new(|| {
    RwLock::new(
        constants::UNCOUNTABLE_RULES
            .iter()
            .map(|s| s.to_string())
            .collect(),
    )
});

/// Add an irregular word definition.
///
/// # Examples
/// ```
/// pluralizer::add_irregular_rule("I".to_string(), "we".to_string());
///
/// let result = pluralizer::pluralize("I", 2, false); // we
/// ```
pub fn add_irregular_rule(singular: String, plural: String) {
    {
        let mut singles = IRREGULAR_SINGLES.write().unwrap();
        singles.insert(singular.clone(), plural.clone());
    }
    {
        let mut plurals = IRREGULAR_PLURALS.write().unwrap();
        plurals.insert(plural, singular);
    }
}

/// Add a pluralization rule to the collection.
///
/// The rule argument must be a regular expression string.
///
/// # Examples
/// ```
/// use regex::Regex;
///
/// pluralizer::add_plural_rule(Regex::new("(?i)(matr|cod|mur|sil|vert|ind|append)(?:ix|ex)$").unwrap(), "$1ices".to_string());
///
/// let result = pluralizer::pluralize("Vertex", 2, false); // Vertices
/// ```
pub fn add_plural_rule(rule: Regex, placement: String) {
    let mut plural_rules = PLURAL_RULES.write().unwrap();
    plural_rules.push(WordRule { rule, placement });
}

/// Add a singularization rule to the collection.
///
/// The rule argument must be a regular expression string.
///
/// # Examples
/// ```
/// use regex::Regex;
///
/// pluralizer::add_singular_rule(Regex::new("(?i)(matr|append)ices$").unwrap(), "$1ix".to_string());
///
/// let result = pluralizer::pluralize("Matrices", 1, false); // Matrix
/// ```
pub fn add_singular_rule(rule: Regex, placement: String) {
    let mut singular_rules = SINGULAR_RULES.write().unwrap();
    singular_rules.push(WordRule { rule, placement });
}

/// Uncountable rule struct
///
/// It's given as a parameter of [add_uncountable_rule](add_uncountable_rule) method
pub enum UncountableRule {
    Regex(Regex),
    String(String),
}

/// Add an uncountable word rule.
///
/// The rule can be either a word or a RegEx using [the rule struct](UncountableRule)
///
/// # Examples
/// ```
/// use pluralizer::UncountableRule;
///
/// pluralizer::add_uncountable_rule(UncountableRule::String("cash".to_string()));
///
/// let result = pluralizer::pluralize("Cash", 2, false); // Cash
/// ```
pub fn add_uncountable_rule(rule: UncountableRule) {
    match rule {
        UncountableRule::Regex(regex_rule) => {
            // We add it as both plural and singular rules with the same placement
            add_plural_rule(regex_rule.clone(), "$0".to_string());
            add_singular_rule(regex_rule, "$0".to_string());
        }
        UncountableRule::String(word) => {
            let mut uncountable = UNCOUNTABLE_RULES.write().unwrap();
            uncountable.push(word.to_lowercase());
        }
    }
}

/// Pluralize or singularize a word based on the passed in count.
///
/// # Examples
/// ```
/// pluralizer::pluralize("House", 2, true); // 2 Houses
/// pluralizer::pluralize("Houses", 1, true); // 1 House
/// pluralizer::pluralize("House", 1, false); // House
/// pluralizer::pluralize("Houses", 2, false); // Houses
/// ```
pub fn pluralize(word: &str, count: isize, include_count: bool) -> String {
    let pluralized = if count == 1 {
        to_singular(word)
    } else {
        to_plural(word)
    };
    if include_count {
        format!("{} {}", count, pluralized)
    } else {
        pluralized
    }
}

fn to_singular(word: &str) -> String {
    let irregular_plurals = IRREGULAR_PLURALS.read().unwrap();
    let irregular_singles = IRREGULAR_SINGLES.read().unwrap();
    let singular_rules = SINGULAR_RULES.read().unwrap();
    let uncountable = UNCOUNTABLE_RULES.read().unwrap();

    replace_word(
        &irregular_plurals,
        &irregular_singles,
        &singular_rules,
        &uncountable,
        word,
    )
}

fn to_plural(word: &str) -> String {
    let irregular_singles = IRREGULAR_SINGLES.read().unwrap();
    let irregular_plurals = IRREGULAR_PLURALS.read().unwrap();
    let plural_rules = PLURAL_RULES.read().unwrap();
    let uncountable = UNCOUNTABLE_RULES.read().unwrap();

    replace_word(
        &irregular_singles,
        &irregular_plurals,
        &plural_rules,
        &uncountable,
        word,
    )
}

// This function tries to replace the given word by looking at:
// 1. The "replace_map" (e.g., known irregular singular->plural or vice versa)
// 2. The "keep_map" (the inverse map, e.g., known irregular plural->singular)
// 3. The set of regex-based rules
// 4. The list of uncountable words
fn replace_word(
    replace_map: &HashMap<String, String>,
    keep_map: &HashMap<String, String>,
    rules: &[WordRule],
    uncountable: &[String],
    word: &str,
) -> String {
    let token = word.to_lowercase();

    // Check against the keep map
    if keep_map.contains_key(&token) {
        return restore_case(word, &token);
    }

    // Check if there's a direct replacement in the replace_map
    if let Some(replacement) = replace_map.get(&token) {
        return restore_case(word, replacement);
    }

    // Finally, check rules or see if it's uncountable
    sanitize_word(&token, word, rules, uncountable)
}

// This performs the main logic for applying regex-based transformations,
// taking into account "uncountable" words
fn sanitize_word(token: &str, word: &str, rules: &[WordRule], uncountable: &[String]) -> String {
    // If empty or uncountable, return as-is
    if token.is_empty() || uncountable.contains(&token.to_owned()) {
        return word.to_string();
    }

    // Iterate rules from last to first
    for word_rule in rules.iter().rev() {
        if word_rule.rule.is_match(word) {
            let replaced = word_rule.rule.replace(word, |caps: &regex::Captures| {
                let mut out = restore_case(word, &word_rule.placement);
                for (i, m) in caps.iter().filter_map(|m| m).enumerate() {
                    out = out.replace(&format!("${}", i), &restore_case(word, m.as_str()));
                }
                out
            });
            // Remove leftover `$` if any
            return remove_dollar_escapes(&replaced);
        }
    }

    word.to_string()
}

// If the replaced string has `$` leftover, remove them
fn remove_dollar_escapes(s: &str) -> String {
    let mut skip = false;
    let mut result = String::new();

    for c in s.chars() {
        if skip {
            skip = false;
            continue;
        }
        if c == '$' {
            skip = true;
            continue;
        }
        result.push(c);
    }

    result
}

// Restores the case from `word` into `token`
fn restore_case(word: &str, token: &str) -> String {
    if word == token {
        return token.to_string();
    }
    if word == word.to_lowercase() {
        return token.to_lowercase();
    }
    if word == word.to_uppercase() {
        return token.to_uppercase();
    }
    if let Some(first) = word.chars().next() {
        if first.is_uppercase() {
            if let Some(token_first) = token.chars().next() {
                let remainder = if token.len() > 1 { &token[1..] } else { "" };
                return format!("{}{}", token_first, remainder);
            }
        }
    }
    token.to_lowercase()
}

