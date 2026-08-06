//! Re-runs the real jQuery-history corpus benchmark against divot's
//! actual public `line_diff` API (Change reconstruction included, not
//! just the raw imara-diff call) -- confirms the wrapping work here
//! didn't eat the underlying speedup before any binding is added.
use std::fs;
use std::time::Instant;

use divot::{line_diff, line_diff_many, Algorithm};

fn main() {
    let corpus_dir = std::env::args().nth(1).expect("corpus dir arg required");
    let n_pairs: usize = std::env::args().nth(2).expect("pair count arg required").parse().unwrap();
    let algorithm = match std::env::args().nth(3).as_deref() {
        Some("myers") => Algorithm::Myers,
        _ => Algorithm::Histogram,
    };
    let batched = std::env::args().nth(4).as_deref() == Some("batched");

    let mut pairs = Vec::with_capacity(n_pairs);
    for i in 1..=n_pairs {
        let before_path = format!("{corpus_dir}/{i}.before");
        let after_path = format!("{corpus_dir}/{i}.after");
        if let (Ok(before), Ok(after)) = (fs::read_to_string(&before_path), fs::read_to_string(&after_path)) {
            pairs.push((before, after));
        }
    }
    eprintln!("loaded {} pairs, batched={batched}, threads={}", pairs.len(), rayon::current_num_threads());

    let pair_refs: Vec<(&str, &str)> = pairs.iter().map(|(b, a)| (b.as_str(), a.as_str())).collect();

    if batched {
        std::hint::black_box(line_diff_many(&pair_refs, algorithm));
    } else {
        for (before, after) in &pairs {
            std::hint::black_box(line_diff(before, after, algorithm));
        }
    }

    let t0 = Instant::now();
    let total_changes: usize = if batched {
        line_diff_many(&pair_refs, algorithm)
            .iter()
            .map(|changes| changes.iter().filter(|c| c.added || c.removed).count())
            .sum()
    } else {
        pairs
            .iter()
            .map(|(before, after)| {
                line_diff(before, after, algorithm).iter().filter(|c| c.added || c.removed).count()
            })
            .sum()
    };
    let elapsed = t0.elapsed();

    println!("pairs: {}", pairs.len());
    println!("total_change_runs: {total_changes}");
    println!("total_time_ms: {:.3}", elapsed.as_secs_f64() * 1000.0);
    println!("per_pair_us: {:.2}", elapsed.as_secs_f64() * 1_000_000.0 / pairs.len() as f64);
}
