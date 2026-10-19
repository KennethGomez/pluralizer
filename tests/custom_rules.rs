//! Rules are global and tests run in parallel, so every test uses words no other test touches.

use pluralizer::{
    add_irregular_rule, add_plural_rule, add_singular_rule, add_uncountable_rule, pluralize,
    UncountableRule,
};
use regex::Regex;

#[test]
fn uncountable_string() {
    assert_eq!(pluralize("paper", 2, false), "papers");
    add_uncountable_rule(UncountableRule::String("Paper".to_string()));
    assert_eq!(pluralize("paper", 2, false), "paper");
    assert_eq!(pluralize("PAPER", 2, false), "PAPER");
    assert_eq!(pluralize("paper", 1, false), "paper");
}

#[test]
fn uncountable_string_added_twice() {
    add_uncountable_rule(UncountableRule::String("gold".to_string()));
    add_uncountable_rule(UncountableRule::String("gold".to_string()));
    assert_eq!(pluralize("gold", 2, false), "gold");
}

#[test]
fn uncountable_regex() {
    assert_eq!(pluralize("widgetz", 2, false), "widgetzs");
    add_uncountable_rule(UncountableRule::Regex(Regex::new("(?i)widgetz$").unwrap()));
    assert_eq!(pluralize("widgetz", 2, false), "widgetz");
    assert_eq!(pluralize("Gadgetwidgetz", 1, false), "Gadgetwidgetz");
}

#[test]
fn irregular_in_both_directions() {
    assert_eq!(pluralize("irregular", 2, false), "irregulars");
    add_irregular_rule("irregular".to_string(), "regular".to_string());
    assert_eq!(pluralize("irregular", 2, false), "regular");
    assert_eq!(pluralize("regular", 1, false), "irregular");
    assert_eq!(pluralize("regular", 2, false), "regular");
    assert_eq!(pluralize("irregular", 1, false), "irregular");
    assert_eq!(pluralize("IRREGULAR", 2, false), "REGULAR");
}

#[test]
fn irregular_with_capitals() {
    add_irregular_rule("Octopus".to_string(), "Octopodes".to_string());
    assert_eq!(pluralize("octopus", 2, false), "octopodes");
    assert_eq!(pluralize("Octopus", 2, false), "Octopodes");
    assert_eq!(pluralize("OCTOPODES", 1, false), "OCTOPUS");
}

#[test]
fn irregular_replacement_with_its_own_capitals() {
    add_irregular_rule("McDonald".to_string(), "McDonalds".to_string());
    assert_eq!(pluralize("McDonald", 2, false), "McDonalds");
    assert_eq!(pluralize("McDonalds", 1, false), "McDonald");
    assert_eq!(pluralize("Mcdonald", 2, false), "McDonalds");
    assert_eq!(pluralize("mcdonald", 2, false), "mcdonalds");
    assert_eq!(pluralize("MCDONALD", 2, false), "MCDONALDS");

    add_irregular_rule("iPhone".to_string(), "iPhones".to_string());
    assert_eq!(pluralize("iPhone", 2, false), "iPhones");
    assert_eq!(pluralize("iPhones", 1, false), "iPhone");
}

#[test]
fn irregular_replacement_takes_the_capitals_of_the_word() {
    add_irregular_rule("ebook".to_string(), "ebooks".to_string());
    assert_eq!(pluralize("eBook", 2, false), "eBooks");
    assert_eq!(pluralize("eBooks", 1, false), "eBook");
    assert_eq!(pluralize("EBook", 2, false), "EBooks");

    add_irregular_rule("grue".to_string(), "frue".to_string());
    assert_eq!(pluralize("Grue", 2, false), "Frue");
}

#[test]
fn irregular_words_in_the_requested_form_are_kept() {
    add_irregular_rule("PhD".to_string(), "PhDs".to_string());
    assert_eq!(pluralize("PhD", 1, false), "PhD");
    assert_eq!(pluralize("PhDs", 2, false), "PhDs");
    assert_eq!(pluralize("phds", 2, false), "phds");
    assert_eq!(pluralize("PHDS", 2, false), "PHDS");
    assert_eq!(pluralize("PhD", 2, false), "PhDs");
}

#[test]
fn placement_case() {
    add_plural_rule(Regex::new("(?i)blop$").unwrap(), "blops".to_string());
    assert_eq!(pluralize("blop", 2, false), "blops");
    assert_eq!(pluralize("Blop", 2, false), "Blops");
    assert_eq!(pluralize("myBlop", 2, false), "myBlops");
    assert_eq!(pluralize("myBLOP", 2, false), "myBLOPS");
    assert_eq!(pluralize("BLOP", 2, false), "BLOPS");

    // Kept as written in words that start with a capital, lowercased in other mixed-case words
    add_plural_rule(Regex::new("(?i)phon$").unwrap(), "pHons".to_string());
    assert_eq!(pluralize("Phon", 2, false), "pHons");
    assert_eq!(pluralize("myPhon", 2, false), "myPhons");

    // A placement that starts with a capture takes its case from the capture
    add_plural_rule(Regex::new("(?i)x(yz)$").unwrap(), "$1zy".to_string());
    assert_eq!(pluralize("Xyz", 2, false), "yzzy");
}

#[test]
fn plural_rule() {
    assert_eq!(pluralize("regex", 2, false), "regexes");
    add_plural_rule(Regex::new("(?i)gex$").unwrap(), "gexii".to_string());
    assert_eq!(pluralize("regex", 2, false), "regexii");
    assert_eq!(pluralize("REGEX", 2, false), "REGEXII");
}

#[test]
fn singular_rule() {
    assert_eq!(pluralize("singles", 1, false), "single");
    add_singular_rule(Regex::new("singles$").unwrap(), "singular".to_string());
    assert_eq!(pluralize("singles", 1, false), "singular");
    // The rule is case-sensitive, so it does not apply here
    assert_eq!(pluralize("SINGLES", 1, false), "SINGLE");
}

#[test]
fn custom_rules_take_precedence_over_builtin_ones() {
    assert_eq!(pluralize("person", 2, false), "people");
    add_plural_rule(Regex::new("(?i)^person$").unwrap(), "peeps".to_string());
    assert_eq!(pluralize("person", 2, false), "peeps");
    assert_eq!(pluralize("Person", 2, false), "Peeps");

    add_singular_rule(Regex::new("(?i)^mornings$").unwrap(), "suck".to_string());
    assert_eq!(pluralize("mornings", 1, false), "suck");
}

#[test]
fn newest_custom_rule_wins() {
    add_plural_rule(Regex::new("(?i)zork$").unwrap(), "zorkim".to_string());
    add_plural_rule(Regex::new("(?i)zork$").unwrap(), "zorkii".to_string());
    assert_eq!(pluralize("zork", 2, false), "zorkii");
}

#[test]
fn placement_capture_groups() {
    add_plural_rule(
        Regex::new("(?i)(?:(qu)a|(fr)o)bble$").unwrap(),
        "[$1|$2]bbles".to_string(),
    );
    // Groups keep their number when an earlier group does not participate
    assert_eq!(pluralize("quabble", 2, false), "[qu|]bbles");
    assert_eq!(pluralize("frobble", 2, false), "[|fr]bbles");

    add_plural_rule(Regex::new("(?i)(gro)nk$").unwrap(), "$1nk$$0".to_string());
    // A `$` that is not followed by a digit is kept
    assert_eq!(pluralize("gronk", 2, false), "gronk$gronk");

    // 11 groups, so `$10` and `$11` are read as two-digit groups
    let eleven_groups = "(a)(b)(c)(d)(e)(f)(g)(h)(i)(j)(k)$";
    add_plural_rule(Regex::new(eleven_groups).unwrap(), "$11-$10-$1".to_string());
    assert_eq!(pluralize("xabcdefghijk", 2, false), "xk-j-a");

    // Otherwise `$NN` is `$N` followed by a digit, as before
    let nine_groups = "^nine(a)(b)(c)(d)(e)(f)(g)(h)(i)$";
    add_plural_rule(Regex::new(nine_groups).unwrap(), "$10".to_string());
    assert_eq!(pluralize("nineabcdefghi", 2, false), "a0");
    let exactly_ten_groups = "^ten(a)(b)(c)(d)(e)(f)(g)(h)(i)(j)$";
    add_plural_rule(
        Regex::new(exactly_ten_groups).unwrap(),
        "$10-$11".to_string(),
    );
    assert_eq!(pluralize("tenabcdefghij", 2, false), "j-a1");

    // `$0` followed by a digit is the whole match followed by that digit
    add_plural_rule(Regex::new("^zonk$").unwrap(), "$00".to_string());
    assert_eq!(pluralize("zonk", 2, false), "zonk0");
    add_plural_rule(Regex::new("^(blo)(rp)$").unwrap(), "$01".to_string());
    assert_eq!(pluralize("blorp", 2, false), "blorp1");
}
