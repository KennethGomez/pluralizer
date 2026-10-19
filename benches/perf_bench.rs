// The crate's minimum supported Rust version does not apply to benches
#![allow(clippy::incompatible_msrv)]

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use pluralizer::{
    add_irregular_rule, add_plural_rule, add_uncountable_rule, pluralize, UncountableRule,
};
use regex::Regex;
use std::hint::black_box;
use std::thread;
use std::time::Instant;

// A mix of regular, irregular, uncountable and non-ASCII words, in both forms
#[rustfmt::skip]
const WORDS: &[&str] = &[
    "house", "houses", "car", "cars", "box", "boxes", "church", "churches", "city", "cities",
    "day", "days", "wife", "wives", "leaf", "leaves", "hero", "heroes", "photo", "photos",
    "person", "people", "child", "children", "mouse", "mice", "goose", "geese", "tooth",
    "teeth", "ox", "oxen", "sheep", "fish", "information", "news", "series", "cactus", "cacti",
    "criterion", "criteria", "analysis", "analyses", "matrix", "matrices", "index", "indices",
    "status", "statuses", "quiz", "quizzes", "cookie", "cookies", "zombie", "zombies",
    "woman", "women", "fireman", "firemen", "walkie-talkie", "walkie-talkies", "black olive",
    "black olives", "database", "databases", "camelCase", "camelCases", "café", "naïve",
    "pokémon", "日本語",
];

fn title(word: &str) -> String {
    let mut chars = word.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

fn corpus() -> Vec<String> {
    WORDS
        .iter()
        .flat_map(|word| [word.to_string(), title(word), word.to_uppercase()])
        .collect()
}

fn pluralize_all(words: &[String], count: isize) {
    for word in words {
        black_box(pluralize(black_box(word), count, false));
    }
}

pub fn pluralizer_bench(c: &mut Criterion) {
    c.bench_function("pluralize 10k calls", |b| {
        b.iter(|| {
            for _ in 0..10_000 {
                black_box(pluralize(black_box("house"), 2, false));
            }
        });
    });

    let words = corpus();
    let mut group = c.benchmark_group("corpus");
    group.throughput(Throughput::Elements(words.len() as u64));
    for (direction, count) in [("plural", 2), ("singular", 1)] {
        group.bench_function(direction, |b| b.iter(|| pluralize_all(&words, count)));

        // Only the first thread to use a regex gets its fastest path, so measure another one too
        group.bench_function(format!("{direction} (other thread)"), |b| {
            b.iter_custom(|iters| {
                let words = &words;
                thread::scope(|scope| {
                    scope
                        .spawn(move || {
                            let start = Instant::now();
                            for _ in 0..iters {
                                pluralize_all(words, count);
                            }
                            start.elapsed()
                        })
                        .join()
                        .unwrap()
                })
            });
        });
    }
    group.finish();
}

// Rules are global and can't be removed, so this has to run last
pub fn custom_rules_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("custom rules");
    let mut added = 0;
    for rules in [0, 10, 100] {
        while added < rules {
            add_plural_rule(
                Regex::new(&format!("(?i)zzrule{added}$")).unwrap(),
                "zzrules".to_string(),
            );
            add_irregular_rule(format!("zzsingular{added}"), format!("zzplural{added}"));
            add_uncountable_rule(UncountableRule::String(format!("zzuncountable{added}")));
            added += 1;
        }
        group.bench_with_input(BenchmarkId::new("pluralize", rules), &rules, |b, _| {
            b.iter(|| black_box(pluralize(black_box("house"), 2, false)));
        });
    }
    group.finish();
}

criterion_group!(benches, pluralizer_bench, custom_rules_bench);
criterion_main!(benches);
