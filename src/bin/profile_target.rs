use broken_app::{algo, concurrency, leak_buffer, normalize, sum_even};

fn main() {
    let data: Vec<i64> = (0..100_000).collect();
    let dedup_data: Vec<u64> = (0..10_000).flat_map(|n| [n, n]).collect();

    for _ in 0..200 {
        let _ = sum_even(&data);
    }

    for _ in 0..200 {
        let _ = algo::slow_fib(60);
    }

    for _ in 0..200 {
        let _ = algo::slow_dedup(&dedup_data);
    }

    for _ in 0..50 {
        let _ = normalize("  Hello\tWorld  from   Rust\tprogramming  ");
    }

    for _ in 0..10 {
        let _ = concurrency::race_increment(10_000, 4);
    }

    for _ in 0..200 {
        let _ = leak_buffer(&[1, 0, 2, 0, 3, 0, 4, 5, 6, 0, 7, 8]);
    }
}
