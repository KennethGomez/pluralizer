//! Runs in its own process so that its rules do not leak into other tests.

use pluralizer::{add_irregular_rule, pluralize};
use std::io::Write;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::thread;
use std::time::{Duration, Instant};

// Readers in both directions plus concurrent `add_irregular_rule` callers used to deadlock
// when the irregular tables had their own locks, taken in a different order by each direction.
#[test]
fn concurrent_reads_and_writes_do_not_deadlock() {
    let stop = AtomicBool::new(false);
    let progress = AtomicUsize::new(0);

    thread::scope(|scope| {
        let mut workers = Vec::new();
        for _ in 0..2 {
            workers.push(scope.spawn(|| {
                while !stop.load(Ordering::Relaxed) {
                    assert_eq!(pluralize("ourselves", 1, false), "myself");
                    progress.fetch_add(1, Ordering::Relaxed);
                }
            }));
            workers.push(scope.spawn(|| {
                while !stop.load(Ordering::Relaxed) {
                    assert_eq!(pluralize("myself", 2, false), "ourselves");
                    progress.fetch_add(1, Ordering::Relaxed);
                }
            }));
            workers.push(scope.spawn(|| {
                while !stop.load(Ordering::Relaxed) {
                    // Already built in, so this changes nothing for the readers
                    add_irregular_rule("myself".to_string(), "ourselves".to_string());
                    progress.fetch_add(1, Ordering::Relaxed);
                }
            }));
        }

        // Watch until every worker has exited, so a deadlock fails the test instead of hanging it
        let start = Instant::now();
        let mut last_progress = 0;
        let mut last_change = Instant::now();
        while !workers.iter().all(|worker| worker.is_finished()) {
            thread::sleep(Duration::from_millis(50));
            if start.elapsed() > Duration::from_secs(1) {
                stop.store(true, Ordering::Relaxed);
            }
            let current = progress.load(Ordering::Relaxed);
            if current != last_progress {
                last_progress = current;
                last_change = Instant::now();
            } else if last_change.elapsed() > Duration::from_secs(2) {
                // Deadlocked threads never finish, so the scope can't be joined. Write to stderr
                // directly, since captured test output is lost when the process exits.
                let _ = writeln!(
                    std::io::stderr(),
                    "concurrent_reads_and_writes_do_not_deadlock: no progress for 2 seconds"
                );
                std::process::exit(1);
            }
        }
    });
}

// Both directions of an irregular rule become visible at the same time.
#[test]
fn irregular_rules_are_added_atomically() {
    const RULES: usize = 20_000;
    let written = AtomicUsize::new(0);

    thread::scope(|scope| {
        scope.spawn(|| {
            for i in 0..RULES {
                add_irregular_rule(format!("blorf{i}"), format!("blorvii{i}"));
                written.store(i + 1, Ordering::Release);
            }
        });
        scope.spawn(|| loop {
            // The rule that is being added right now
            let i = written.load(Ordering::Acquire);
            if i == RULES {
                break;
            }
            let singular = format!("blorf{i}");
            let plural = pluralize(&singular, 2, false);
            if plural == format!("blorvii{i}") {
                assert_eq!(pluralize(&plural, 1, false), singular);
            }
        });
    });
}
