fn main() {
    let font = badgen::raleway_reg_font();
    let mut renderer = badgen::ScaledFont::new(&font, 1.0);
    let mut scratch = Vec::with_capacity(4098);
    let mut out = String::with_capacity(4098);

    for _ in 0..10_000 {
        out.clear();
        badgen::write_badge_with_font(
            &mut out,
            &badgen::Style::classic(),
            "99.99",
            Some("crates.io"),
            &mut renderer,
            &mut scratch,
        )
        .unwrap();
    }
}
