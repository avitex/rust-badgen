use badgen;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmarks(c: &mut Criterion) {
    c.bench_function("classic", |b| {
        let font = badgen::notosans_font();
        let mut font = badgen::font(&font);
        let mut scratch = String::with_capacity(4098);
        let mut out = String::with_capacity(4098);

        b.iter(|| {
            out.clear();
            badgen::write_badge_with_font(
                &mut out,
                &badgen::Style::classic(),
                black_box("world"),
                black_box(Some("hello")),
                &mut font,
                &mut scratch,
            )
            .unwrap();
        })
    });

    c.bench_function("flat", |b| {
        let font = badgen::notosans_font();
        let mut font = badgen::font(&font);
        let mut scratch = String::with_capacity(4098);
        let mut out = String::with_capacity(4098);

        b.iter(|| {
            out.clear();
            badgen::write_badge_with_font(
                &mut out,
                &badgen::Style::flat(),
                black_box("world"),
                black_box(Some("hello")),
                &mut font,
                &mut scratch,
            )
            .unwrap();
        })
    });

    c.bench_function("default-slow", |b| {
        b.iter(|| {
            badgen::badge(
                &badgen::Style::flat(),
                black_box("world"),
                black_box(Some("hello")),
            )
            .unwrap();
        })
    });
}

criterion_group!(benches, benchmarks);
criterion_main!(benches);
