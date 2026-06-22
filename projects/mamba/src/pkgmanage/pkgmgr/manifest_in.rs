// manifest_in.rs — MANIFEST.in parser for sdist building.
//
// `MANIFEST.in` is the setuptools-defined include/exclude template that
// controls which files end up in a source distribution. uv reads it when
// building an sdist from a setuptools-style project, so mamba's pkg
// manager needs the same surface.
//
// The format is one command per line. Comments start with `#` and run to
// end of line. Blank lines are ignored. Setuptools does NOT support line
// continuations, so neither do we.
//
// Commands (per the setuptools docs and the sdist PEP-625-adjacent tooling
// surface):
//
//     include          PAT  [PAT ...]
//     exclude          PAT  [PAT ...]
//     recursive-include DIR PAT [PAT ...]
//     recursive-exclude DIR PAT [PAT ...]
//     global-include   PAT  [PAT ...]
//     global-exclude   PAT  [PAT ...]
//     graft            DIR  [DIR ...]
//     prune            DIR  [DIR ...]
//
// Glob semantics (`*`, `?`, `[seq]`, `**`) are NOT expanded at parse time:
// downstream code is responsible for matching `Pat` strings against an
// actual filesystem listing.

use crate::pkgmanage::pkgmgr::types::IndexError;

/// One parsed MANIFEST.in directive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManifestCommand {
    Include(Vec<String>),
    Exclude(Vec<String>),
    RecursiveInclude { dir: String, patterns: Vec<String> },
    RecursiveExclude { dir: String, patterns: Vec<String> },
    GlobalInclude(Vec<String>),
    GlobalExclude(Vec<String>),
    Graft(Vec<String>),
    Prune(Vec<String>),
}

/// Strip a `#` end-of-line comment from a line. The `#` must be preceded
/// by whitespace or be at column zero — `#` inside a token is left alone
/// so that patterns containing `#` survive.
fn strip_comment(line: &str) -> &str {
    let bytes = line.as_bytes();
    let mut prev_ws = true;
    for (i, &b) in bytes.iter().enumerate() {
        if b == b'#' && prev_ws {
            return &line[..i];
        }
        prev_ws = (b as char).is_whitespace();
    }
    line
}

fn tokenize_line(line: &str) -> Vec<String> {
    line.split_whitespace().map(str::to_string).collect()
}

/// Parse a complete MANIFEST.in source into a list of commands. Returns
/// the first malformed line as a `ParseError`.
pub fn parse_manifest_in(src: &str) -> Result<Vec<ManifestCommand>, IndexError> {
    let mut out = Vec::new();
    for (lineno, raw) in src.lines().enumerate() {
        let stripped = strip_comment(raw).trim();
        if stripped.is_empty() {
            continue;
        }
        let toks = tokenize_line(stripped);
        let cmd = toks[0].to_ascii_lowercase();
        let args: Vec<String> = toks.into_iter().skip(1).collect();
        let line_for_err = || format!("manifest.in line {}", lineno + 1);
        let cmd = match cmd.as_str() {
            "include" => {
                require_nonempty(&args, "include", lineno)?;
                ManifestCommand::Include(args)
            }
            "exclude" => {
                require_nonempty(&args, "exclude", lineno)?;
                ManifestCommand::Exclude(args)
            }
            "recursive-include" => {
                if args.len() < 2 {
                    return Err(IndexError::ParseError {
                        url: line_for_err(),
                        detail: "recursive-include requires a directory and at least one pattern"
                            .into(),
                    });
                }
                let mut it = args.into_iter();
                let dir = it.next().unwrap();
                ManifestCommand::RecursiveInclude {
                    dir,
                    patterns: it.collect(),
                }
            }
            "recursive-exclude" => {
                if args.len() < 2 {
                    return Err(IndexError::ParseError {
                        url: line_for_err(),
                        detail: "recursive-exclude requires a directory and at least one pattern"
                            .into(),
                    });
                }
                let mut it = args.into_iter();
                let dir = it.next().unwrap();
                ManifestCommand::RecursiveExclude {
                    dir,
                    patterns: it.collect(),
                }
            }
            "global-include" => {
                require_nonempty(&args, "global-include", lineno)?;
                ManifestCommand::GlobalInclude(args)
            }
            "global-exclude" => {
                require_nonempty(&args, "global-exclude", lineno)?;
                ManifestCommand::GlobalExclude(args)
            }
            "graft" => {
                require_nonempty(&args, "graft", lineno)?;
                ManifestCommand::Graft(args)
            }
            "prune" => {
                require_nonempty(&args, "prune", lineno)?;
                ManifestCommand::Prune(args)
            }
            other => {
                return Err(IndexError::ParseError {
                    url: line_for_err(),
                    detail: format!("unknown MANIFEST.in command {other:?}"),
                });
            }
        };
        out.push(cmd);
    }
    Ok(out)
}

fn require_nonempty(args: &[String], cmd: &str, lineno: usize) -> Result<(), IndexError> {
    if args.is_empty() {
        return Err(IndexError::ParseError {
            url: format!("manifest.in line {}", lineno + 1),
            detail: format!("{cmd} requires at least one pattern"),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(v: &[&str]) -> Vec<String> {
        v.iter().map(|x| (*x).to_string()).collect()
    }

    #[test]
    fn parses_include() {
        let r = parse_manifest_in("include README.md LICENSE").unwrap();
        assert_eq!(
            r,
            vec![ManifestCommand::Include(s(&["README.md", "LICENSE"]))]
        );
    }

    #[test]
    fn parses_exclude() {
        let r = parse_manifest_in("exclude .gitignore").unwrap();
        assert_eq!(r, vec![ManifestCommand::Exclude(s(&[".gitignore"]))]);
    }

    #[test]
    fn parses_recursive_include() {
        let r = parse_manifest_in("recursive-include src *.py *.pyi").unwrap();
        assert_eq!(
            r,
            vec![ManifestCommand::RecursiveInclude {
                dir: "src".into(),
                patterns: s(&["*.py", "*.pyi"]),
            }]
        );
    }

    #[test]
    fn parses_recursive_exclude() {
        let r = parse_manifest_in("recursive-exclude tests *.pyc").unwrap();
        assert_eq!(
            r,
            vec![ManifestCommand::RecursiveExclude {
                dir: "tests".into(),
                patterns: s(&["*.pyc"]),
            }]
        );
    }

    #[test]
    fn parses_global_include() {
        let r = parse_manifest_in("global-include *.txt").unwrap();
        assert_eq!(r, vec![ManifestCommand::GlobalInclude(s(&["*.txt"]))]);
    }

    #[test]
    fn parses_global_exclude() {
        let r = parse_manifest_in("global-exclude .DS_Store").unwrap();
        assert_eq!(r, vec![ManifestCommand::GlobalExclude(s(&[".DS_Store"]))]);
    }

    #[test]
    fn parses_graft() {
        let r = parse_manifest_in("graft data tests/fixtures").unwrap();
        assert_eq!(
            r,
            vec![ManifestCommand::Graft(s(&["data", "tests/fixtures"]))]
        );
    }

    #[test]
    fn parses_prune() {
        let r = parse_manifest_in("prune build").unwrap();
        assert_eq!(r, vec![ManifestCommand::Prune(s(&["build"]))]);
    }

    #[test]
    fn case_insensitive_command_keyword() {
        let r = parse_manifest_in("INCLUDE README.md").unwrap();
        assert_eq!(r, vec![ManifestCommand::Include(s(&["README.md"]))]);
    }

    #[test]
    fn blank_lines_and_comments_skipped() {
        let src = "\
# top of file
include README.md

   # indented comment
exclude foo.txt
";
        let r = parse_manifest_in(src).unwrap();
        assert_eq!(
            r,
            vec![
                ManifestCommand::Include(s(&["README.md"])),
                ManifestCommand::Exclude(s(&["foo.txt"])),
            ]
        );
    }

    #[test]
    fn trailing_comment_after_command() {
        let r = parse_manifest_in("include README.md  # keep the readme").unwrap();
        assert_eq!(r, vec![ManifestCommand::Include(s(&["README.md"]))]);
    }

    #[test]
    fn hash_inside_token_preserved() {
        // The `#` here is not preceded by whitespace, so it does not start
        // a comment — the pattern `pkg#data.dat` should survive intact.
        let r = parse_manifest_in("include pkg#data.dat").unwrap();
        assert_eq!(r, vec![ManifestCommand::Include(s(&["pkg#data.dat"]))]);
    }

    #[test]
    fn unknown_command_rejected() {
        let err = parse_manifest_in("teleport src/").unwrap_err();
        let s = err.to_string();
        assert!(s.contains("unknown MANIFEST.in command"), "got {s}");
        assert!(s.contains("manifest.in line 1"), "got {s}");
    }

    #[test]
    fn include_without_pattern_rejected() {
        let err = parse_manifest_in("include").unwrap_err();
        let s = err.to_string();
        assert!(s.contains("at least one pattern"), "got {s}");
    }

    #[test]
    fn recursive_include_without_dir_rejected() {
        let err = parse_manifest_in("recursive-include").unwrap_err();
        let s = err.to_string();
        assert!(s.contains("recursive-include"), "got {s}");
    }

    #[test]
    fn recursive_include_with_dir_only_rejected() {
        // setuptools requires both a dir and at least one pattern.
        let err = parse_manifest_in("recursive-include src").unwrap_err();
        let s = err.to_string();
        assert!(s.contains("recursive-include"), "got {s}");
        assert!(s.contains("at least one pattern"), "got {s}");
    }

    #[test]
    fn recursive_exclude_with_dir_only_rejected() {
        let err = parse_manifest_in("recursive-exclude tests").unwrap_err();
        let s = err.to_string();
        assert!(s.contains("recursive-exclude"), "got {s}");
    }

    #[test]
    fn graft_without_dir_rejected() {
        let err = parse_manifest_in("graft").unwrap_err();
        let s = err.to_string();
        assert!(s.contains("graft"), "got {s}");
    }

    #[test]
    fn multi_line_program() {
        let src = "\
# include the top-level docs and licence
include README.md LICENSE CHANGELOG.md

# pull in source recursively
recursive-include src *.py *.pyi

# but never ship caches
global-exclude __pycache__/* *.pyc

graft data
prune build
";
        let r = parse_manifest_in(src).unwrap();
        assert_eq!(
            r,
            vec![
                ManifestCommand::Include(s(&["README.md", "LICENSE", "CHANGELOG.md"])),
                ManifestCommand::RecursiveInclude {
                    dir: "src".into(),
                    patterns: s(&["*.py", "*.pyi"]),
                },
                ManifestCommand::GlobalExclude(s(&["__pycache__/*", "*.pyc"])),
                ManifestCommand::Graft(s(&["data"])),
                ManifestCommand::Prune(s(&["build"])),
            ]
        );
    }

    #[test]
    fn tabs_are_treated_as_whitespace() {
        let r = parse_manifest_in("include\tREADME.md\tLICENSE").unwrap();
        assert_eq!(
            r,
            vec![ManifestCommand::Include(s(&["README.md", "LICENSE"]))]
        );
    }

    #[test]
    fn line_number_in_error_message() {
        let src = "include README.md\n\nteleport oops\n";
        let err = parse_manifest_in(src).unwrap_err();
        let s = err.to_string();
        assert!(s.contains("line 3"), "got {s}");
    }
}
