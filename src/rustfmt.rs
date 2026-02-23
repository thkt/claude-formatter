//! rustfmt integration (.rs files). Uses global PATH (distributed via rustup).

use std::path::Path;
use std::process::Command;

pub const EXTENSIONS: &[&str] = &["rs"];

pub fn is_formattable(path: &str) -> bool {
    Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| EXTENSIONS.contains(&e))
}

/// Checks global PATH — rustfmt is distributed via rustup, not node_modules.
pub fn is_available(_file_path: &str) -> bool {
    Command::new("rustfmt")
        .arg("--version")
        .output()
        .is_ok_and(|o| o.status.success())
}

pub fn format(file_path: &str) {
    match Command::new("rustfmt").arg(file_path).output() {
        Ok(o) if o.status.success() => {}
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr);
            if !stderr.is_empty() {
                eprintln!(
                    "formatter: rustfmt: {}",
                    stderr.lines().next().unwrap_or("(no details)")
                );
            }
        }
        Err(e) => eprintln!("formatter: rustfmt: {}", e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rs_files_are_formattable() {
        assert!(is_formattable("src/main.rs"));
        assert!(is_formattable("src/lib.rs"));
        assert!(is_formattable("/absolute/path.rs"));
    }

    #[test]
    fn non_rs_files_not_formattable() {
        assert!(!is_formattable("src/app.ts"));
        assert!(!is_formattable("Cargo.toml"));
    }

    #[test]
    fn dotfile_not_formattable() {
        assert!(!is_formattable(".rs"));
        assert!(!is_formattable("/tmp/.rs"));
    }

    #[test]
    fn format_nonexistent_file_does_not_panic() {
        format("/nonexistent/path/to/file.rs");
    }

    #[test]
    fn format_fixes_indentation() {
        use std::fs;
        use tempfile::TempDir;

        if !is_available("") {
            eprintln!("rustfmt not available, skipping");
            return;
        }

        let tmp = TempDir::new().unwrap();
        let file = tmp.path().join("test.rs");
        fs::write(&file, "fn main(){\nlet x=1;\n}\n").unwrap();

        format(file.to_str().unwrap());

        let content = fs::read_to_string(&file).unwrap();
        assert!(
            content.contains("    let x = 1;"),
            "Expected formatted content, got: {content}"
        );
    }
}
