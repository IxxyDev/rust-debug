# Отчёт по broken-app

## 1. Баги

### sum_even — off-by-one + UB
В оригинале был цикл `for idx in 0..=values.len()` с `get_unchecked` внутри unsafe блока.
На последней итерации idx == values.len(), что вылезает за границу среза. Классический off-by-one, но из-за unsafe это ещё и UB. В debug-сборке крашится с SIGABRT, в release — непредсказуемое поведение.

Выкинул unsafe, заменил на `values.iter().copied().filter(|v| v % 2 == 0).sum()`.
Тесты: `sum_even_empty_slice`, `sum_even_single_element`, `sum_even_large_input`.

### leak_buffer — утечка памяти
`Box::into_raw()` и дальше работа через raw pointer, но `Box::from_raw()` нигде не вызывается. Память утекает при каждом вызове. Miri это ловит.

Убрал unsafe целиком: `input.iter().filter(|b| **b != 0).count()`.
Тесты: `leak_buffer_empty`, `leak_buffer_all_zeros`.

### normalize — табуляции игнорируются
`input.replace(' ', "")` убирает только пробелы, `\t` и прочие whitespace остаются.
Оригинальный тест это не ловил потому что входная строка без табов.

Заменил на `input.split_whitespace().collect::<String>().to_lowercase()`.
Добавил тесты с табами: `normalize_tabs_and_multiple_spaces`, `normalize_tab_only`.

### average_positive — неправильное деление
Считал сумму всех элементов (а не только положительных) и делил на полную длину массива.
При входе [-5, 5, 15] возвращал 5.0 вместо 10.0.

Теперь фильтруем `v > 0`, делим на `positives.len()`. Обрабатываем пустой массив и случай когда все отрицательные.
Тесты: `average_positive_all_negative`, `average_positive_empty`, `average_positive_mixed`.

### use_after_free — UB
`Box::from_raw(raw)` освобождает память, потом читаем `*raw`. Классика. Функция нигде не вызывалась кроме как была объявлена, удалил полностью.

### data race в concurrency
`static mut COUNTER: u64` + несколько потоков = data race. Заменил на `AtomicU64` с `SeqCst`. Добавил тест `race_increment_is_correct` (1000*4 и 10000*8).

## 2. Оптимизации

**slow_fib: O(2^n) -> O(n)**. Наивная рекурсия → итеративный цикл, две переменные, ноль аллокаций.

**slow_dedup: O(n^2) -> O(n log n)**. Линейный поиск через `.contains()` + сортировка при каждой вставке → HashSet с предвыделённой ёмкостью + одна `sort_unstable()` в конце.

В sum_even и leak_buffer убрал unsafe и лишние аллокации (to_vec/into_boxed_slice) — это больше про корректность, но и по перформансу стало чище.

## 3. Бенчмарки

### Baseline (harness=false, release, 3 итерации)

| Функция       | До (медиана) | После (медиана) | Ускорение |
|--------------|-------------|----------------|-----------|
| sum_even     | 41 ns       | ~10 µs*        | --        |
| slow_fib(32) | 4.95 ms     | 41 ns          | ~120000x  |
| slow_dedup   | 7.43 ms     | 79 µs          | ~94x      |

*sum_even "до" показывал 41ns из-за UB — get_unchecked пропускал bounds checking, компилятор заинлайнил. После — честный safe итератор на 50k элементов. Сравнивать напрямую некорректно.

### Criterion (после оптимизации)

| Бенч      | mean ± σ              |
|-----------|----------------------|
| sum_even  | 6.19 µs ± 0.015 µs  |
| fib_32    | 11.3 ns ± 0.08 ns   |
| dedup_10k | 57.5 µs ± 0.17 µs   |

## 4. Проверки

| Что                          | Результат           | Лог                |
|-----------------------------|---------------------|--------------------|
| cargo test                   | 21 passed, 0 failed | test_after.txt     |
| cargo +nightly miri test    | 21 passed, 0 UB     | miri_output.txt    |
| ASan (-Zsanitizer=address)  | 21 passed, чисто    | asan_output.txt    |
| TSan (-Zsanitizer=thread)   | 21 passed, чисто    | tsan_output.txt    |
| Valgrind                     | не запускали (нет на macOS ARM64) | —       |

TSan потребовал `-Cunsafe-allow-abi-mismatch=sanitizer` т.к. без -Zbuild-std std не пересобирается с санитайзером.

## 5. Профилирование

Профилировали через xctrace (Instruments CPU Profiler) на стресс-бинаре `profile_target`.

| Что                          | Доля CPU |
|------------------------------|----------|
| slow_dedup (с HashMap::insert) | ~64%   |
| спавн потоков                | ~15%     |
| sum_even                     | ~10%     |
| dyld startup                 | ~3%      |

slow_fib, leak_buffer, normalize не попали в сэмплы — слишком быстрые.

Подробнее: `profile_summary.txt`, файл Instruments: `profile.trace`.

## 6. Регрессионные тесты

15 штук, покрывают краевые случаи для каждого исправленного бага:

sum_even_empty_slice, sum_even_single_element, sum_even_large_input,
leak_buffer_empty, leak_buffer_all_zeros,
normalize_tabs_and_multiple_spaces, normalize_tab_only,
average_positive_all_negative, average_positive_empty, average_positive_mixed,
race_increment_is_correct,
dedup_empty, dedup_no_duplicates,
fib_base_cases, fib_large
