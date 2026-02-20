use crate::resolve::resolve_bin;
use std::path::Path;
use std::process::Command;

pub const EXTENSIONS: &[&str] = &[
    "ts", "tsx", "js", "jsx", "mts", "cts", "mjs", "cjs", "json", "jsonc", "css",
];

pub fn is_formattable(path: &str) -> bool {
    Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| EXTENSIONS.contains(&e))
}

pub fn is_available(file_path: &str) -> bool {
    Command::new(resolve_bin("biome", file_path))
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn format(file_path: &str) {
    let biome = resolve_bin("biome", file_path);

    let output = Command::new(&biome)
        .args(["check", "--write", "--linter-enabled=false", file_path])
        .output();

    match output {
        Ok(o) if o.status.success() => {}
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr);
            // biome does not use distinct exit codes for flag errors vs formatting errors.
            // These patterns detect when --linter-enabled is unsupported (older biome).
            let is_flag_error = stderr.contains("unexpected argument")
                || stderr.contains("unrecognized option")
                || stderr.contains("unknown option");

            if !is_flag_error {
                if !stderr.is_empty() {
                    eprintln!(
                        "formatter: biome: {}",
                        stderr.lines().next().unwrap_or("(no details)")
                    );
                }
                return;
            }

            match Command::new(&biome)
                .args(["format", "--write", file_path])
                .output()
            {
                Ok(o) if !o.status.success() => {
                    let stderr = String::from_utf8_lossy(&o.stderr);
                    if !stderr.is_empty() {
                        eprintln!(
                            "formatter: biome: {}",
                            stderr.lines().next().unwrap_or("(no details)")
                        );
                    }
                }
                Err(e) => {
                    eprintln!("formatter: biome: {}", e);
                }
                _ => {}
            }
        }
        Err(e) => {
            eprintln!("formatter: biome: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formattable_extensions() {
        for ext in [
            "ts", "tsx", "js", "jsx", "mts", "cts", "mjs", "cjs", "json", "jsonc", "css",
        ] {
            assert!(is_formattable(&format!("src/app.{ext}")), "{ext}");
        }
    }

    #[test]
    fn non_formattable() {
        for path in ["src/main.rs", "README.md", "Cargo.toml", ".env"] {
            assert!(!is_formattable(path), "{path}");
        }
    }

    #[test]
    fn dotfile_not_formattable() {
        assert!(!is_formattable("/tmp/.css"));
        assert!(!is_formattable("/tmp/.ts"));
        assert!(!is_formattable(".json"));
    }

    #[test]
    fn multiple_dots_formattable() {
        assert!(is_formattable("src/app.test.ts"));
        assert!(is_formattable("src/app.module.css"));
    }

    #[test]
    fn no_extension_not_formattable() {
        assert!(!is_formattable("Makefile"));
        assert!(!is_formattable("file"));
    }

    #[test]
    fn format_nonexistent_file_does_not_panic() {
        format("/nonexistent/path/to/file.ts");
    }
}
