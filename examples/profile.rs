fn main() {
    let font = badgen::notosans_font();
    let mut font = badgen::font(&font);
    let mut scratch = String::with_capacity(4098);
    let mut out = String::with_capacity(4098);

    for _ in 0..10_000 {
        out.clear();
        badgen::write_badge_with_font(
            &mut out,
            &badgen::Style::classic(),
            "99.99",
            Some("crates.io"),
            &mut font,
            &mut scratch,
        )
        .unwrap();
    }
}
