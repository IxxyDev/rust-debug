# broken-app

Тут лежит rust-приложение с заложенными багами и тормозными алгоритмами.
Цель — всё починить, убедиться что UB нет, и ускорить горячие места.

Эталон для сверки — `reference-app/` (лежит рядом).

## Как собрать и запустить

```bash
cargo build --release
cargo run --bin demo
cargo test
```

Бенчмарки:
```bash
cargo bench --bench baseline    # простой harness=false
cargo bench --bench criterion   # criterion со статистикой
```

## Проверки на UB/утечки/гонки

```bash
# Miri
cargo +nightly miri test

# ASan
RUSTFLAGS="-Zsanitizer=address" cargo +nightly test --tests --target aarch64-apple-darwin

# TSan (нужен флаг -Cunsafe-allow-abi-mismatch=sanitizer, иначе ругается на ABI)
RUSTFLAGS="-Zsanitizer=thread -Cunsafe-allow-abi-mismatch=sanitizer" \
  cargo +nightly test --tests --target aarch64-apple-darwin
```

Valgrind на macOS (ARM64) **не запускали** — он там просто не работает, нет поддержки этой платформы.
Для компенсации использовали Miri + ASan, которые ловят большинство тех же проблем (UB, out-of-bounds, use-after-free, утечки через Miri).

## Профилирование

Профилировали через Apple Instruments (xctrace). Нужен Xcode.
Если `xcrun` не находит `xctrace`, переключить `xcode-select`:
```bash
sudo xcode-select -s /Applications/Xcode.app/Contents/Developer
```

Запуск:
```bash
cargo build --release --bin profile_target
xctrace record --template "CPU Profiler" --output artifacts/profile.trace \
  --launch -- ./target/release/profile_target
```

Результат лежит в `artifacts/profile.trace` (открывать в Instruments.app) и текстом в `artifacts/profile_summary.txt`.

## Что было найдено и починено

6 багов:
1. **sum_even** — off-by-one + unsafe get_unchecked → UB. Переписал на итераторы.
2. **leak_buffer** — Box::into_raw() без from_raw, утечка. Убрал unsafe, простой count.
3. **normalize** — replace(' ', "") не трогал табы. Заменил на split_whitespace().
4. **average_positive** — делил на длину всего массива вместо кол-ва положительных.
5. **use_after_free** — классический use-after-free через raw pointer. Удалил функцию.
6. **race_increment** — static mut из нескольких потоков = data race. Заменил на AtomicU64.

## Оптимизации

- **slow_fib**: была рекурсия O(2^n), стала итерация O(n). Ускорение ~120000x на fib(32).
- **slow_dedup**: был линейный поиск O(n^2), стал HashSet + sort O(n log n). Ускорение ~94x.
- Плюс мелочи: убраны лишние аллокации в sum_even и leak_buffer (были unsafe циклы с to_vec/into_boxed_slice).

## Структура artifacts/

- `REPORT.md` — подробный отчет со всеми деталями
- `baseline_before.txt`, `baseline_after.txt` — замеры до/после
- `criterion_after.txt` — точные замеры criterion
- `test_before.txt`, `test_after.txt` — вывод тестов до/после
- `miri_output.txt` — прогон Miri
- `asan_output.txt`, `tsan_output.txt` — прогоны санитайзеров
- `profile.trace` — файл Instruments (CPU Profiler)
- `profile_summary.txt` — текстовая выжимка из профиля

## Итого по проверкам

| Что              | Результат        |
|-----------------|------------------|
| cargo test       | 21 passed       |
| Miri             | 21 passed, 0 UB |
| ASan             | 21 passed, чисто |
| TSan             | 21 passed, чисто |
| Valgrind         | не запускали (macOS ARM64 не поддерживается) |
