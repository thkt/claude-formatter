mod config;
mod resolve;

use config::Config;
use resolve::resolve_bin;
use serde::Deserialize;
use std::io::{self, Read};
use std::process::Command;

const MAX_INPUT_SIZE: u64 = 10_000_000;

#[derive(Deserialize)]
struct HookInput {
    tool_name: String,
    tool_input: ToolInput,
}

#[derive(Deserialize)]
struct ToolInput {
    file_path: Option<String>,
}

fn is_formattable(path: &str) -> bool {
    let extensions = [
        ".ts", ".tsx", ".js", ".jsx", ".mts", ".cts", ".mjs", ".cjs", ".json", ".jsonc", ".css",
    ];
    extensions.iter().any(|ext| path.ends_with(ext))
}

fn main() {
    let config = Config::load();
    if !config.enabled {
        return;
    }

    let mut input_str = String::new();
    if io::stdin()
        .take(MAX_INPUT_SIZE)
        .read_to_string(&mut input_str)
        .is_err()
    {
        return;
    }

    let input: HookInput = match serde_json::from_str(&input_str) {
        Ok(v) => v,
        Err(_) => return,
    };

    if !matches!(input.tool_name.as_str(), "Write" | "Edit" | "MultiEdit") {
        return;
    }

    let file_path = match input.tool_input.file_path {
        Some(ref p) if !p.is_empty() => p.as_str(),
        _ => return,
    };

    if !is_formattable(file_path) {
        return;
    }

    let biome = resolve_bin("biome", file_path);

    let available = Command::new(&biome)
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !available {
        eprintln!("formatter: biome not found");
        return;
    }

    // biome check --write: format + organizeImports (linter disabled)
    // User controls scope via biome.json
    let output = Command::new(&biome)
        .args(["check", "--write", "--linter-enabled=false", file_path])
        .output();

    match output {
        Ok(o) if o.status.success() => {}
        Ok(_) => {
            // Fallback: --linter-enabled may not be supported in older biome
            let _ = Command::new(&biome)
                .args(["format", "--write", file_path])
                .output();
        }
        Err(e) => {
            eprintln!("formatter: biome execution failed: {}", e);
        }
    }
}
