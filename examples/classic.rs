fn main() {
    let badge = badgen::badge(&badgen::Style::classic(), "4.2 KB", Some("minzipped size")).unwrap();

    println!("{}", badge);
}
