fn main() {
    let badge = badgen::badge(&badgen::Style::classic(), "99.99", Some("crates.io")).unwrap();

    println!("{}", badge);
}
