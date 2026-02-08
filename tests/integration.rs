use broken_app::{algo, concurrency, leak_buffer, normalize, sum_even};

#[test]
fn sums_even_numbers() {
    let nums = [1, 2, 3, 4];
    assert_eq!(sum_even(&nums), 6);
}

#[test]
fn counts_non_zero_bytes() {
    let data = [0_u8, 1, 0, 2, 3];
    assert_eq!(leak_buffer(&data), 3);
}

#[test]
fn dedup_preserves_uniques() {
    let uniq = algo::slow_dedup(&[5, 5, 1, 2, 2, 3]);
    assert_eq!(uniq, vec![1, 2, 3, 5]);
}

#[test]
fn fib_small_numbers() {
    assert_eq!(algo::slow_fib(10), 55);
}

#[test]
fn normalize_simple() {
    assert_eq!(normalize(" Hello World "), "helloworld");
}

#[test]
fn averages_only_positive() {
    let nums = [-5, 5, 15];
    assert!((broken_app::average_positive(&nums) - 10.0).abs() < f64::EPSILON);
}

#[test]
fn sum_even_empty_slice() {
    assert_eq!(sum_even(&[]), 0);
}

#[test]
fn sum_even_single_element() {
    assert_eq!(sum_even(&[2]), 2);
    assert_eq!(sum_even(&[3]), 0);
}

#[test]
fn sum_even_large_input() {
    let data: Vec<i64> = (0..10_000).collect();
    let expected: i64 = (0..10_000).filter(|v| v % 2 == 0).sum();
    assert_eq!(sum_even(&data), expected);
}

#[test]
fn leak_buffer_empty() {
    assert_eq!(leak_buffer(&[]), 0);
}

#[test]
fn leak_buffer_all_zeros() {
    assert_eq!(leak_buffer(&[0, 0, 0]), 0);
}

#[test]
fn normalize_tabs_and_multiple_spaces() {
    assert_eq!(normalize("  Hello\t\tWorld  "), "helloworld");
}

#[test]
fn normalize_tab_only() {
    assert_eq!(normalize("\tHello\tWorld\t"), "helloworld");
}

#[test]
fn average_positive_all_negative() {
    assert_eq!(broken_app::average_positive(&[-1, -2, -3]), 0.0);
}

#[test]
fn average_positive_empty() {
    assert_eq!(broken_app::average_positive(&[]), 0.0);
}

#[test]
fn average_positive_mixed() {
    let nums = [-10, -5, 0, 5, 10];
    assert!((broken_app::average_positive(&nums) - 7.5).abs() < f64::EPSILON);
}

#[test]
fn race_increment_is_correct() {
    let total = concurrency::race_increment(1_000, 4);
    assert_eq!(total, 4_000);

    let total = concurrency::race_increment(10_000, 8);
    assert_eq!(total, 80_000);
}

#[test]
fn dedup_empty() {
    let uniq = algo::slow_dedup(&[]);
    assert_eq!(uniq, Vec::<u64>::new());
}

#[test]
fn dedup_no_duplicates() {
    let uniq = algo::slow_dedup(&[3, 1, 2]);
    assert_eq!(uniq, vec![1, 2, 3]);
}

#[test]
fn fib_base_cases() {
    assert_eq!(algo::slow_fib(0), 0);
    assert_eq!(algo::slow_fib(1), 1);
    assert_eq!(algo::slow_fib(2), 1);
}

#[test]
fn fib_large() {
    assert_eq!(algo::slow_fib(32), 2_178_309);
}
