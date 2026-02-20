use std::path::{Path, PathBuf};

pub fn resolve_bin(name: &str, file_path: &str) -> PathBuf {
    let mut dir = Path::new(file_path).parent();
    while let Some(d) = dir {
        let candidate = d.join("node_modules/.bin").join(name);
        if candidate.exists() {
            return candidate;
        }
        dir = d.parent();
    }
    PathBuf::from(name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn finds_bin_in_node_modules() {
        let tmp = TempDir::new().unwrap();
        let bin_dir = tmp.path().join("node_modules/.bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let bin_path = bin_dir.join("biome");
        fs::write(&bin_path, "").unwrap();

        let file_path = tmp.path().join("src/app.ts");
        let result = resolve_bin("biome", file_path.to_str().unwrap());
        assert_eq!(result, bin_path);
    }

    #[test]
    fn falls_back_to_bare_name() {
        let tmp = TempDir::new().unwrap();
        let file_path = tmp.path().join("test.ts");
        let result = resolve_bin("biome", file_path.to_str().unwrap());
        assert_eq!(result, PathBuf::from("biome"));
    }
}
