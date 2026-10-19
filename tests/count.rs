use pluralizer::pluralize;

#[test]
fn only_a_count_of_one_is_singular() {
    assert_eq!(pluralize("tests", 1, false), "test");
    for count in [0, 2, 5, -1, -2, isize::MIN, isize::MAX] {
        assert_eq!(pluralize("test", count, false), "tests", "count {count}");
    }
}

#[test]
fn include_count() {
    assert_eq!(pluralize("test", 5, true), "5 tests");
    assert_eq!(pluralize("test", 1, true), "1 test");
    assert_eq!(pluralize("tests", 1, true), "1 test");
    assert_eq!(pluralize("test", 0, true), "0 tests");
    assert_eq!(pluralize("test", -1, true), "-1 tests");
}

#[test]
fn empty_word() {
    assert_eq!(pluralize("", 2, false), "");
    assert_eq!(pluralize("", 1, false), "");
    assert_eq!(pluralize("", 3, true), "3 ");
}
