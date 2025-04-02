pub fn escape_string(text: &str) -> String {
    format!(
        "\"{}\"",
        str::replace(str::replace(text, "\\", "\\\\").as_str(), "\"", "\\\"")
    )
}
