fn use_color() -> bool {
    static COLOR: std::sync::LazyLock<bool> =
        std::sync::LazyLock::new(|| std::env::var_os("NO_COLOR").is_none());
    *COLOR
}

fn wrap_with(color: bool, ansi_code: &str, text: &str) -> String {
    if color {
        format!("\x1b[{}m{}\x1b[0m", ansi_code, text)
    } else {
        text.to_string()
    }
}

pub fn yellow(text: &str) -> String {
    wrap_with(use_color(), "33", text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn t_012_yellow_with_color_wraps_ansi() {
        // [T-012] NO_COLOR unset -> yellow wraps with ANSI codes
        let result = wrap_with(true, "33", "text");
        assert_eq!(result, "\x1b[33mtext\x1b[0m");
    }

    #[test]
    fn t_013_yellow_without_color_returns_plain() {
        // [T-013] NO_COLOR set -> yellow returns plain text
        let result = wrap_with(false, "33", "text");
        assert_eq!(result, "text");
    }
}
