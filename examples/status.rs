fn main() {
    let mut style = badgen::Style::classic();
    style.background = badgen::Color::Orange;
    let badge = badgen::badge(&style, "development", Some("status")).unwrap();

    println!("{}", badge);
}
