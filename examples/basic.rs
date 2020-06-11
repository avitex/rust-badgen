fn main() {
    // TODO: expand this...
    let mut out = String::new();

    badgen::badge(
        &mut out,
        &badgen::Style::classic(),
        "99.99",
        Some("crates.io"),
    )
    .unwrap();

    println!("{}", out);
}
