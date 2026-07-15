//! Conservative destructive-command guard used only by agent hooks.
//!
//! The guard never changes interactive `cap run` behavior. It rejects commands
//! whose destructive target cannot be proven to stay below the current Git
//! workspace, instead of attempting to interpret arbitrary shell syntax.

use std::{
    env, fs,
    path::{Component, Path, PathBuf},
};

const WRAPPERS: &[&str] = &["env", "command", "time", "nice", "nohup", "exec"];

pub fn deny_reason(command: &str) -> Option<String> {
    let words = shell_words(command)
        .or_else(|| contains_destructive_keyword(command).then(|| Vec::new()))?;
    if words.is_empty() {
        return Some("cap guard rejected destructive shell syntax it cannot safely inspect".into());
    }
    guard_words(&words)
}

fn guard_words(words: &[String]) -> Option<String> {
    let (program_index, program) = effective_program(words)?;
    match program {
        "sudo" | "doas" | "dd" | "diskutil" => Some(format!(
            "cap guard blocks privileged or disk-destructive command `{program}`"
        )),
        program if program.starts_with("mkfs") => Some(format!(
            "cap guard blocks filesystem creation command `{program}`"
        )),
        "rm" | "rmdir" | "unlink" => guard_remove(&words[program_index + 1..]),
        "find"
            if words[program_index + 1..]
                .iter()
                .any(|word| word == "-delete") =>
        {
            Some(
                "cap guard blocks `find -delete`; use an explicit workspace-relative rm target"
                    .into(),
            )
        }
        "git" => guard_git(&words[program_index + 1..]),
        _ => None,
    }
}

fn effective_program(words: &[String]) -> Option<(usize, &str)> {
    for (index, word) in words.iter().enumerate() {
        if WRAPPERS.contains(&word.as_str()) || is_assignment(word) {
            continue;
        }
        return Some((index, word.rsplit('/').next().unwrap_or(word)));
    }
    None
}

fn is_assignment(word: &str) -> bool {
    let Some((name, _)) = word.split_once('=') else {
        return false;
    };
    !name.is_empty()
        && name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

fn guard_remove(args: &[String]) -> Option<String> {
    let cwd = env::current_dir().ok()?;
    guard_remove_at(&cwd, args)
}

fn guard_remove_at(cwd: &Path, args: &[String]) -> Option<String> {
    let mut operands = Vec::new();
    let mut options = true;
    for arg in args {
        if options && arg == "--" {
            options = false;
        } else if !options && !arg.is_empty() {
            operands.push(arg);
        } else if !options || !arg.starts_with('-') {
            operands.push(arg);
        }
    }
    if operands.is_empty() {
        return None;
    }
    let Some(root) = workspace_root(cwd) else {
        return Some("cap guard blocks deletion because no Git workspace root is available".into());
    };
    let cwd = fs::canonicalize(cwd).ok()?;
    if !cwd.starts_with(&root) {
        return Some(
            "cap guard blocks deletion because the agent cwd is outside its Git workspace".into(),
        );
    }
    for operand in operands {
        if operand == "~" || operand.starts_with("~/") {
            return Some("cap guard blocks home-directory deletion from an agent hook".into());
        }
        let target = resolve_target(&cwd, operand);
        if target == root || !target.starts_with(&root) {
            return Some(format!(
                "cap guard blocks deletion target `{operand}` outside the workspace `{}`",
                root.display()
            ));
        }
    }
    None
}

fn guard_git(args: &[String]) -> Option<String> {
    match args.first().map(String::as_str) {
        Some("reset") if args.iter().any(|arg| arg == "--hard") => {
            Some("cap guard blocks `git reset --hard` from an agent hook".into())
        }
        Some("clean")
            if args
                .iter()
                .any(|arg| arg.starts_with('-') && arg.contains('f')) =>
        {
            Some("cap guard blocks forceful `git clean` from an agent hook".into())
        }
        _ => None,
    }
}

fn workspace_root(cwd: &Path) -> Option<PathBuf> {
    cwd.ancestors()
        .find(|dir| dir.join(".git").exists())
        .and_then(|dir| fs::canonicalize(dir).ok())
}

fn resolve_target(cwd: &Path, operand: &str) -> PathBuf {
    let raw = Path::new(operand);
    let candidate = if raw.is_absolute() {
        raw.to_path_buf()
    } else {
        cwd.join(raw)
    };
    fs::canonicalize(&candidate).unwrap_or_else(|_| lexical_normalize(&candidate))
}

fn lexical_normalize(path: &Path) -> PathBuf {
    let mut result = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => result.push(prefix.as_os_str()),
            Component::RootDir => result.push(component.as_os_str()),
            Component::CurDir => {}
            Component::ParentDir => {
                result.pop();
            }
            Component::Normal(part) => result.push(part),
        }
    }
    result
}

fn contains_destructive_keyword(command: &str) -> bool {
    [
        "rm", "rmdir", "unlink", "find", "sudo", "doas", "dd", "mkfs", "diskutil",
    ]
    .iter()
    .any(|needle| {
        command
            .split(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_')
            .any(|word| word == *needle)
    })
}

fn shell_words(command: &str) -> Option<Vec<String>> {
    let mut words = Vec::new();
    let mut current = String::new();
    let mut quote = None;
    for ch in command.chars() {
        if let Some(active) = quote {
            if ch == active {
                quote = None;
            } else if active == '"' && matches!(ch, '$' | '`' | '\\') {
                return None;
            } else {
                current.push(ch);
            }
            continue;
        }
        match ch {
            '\'' | '"' => quote = Some(ch),
            '$' | '`' | '\\' | '|' | '&' | ';' | '<' | '>' | '\n' => return None,
            ch if ch.is_whitespace() => {
                if !current.is_empty() {
                    words.push(std::mem::take(&mut current));
                }
            }
            _ => current.push(ch),
        }
    }
    if quote.is_some() {
        return None;
    }
    if !current.is_empty() {
        words.push(current);
    }
    Some(words)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn rejects_destructive_commands_without_needing_a_workspace() {
        for command in [
            "sudo rm -rf /",
            "rm -rf /",
            "dd if=/dev/zero of=/dev/disk1",
            "mkfs.ext4 /dev/sda",
            "find . -delete",
            "git reset --hard",
            "git clean -fdx",
        ] {
            assert!(
                guard_words(&shell_words(command).unwrap()).is_some(),
                "{command}"
            );
        }
        assert!(deny_reason("rm -rf $HOME").is_some());
        assert!(deny_reason("rm -rf \"$HOME\"").is_some());
    }

    #[test]
    fn deletion_boundary_allows_descendants_and_rejects_workspace_root_or_parent() {
        let tmp = tempdir().unwrap();
        let root = tmp.path().join("repo");
        let child = root.join("nested");
        fs::create_dir_all(&child).unwrap();
        fs::create_dir(root.join(".git")).unwrap();
        assert!(guard_remove_at(&child, &["file.txt".into()]).is_none());
        assert!(guard_remove_at(&child, &["../..".into()]).is_some());
        assert!(guard_remove_at(&child, &["..".into()]).is_some());
        assert!(guard_remove_at(tmp.path(), &["anything".into()]).is_some());
    }
}
