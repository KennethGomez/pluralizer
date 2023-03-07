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
    pluralizer::initialize();

    // It can convert to plural
    println!("{}", pluralizer::pluralize("House", 2, true)); // 2 Houses

    // But also can convert to singular
    println!("{}", pluralizer::pluralize("Houses", 1, true)); // 1 House

    // And keep singularization if needed
    println!("{}", pluralizer::pluralize("House", 1, false)); // House

    // Or keep pluralization
    println!("{}", pluralizer::pluralize("Houses", 2, false)); // Houses
}
```

 */

pub(crate) mod constants;

#[cfg(test)]
mod test;

use std::collections::HashMap;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use lazy_static::lazy_static;
use regex::{Regex};

#[derive(Debug, Clone)]
struct WordRule {
    rule: Regex,
    placement: String,
}

macro_rules! load_regex_vec {
    ($rules: expr) => {
        $rules
            .iter()
            .map(|(k, v)| WordRule {
                rule: Regex::new(k).expect("Invalid regular expression"),
                placement: v.to_string()
            })
            .collect()
    }
}

lazy_static! {
    static ref IRREGULAR_SINGLES: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
    static ref IRREGULAR_PLURALS: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
    static ref PLURAL_RULES: Mutex<Vec<WordRule>> = Mutex::new(load_regex_vec!(constants::PLURAL_RULES));
    static ref SINGULAR_RULES: Mutex<Vec<WordRule>> = Mutex::new(load_regex_vec!(constants::SINGULAR_RULES));
    static ref UNCOUNTABLE_RULES: Mutex<Vec<String>> = Mutex::new(Vec::new());
    static ref INITIALIZED: AtomicBool = AtomicBool::new(false);
}

/// This function initializes all constants into static references
///
/// This also converts the constant string rules into RegEx which is a heavy process
/// so if you want to make your pluralize calls faster just call this at the beginning
/// of your program.
pub fn initialize() {
    let initialized = INITIALIZED.load(Ordering::Relaxed);


    if !initialized {
        for (singular, plural) in constants::IRREGULAR_RULES.iter() {
            _add_irregular_rule(singular.to_string(), plural.to_string())
        }
        for rule in constants::UNCOUNTABLE_RULES.iter() {
            _add_uncountable_rule(rule.to_string())
        }

        INITIALIZED.store(true, Ordering::Relaxed)
    }
}

fn _add_irregular_rule(singular: String, plural: String) {
    IRREGULAR_SINGLES
        .lock()
        .unwrap()
        .insert(singular.clone(), plural.clone());
    IRREGULAR_PLURALS.lock().unwrap().insert(plural, singular);
}

/// Add an irregular word definition.
///
/// # Examples
/// ```
/// pluralizer::initialize();
///
/// pluralizer::add_irregular_rule("I".to_string(), "me".to_string());
///
/// let result = pluralizer::pluralize("I", 2, false); // me
/// ```
pub fn add_irregular_rule(singular: String, plural: String) {
    initialize();

    _add_irregular_rule(singular, plural);
}

fn _add_plural_rule(rule: String, placement: String) {
    PLURAL_RULES.lock().unwrap().push(WordRule {
        rule: Regex::new(rule.as_str()).expect("Invalid regular expression"),
        placement,
    });
}

/// Add a pluralization rule to the collection.
///
/// The rule argument must be a regular expression string.
///
/// # Examples
/// ```
/// pluralizer::initialize();
///
/// pluralizer::add_plural_rule("(?i)(matr|cod|mur|sil|vert|ind|append)(?:ix|ex)$".to_string(), "$1ices".to_string());
///
/// let result = pluralizer::pluralize("Vertex", 2, false); // Vertices
/// ```
pub fn add_plural_rule(rule: String, placement: String) {
    initialize();

    _add_irregular_rule(rule, placement);
}

fn _add_singular_rule(rule: String, placement: String) {
    SINGULAR_RULES.lock().unwrap().push(WordRule {
        rule: Regex::new(rule.as_str()).expect("Invalid regular expression"),
        placement,
    });
}

/// Add a singularization rule to the collection.
///
/// The rule argument must be a regular expression string.
///
/// # Examples
/// ```
/// pluralizer::initialize();
///
/// pluralizer::add_singular_rule("(?i)(matr|append)ices$".to_string(), "$1ix".to_string());
///
/// let result = pluralizer::pluralize("Matrices", 1, false); // Matrix
/// ```
pub fn add_singular_rule(rule: String, placement: String) {
    initialize();

    _add_irregular_rule(rule, placement);
}

fn _add_uncountable_rule(rule: String) {
    // Is regex
    if rule.starts_with("(?i)") {
        _add_plural_rule(rule.clone(), "$0".to_string());
        _add_singular_rule(rule, "$0".to_string());
    } else {
        UNCOUNTABLE_RULES.lock().unwrap().push(rule.to_lowercase());
    }
}

/// Add an uncountable word rule.
///
/// The rule can be either a word or a RegEx
///
/// # Examples
/// ```
/// pluralizer::initialize();
///
/// pluralizer::add_uncountable_rule("cash".to_string());
///
/// let result = pluralizer::pluralize("Cash", 2, false); // Cash
/// ```
pub fn add_uncountable_rule(rule: String) {
    initialize();

    _add_uncountable_rule(rule);
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
        return restore_case(
            word,
            token,
        );
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
/// pluralizer::initialize();
///
/// pluralizer::pluralize("House", 2, true); // 2 Houses
/// pluralizer::pluralize("Houses", 1, true); // 1 House
/// pluralizer::pluralize("House", 1, false); // House
/// pluralizer::pluralize("Houses", 2, false); // Houses
/// ```
pub fn pluralize(word: &str, count: isize, include_count: bool) -> String {
    initialize();

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
