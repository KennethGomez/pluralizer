pub(crate) mod constants;

use std::collections::HashMap;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use lazy_static::lazy_static;
use regex::Regex;

#[derive(Debug, Clone)]
struct WordRule {
    rule: Regex,
    placement: String,
}

lazy_static! {
    static ref IRREGULAR_SINGLES: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
    static ref IRREGULAR_PLURALS: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
    static ref PLURAL_RULES: Mutex<Vec<WordRule>> = Mutex::new(Vec::new());
    static ref SINGULAR_RULES: Mutex<Vec<WordRule>> = Mutex::new(Vec::new());
    static ref UNCOUNTABLE_RULES: Mutex<Vec<String>> = Mutex::new(Vec::new());
    static ref INITIALIZED: AtomicBool = AtomicBool::new(false);
}

pub fn initialize() {
    let initialized = INITIALIZED.load(Ordering::SeqCst);

    if !initialized {
        for [singular, plural] in constants::IRREGULAR_RULES.iter() {
            _add_irregular_rule(singular.to_string(), plural.to_string())
        }
        for [rule, placement] in constants::PLURAL_RULES.iter() {
            _add_plural_rule(rule.to_string(), placement.to_string())
        }
        for [rule, placement] in constants::SINGULAR_RULES.iter() {
            _add_singular_rule(rule.to_string(), placement.to_string())
        }
        for rule in constants::UNCOUNTABLE_RULES.iter() {
            _add_uncountable_rule(rule.to_string())
        }

        INITIALIZED.store(true, Ordering::SeqCst)
    }
}

fn _add_irregular_rule(singular: String, plural: String) {
    IRREGULAR_SINGLES
        .lock()
        .unwrap()
        .insert(singular.clone(), plural.clone());
    IRREGULAR_PLURALS.lock().unwrap().insert(plural, singular);
}

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

pub fn add_plural_rule(rule: String, placement: String) {
    initialize();

    _add_irregular_rule(rule, placement);
}

fn _add_singular_rule(rule: String, placement: String) {
    SINGULAR_RULES
        .lock()
        .unwrap()
        .push(WordRule {
            rule: Regex::new(rule.as_str()).expect("Invalid regular expression"),
            placement
        });
}

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

pub fn add_uncountable_rule(rule: String) {
    initialize();

    _add_uncountable_rule(rule);
}

fn restore_case(word: &str, token: String) -> String {
    // Tokens are an exact match.
    if word.eq(&token) {
        return token;
    }

    // Lower cased words. E.g. "hello".
    if word.eq(&word.to_lowercase()) {
        return token.to_lowercase();
    }

    // Upper cased words. E.g. "WHISKY".
    if word.eq(&word.to_uppercase()) {
        return word.to_uppercase();
    }

    // Title cased words. E.g. "Title".
    let first_char = word
        .chars()
        .nth(0)
        .expect("Trying to restore case of empty word")
        .to_string();

    if first_char.eq(&first_char.to_uppercase()) {
        let token_first_char = token
            .chars()
            .nth(0)
            .expect("Trying to restore case of empty token")
            .to_uppercase();

        let last = if token.len() > 1 {
            &token[1..token.len() - 1]
        } else {
            ""
        };

        return format!("{}{}", token_first_char, last);
    }

    // Lower cased words. E.g. "test".
    token.to_lowercase()
}

fn sanitize_word(token: String, word: &str, rules: Vec<WordRule>) -> String {
    // Empty string or doesn't need fixing.
    if token.len() == 0 || UNCOUNTABLE_RULES.lock().unwrap().contains(&token) {
        return word.to_string();
    }

    // Iterate over the sanitization rules and use the first one to match.
    for word_rule in rules.iter().rev() {
        if word_rule.rule.is_match(word) {
            return word_rule.rule.replace(word, |caps: &regex::Captures| {
                    let mut str = word_rule.clone().placement;

                    for (i, m) in caps
                        .iter()
                        .filter(|m| m.is_some())
                        .map(|m| m.unwrap())
                        .enumerate()
                    {
                        str = str.replace(format!("${}", i).as_str(), m.as_str());
                    }

                    str
                })
                .to_string();
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
        return restore_case(word, token);
    }

    // Check against the replacement map for a direct word replacement.
    if replace_map.contains_key(&token) {
        return restore_case(
            word,
            replace_map
                .get(&*token)
                .expect(
                    format!("Word `{}` doesnt't have a replace value ({})", word, token).as_str(),
                )
                .to_string(),
        );
    }

    // Run all the rules against the word.
    sanitize_word(token, word, rules)
}

fn to_singular(word: &str) -> String {
    replace_word(
        IRREGULAR_PLURALS.lock().unwrap().clone(),
        IRREGULAR_SINGLES.lock().unwrap().clone(),
        SINGULAR_RULES.lock().unwrap().clone(),
        word,
    )
}

fn to_plural(word: &str) -> String {
    replace_word(
        IRREGULAR_SINGLES.lock().unwrap().clone(),
        IRREGULAR_PLURALS.lock().unwrap().clone(),
        PLURAL_RULES.lock().unwrap().clone(),
        word,
    )
}

pub fn pluralize(word: &str, count: isize, inclusive: bool) -> String {
    initialize();

    let pluralized: String = if count == 1 {
        to_singular(word)
    } else {
        to_plural(word)
    };

    let mut out: String = String::new();

    if inclusive {
        out.push_str(format!("{} ", count).as_str())
    }

    out.push_str(pluralized.as_str());

    out
}

fn main() {
    println!("{}", pluralize("house", 2, true));
    println!("{}", pluralize("man", 1, true));
    println!("{}", pluralize("messes", 1, true));
    println!("{}", pluralize("chinese", 2, true));
}
