pub fn format_status(msg: &str) -> String {
    format!("[STATUS] {}", msg)
}

pub fn format_error(code: u32, details: &str) -> String {
    format!("[ERROR {}] {}", code, details)
}
