// Copyright 2022 pluralizer Developers
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

use std::collections::HashMap;
use std::sync::Mutex;

use lazy_static::lazy_static;
use regex::Regex;

#[derive(Debug, Clone)]
struct WordRule {
    rule: Regex,
    placement: String,
}

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
                .collect(),
        );

        vec
    }};
}

macro_rules! load_irregular_map {
    ($rules: expr, $map: expr) => {
        $rules.iter().map($map).collect()
    };
}

lazy_static! {
    static ref IRREGULAR_SINGLES: Mutex<HashMap<String, String>> = Mutex::new(load_irregular_map!(
        constants::IRREGULAR_RULES,
        |(k, v)| (k.to_string(), v.to_string())
    ));
    static ref IRREGULAR_PLURALS: Mutex<HashMap<String, String>> = Mutex::new(load_irregular_map!(
        constants::IRREGULAR_RULES,
        |(k, v)| (v.to_string(), k.to_string())
    ));
    static ref PLURAL_RULES: Mutex<Vec<WordRule>> = Mutex::new(load_regex_vec!(
        constants::PLURAL_RULES,
        constants::UNCOUNTABLE_REGEX_RULES
    ));
    static ref SINGULAR_RULES: Mutex<Vec<WordRule>> = Mutex::new(load_regex_vec!(
        constants::SINGULAR_RULES,
        constants::UNCOUNTABLE_REGEX_RULES
    ));
    static ref UNCOUNTABLE_RULES: Mutex<Vec<String>> = Mutex::new(
        constants::UNCOUNTABLE_RULES
            .iter()
            .map(|s| s.to_string())
            .collect()
    );
}

/// Add an irregular word definition.
///
/// # Examples
/// ```
/// pluralizer::add_irregular_rule("I".to_string(), "we".to_string());
///
/// let result = pluralizer::pluralize("I", 2, false); // we
/// ```
pub fn add_irregular_rule(singular: String, plural: String) {
    IRREGULAR_SINGLES
        .lock()
        .unwrap()
        .insert(singular.to_string(), plural.to_string());
    IRREGULAR_PLURALS
        .lock()
        .unwrap()
        .insert(plural.to_string(), singular.to_string());
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
    PLURAL_RULES
        .lock()
        .unwrap()
        .push(WordRule { rule, placement });
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
    SINGULAR_RULES
        .lock()
        .unwrap()
        .push(WordRule { rule, placement });
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
        UncountableRule::Regex(rule) => {
            // We add it as both plural and singular rules with same placement
            add_plural_rule(rule.clone(), "$0".to_string());
            add_singular_rule(rule, "$0".to_string());
        }
        UncountableRule::String(rule) => {
            UNCOUNTABLE_RULES.lock().unwrap().push(rule.to_lowercase());
        }
    }
}

fn restore_case(word: &str, token: &str) -> String {
    // Tokens are an exact match.
    if word.eq(token) {
        return token.to_string();
    }

    // Lower cased words. E.g. "hello".
    if word.eq(&word.to_lowercase()) {
        return token.to_lowercase();
    }

    // Upper cased words. E.g. "WHISKY".
    if word.eq(&word.to_uppercase()) {
        return token.to_uppercase();
    }

    // Title cased words. E.g. "Title".
    let first_char = word.chars().nth(0);

    if let Some(fc) = first_char {
        if fc.is_uppercase() {
            let token_first_char = token.chars().nth(0);

            if let Some(tfc) = token_first_char {
                let last = if token.len() > 1 {
                    &token[1..token.len()]
                } else {
                    ""
                };

                return format!("{}{}", tfc, last);
            }
        }
    }

    // Lower cased words. E.g. "test".
    token.to_lowercase()
}

fn sanitize_word(token: String, word: &str, rules: Vec<WordRule>) -> String {
    let uncountable = get_mutex(&UNCOUNTABLE_RULES);

    // Empty string or doesn't need fixing.
    if token.len() == 0 || uncountable.contains(&token) {
        return word.to_string();
    }

    // Iterate over the sanitization rules and use the first one to match.
    for word_rule in rules.iter().rev() {
        if word_rule.rule.is_match(word) {
            let str = word_rule.rule.replace(word, |caps: &regex::Captures| {
                let mut str = restore_case(word, &word_rule.placement);

                for (i, m) in caps
                    .iter()
                    .filter(|m| m.is_some())
                    .map(|m| m.unwrap())
                    .enumerate()
                {
                    str = str.replace(
                        format!("${}", i).as_str(),
                        restore_case(word, m.as_str()).as_str(),
                    );
                }

                str
            });

            let mut skip = false;

            return str
                .chars()
                .filter(|c| {
                    if skip {
                        skip = false;

                        return skip;
                    }

                    skip = c == &'$';

                    !skip
                })
                .collect();
        }
    }

    word.to_string()
}

fn replace_word(
    replace_map: HashMap<String, String>,
    keep_map: HashMap<String, String>,
    rules: Vec<WordRule>,
    word: &str,
) -> String {
    // Get the correct token and case restoration functions.
    let token = word.to_lowercase();

    // Check against the keep object map.
    if keep_map.contains_key(&token) {
        return restore_case(word, &token);
    }

    // Check against the replacement map for a direct word replacement.
    if let Some(token) = replace_map.get(&*token) {
        return restore_case(word, token);
    }

    // Run all the rules against the word.
    sanitize_word(token, word, rules)
}

fn to_singular(word: &str) -> String {
    replace_word(
        get_mutex(&IRREGULAR_PLURALS),
        get_mutex(&IRREGULAR_SINGLES),
        get_mutex(&SINGULAR_RULES),
        word,
    )
}

fn get_mutex<T: Sized + Clone>(var: &Mutex<T>) -> T {
    match var.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
    .clone()
}

fn to_plural(word: &str) -> String {
    replace_word(
        get_mutex(&IRREGULAR_SINGLES),
        get_mutex(&IRREGULAR_PLURALS),
        get_mutex(&PLURAL_RULES),
        word,
    )
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
    let pluralized: String = if count == 1 {
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
