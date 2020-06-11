use badgen;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("classic", |b| {
        let font = badgen::raleway_reg_font();
        let mut renderer = badgen::ScaledFont::new(&font, 1.0);
        let mut scratch = Vec::with_capacity(4098);
        let mut out = String::with_capacity(4098);

        b.iter(|| {
            out.clear();
            badgen::badge_with_font(
                &mut out,
                &badgen::Style::classic(),
                black_box("world"),
                black_box(Some("hello")),
                &mut renderer,
                &mut scratch,
            )
            .unwrap();
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
