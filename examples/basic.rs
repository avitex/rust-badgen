fn main() {
    // TODO: expand this...
    let mut out = String::new();

    let mut style = badgen::Style::classic();

    style.height = 256;

    badgen::badge(
        &mut out, &style, "9", None, // Some("crates.io"),
    )
    .unwrap();

    println!("{}", out);
}
