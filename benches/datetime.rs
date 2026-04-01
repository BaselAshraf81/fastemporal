//! Criterion benchmarks for fastemporal vs chrono vs jiff (and Luxon via Node).
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use fastemporal::{Duration, ZonedDateTime};

// ─── fastemporal benchmarks ───────────────────────────────────────────────────

fn bench_now(c: &mut Criterion) {
    c.bench_function("fastemporal/now", |b| {
        b.iter(|| black_box(ZonedDateTime::now()))
    });
}

fn bench_from_iso(c: &mut Criterion) {
    let samples = [
        "2025-06-07T14:32:00.000Z",
        "2025-01-01T00:00:00+05:30",
        "2016-03-12T10:00:00-08:00",
    ];
    let mut g = c.benchmark_group("fastemporal/from_iso");
    for s in &samples {
        g.bench_with_input(BenchmarkId::from_parameter(s), s, |b, s| {
            b.iter(|| black_box(ZonedDateTime::from_iso(black_box(s)).unwrap()))
        });
    }
    g.finish();
}

fn bench_plus_days(c: &mut Criterion) {
    let dt = ZonedDateTime::from_iso("2025-01-01T00:00:00Z").unwrap();
    let dur = Duration::days(7);
    c.bench_function("fastemporal/plus_days_7", |b| {
        b.iter(|| black_box(dt).plus(black_box(dur)))
    });
}

fn bench_to_iso(c: &mut Criterion) {
    let dt = ZonedDateTime::from_iso("2025-06-07T14:32:00.000Z").unwrap();
    c.bench_function("fastemporal/to_iso", |b| {
        b.iter(|| black_box(dt).to_iso())
    });
}

fn bench_format(c: &mut Criterion) {
    let dt = ZonedDateTime::from_iso("2025-06-07T14:32:00.000Z").unwrap();
    c.bench_function("fastemporal/format_yyyy_MM_dd", |b| {
        b.iter(|| black_box(dt).format(black_box("yyyy-MM-dd")))
    });
}

fn bench_start_of_day(c: &mut Criterion) {
    let dt = ZonedDateTime::from_iso("2025-06-07T14:32:05.123Z").unwrap();
    c.bench_function("fastemporal/start_of_day", |b| {
        b.iter(|| black_box(dt).start_of(black_box("day")).unwrap())
    });
}

fn bench_diff_days(c: &mut Criterion) {
    let a = ZonedDateTime::from_iso("2025-06-07T00:00:00Z").unwrap();
    let b = ZonedDateTime::from_iso("2024-01-01T00:00:00Z").unwrap();
    c.bench_function("fastemporal/diff_days", |b_| {
        b_.iter(|| black_box(a).diff(black_box(b), black_box("days")).unwrap())
    });
}

#[cfg(any(feature = "tz-embedded", feature = "tz-system"))]
fn bench_in_timezone(c: &mut Criterion) {
    let dt = ZonedDateTime::from_iso("2025-06-07T14:32:00Z").unwrap();
    c.bench_function("fastemporal/in_timezone_NY", |b| {
        b.iter(|| black_box(dt).in_timezone(black_box("America/New_York")).unwrap())
    });
}

// ─── 1 M tight loop ──────────────────────────────────────────────────────────

fn bench_tight_loop_plus_days(c: &mut Criterion) {
    c.bench_function("fastemporal/1M_plus_days", |b| {
        b.iter(|| {
            let mut dt = ZonedDateTime::from_iso("2020-01-01T00:00:00Z").unwrap();
            let dur = Duration::days(1);
            for _ in 0..1_000_000u32 {
                dt = dt.plus(dur);
            }
            black_box(dt)
        })
    });
}

// ─── Groups ──────────────────────────────────────────────────────────────────

criterion_group!(
    basic,
    bench_now,
    bench_from_iso,
    bench_plus_days,
    bench_to_iso,
    bench_format,
    bench_start_of_day,
    bench_diff_days,
);

criterion_group!(tight_loop, bench_tight_loop_plus_days);

#[cfg(any(feature = "tz-embedded", feature = "tz-system"))]
criterion_group!(tz_benches, bench_in_timezone);

#[cfg(any(feature = "tz-embedded", feature = "tz-system"))]
criterion_main!(basic, tight_loop, tz_benches);

#[cfg(not(any(feature = "tz-embedded", feature = "tz-system")))]
criterion_main!(basic, tight_loop);
