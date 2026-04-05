use colored::Color;

pub fn method_color(method: &str) -> Color {
    match method {
        "GET" => Color::Blue,
        "POST" => Color::Green,
        "PUT" => Color::Yellow,
        "PATCH" => Color::Yellow,
        "DELETE" => Color::Red,
        "OPTIONS" => Color::Cyan,
        "HEAD" => Color::Cyan,
        "---" => Color::White,
        _ => Color::White,
    }
}
