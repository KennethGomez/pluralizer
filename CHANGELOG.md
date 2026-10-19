# Changelog

## 0.6.0 - 2026-10-19

The public API is unchanged, but some outputs change: results now follow the case of the input
word more closely. For lowercase and Title-case words they match the `pluralize` NPM package
(on its `master` branch, which these rules mirror) for every word in `/usr/share/dict/words`.
Outputs not listed below are unchanged.

### Changed output

- Title-case words keep their capital when the whole word is replaced:
  `pluralize("Foot", 1, false)` returned `"foot"` and now returns `"Foot"`. Likewise
  `"Goose"` becomes `"Geese"` (was `"geese"`), `"Man"` becomes `"Men"`, `"He"` becomes `"They"`
  and `"Is"` becomes `"Are"`.
- A replacement for text that starts with a capital letter keeps that capital: `"SuperMan"`
  becomes `"SuperMen"` (was `"Supermen"`), `"the Man"` becomes `"the Men"` (was `"the men"`) and
  `"MacOS"` stays `"MacOS"` (was `"MacOs"`).
- A replacement for an UPPERCASE run inside a mixed-case word stays uppercase: `"myPERSON"`
  becomes `"myPEOPLE"` (was `"mypeople"`), and `"myABACUS"` becomes `"myABACUSES"` (was
  `"myABAcuses"`).
- Text that a rule captures from the word keeps its case: `"camelMatrix"` becomes
  `"camelMatrices"` (was `"camelmatrices"`), and `"mySheep"` stays `"mySheep"` (was
  `"mysheep"`).
- Irregular words are matched case-insensitively:
  - `pluralize("I", 2, false)` is `"WE"` (was `"IS"`), and `pluralize("i", 2, false)` is `"we"`
    (was `"is"`).
  - Irregular rules added with capitals, such as `add_irregular_rule("Octopus", "Octopodes")`,
    now apply. They used to be ignored.
  - For words that are not all lowercase or all UPPERCASE, a replacement written with capitals
    of its own (`"McDonalds"`) is returned as written, and an all-lowercase one takes the word's
    capitals on the letters the two start with: with `add_irregular_rule("mcdonald",
    "mcdonalds")`, `"McDonald"` becomes `"McDonalds"`.
  - A word already in the requested form is returned as it is, so `"McDonalds"` stays
    `"McDonalds"` and `"fEET"` stays `"fEET"` (was `"feet"`).

### Fixed

- `pluralize` no longer panics on Title-case words where a rule matches text that starts with a
  multi-byte character, such as `pluralize("Café", 2, false)`, `"Résumé"` or `"Hello 世界"`.
  These words now come back unchanged, or inflected where a rule applies
  (`pluralize("Nine—lives", 1, false)` is `"Nine—life"`).
- A `$` in the input word is kept: `pluralize("cost$", 2, false)` returned `"cost"` and now
  returns `"cost$s"`.
- Calling `add_irregular_rule` from several threads while other threads call `pluralize` with
  both singular and plural counts could deadlock forever. All rule tables now share one lock.
- `add_irregular_rule` and `add_uncountable_rule(UncountableRule::Regex(..))` update all their
  tables at once, so other threads never see a half-added rule.
- Placements of custom rules: `$N` now always means capture group `N`, even when an earlier
  group did not take part in the match; `$10` to `$99` work for rules with that many groups; and
  a `$` that is not followed by a digit is kept instead of being deleted along with the next
  character. `$0` followed by a digit still means the whole match followed by that digit.

### Performance

- `pluralize` is 2.5x to 4x faster in the crate's benchmarks: the built-in rules are matched in a
  single `RegexSet` pass, the matching rule is run once, and the word's case is only worked out
  once.
- It scales better across threads, since each call takes one lock instead of four.
- Adding the same uncountable word repeatedly no longer slows down every later call.

### Changed

- `UncountableRule` implements `Debug` and `Clone`.
- The minimum supported Rust version is declared as 1.65, which `regex` already required.
- The dependencies require `regex` 1.13.1 and `once_cell` 1.21.4, their latest releases, which
  still support Rust 1.65. The benchmarks use `criterion` 0.8.
- The license metadata uses the SPDX expression `MIT OR Apache-2.0`.
- The documentation examples assert their results. The docs now describe rule precedence, how
  counts are handled, and where results differ from the NPM package.
