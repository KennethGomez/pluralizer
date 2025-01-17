use criterion::{black_box, criterion_group, criterion_main, Criterion};
use pluralizer::{add_irregular_rule, add_uncountable_rule, pluralize, UncountableRule};

pub fn pluralizer_bench(c: &mut Criterion) {
    c.bench_function("pluralize 10k calls", |b| {
        b.iter(|| {
            for _ in 0..10_000 {
                black_box(pluralize("house", 2, false));
            }
        })
    });

    c.bench_function("add rules + pluralize", |b| {
        b.iter(|| {
            let _ = add_irregular_rule("child".to_string(), "children".to_string());
            let _ = add_uncountable_rule(UncountableRule::String("money".to_string()));
            black_box(pluralize("child", 2, false));
        })
    });
}

criterion_group!(benches, pluralizer_bench);
criterion_main!(benches);
