//! Binary detection for formatters

use std::path::PathBuf;

/// Find a binary on PATH using `which`
pub fn find_binary(name: &str) -> Option<PathBuf> {
    let output = std::process::Command::new("which")
        .arg(name)
        .output()
        .ok()?;

    if output.status.success() {
        let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if path_str.is_empty() {
            None
        } else {
            Some(PathBuf::from(path_str))
        }
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_binary_known() {
        let result = find_binary("ls");
        assert!(result.is_some(), "ls should be found on PATH");
        assert!(result.unwrap().exists(), "returned path should exist");
    }

    #[test]
    fn test_find_binary_nonexistent() {
        let result = find_binary("this_binary_should_not_exist_xyz_12345");
        assert!(result.is_none(), "nonexistent binary should return None");
    }
}
