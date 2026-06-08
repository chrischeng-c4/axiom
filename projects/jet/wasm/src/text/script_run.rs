// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-wasm-src-text.md#schema
// CODEGEN-BEGIN
//! UAX #24 script-itemization via `unicode-script`.
//!
//! @spec .aw/tech-design/projects/jet/wasm-renderer/line-bidi.md#schema
//!
//! Algorithm (Common/Inherited lookahead-merge per the schema):
//!   1. Walk codepoints in logical order, looking up each one's
//!      `Script` property via `unicode-script`.
//!   2. Group contiguous codepoints sharing the same CONCRETE script
//!      into one `ScriptRun`. Common (`Zyyy`) and Inherited (`Zinh`)
//!      codepoints are pending — they merge into the FIRST concrete
//!      script run that FOLLOWS them in logical order.
//!   3. If no concrete script appears anywhere, emit a single
//!      `ScriptRun{ script: "Common" }` covering the entire text.
//!
//! `script` values are the `Script::full_name()` PascalCase form
//! ("Latin", "Han", "Arabic", "Common", ...). The 4-letter
//! `short_name()` form is non-conforming and never produced.

use std::ops::Range;

use unicode_script::{Script, UnicodeScript};

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-text.md#schema
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptRun {
    pub byte_range: Range<usize>,
    /// `unicode_script::Script::full_name()` PascalCase value.
    pub script: String,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-wasm-src-text.md#schema
pub fn script_runs(text: &str) -> Vec<ScriptRun> {
    if text.is_empty() {
        return Vec::new();
    }

    // Phase 1: per-codepoint (byte_offset, byte_len, classification).
    enum Cls {
        Concrete(Script),
        Pending, // Common / Inherited
    }

    let mut codepoints: Vec<(usize, usize, Cls)> = Vec::with_capacity(text.len());
    for (offset, ch) in text.char_indices() {
        let s = ch.script();
        let cls = if s == Script::Common || s == Script::Inherited {
            Cls::Pending
        } else {
            Cls::Concrete(s)
        };
        codepoints.push((offset, ch.len_utf8(), cls));
    }

    // Phase 2: lookahead-merge — for each contiguous block, attach
    // pending codepoints to the FIRST concrete script that follows.
    let mut runs: Vec<ScriptRun> = Vec::new();
    let mut i = 0;
    while i < codepoints.len() {
        // Skip pending; find next concrete or hit end.
        let pending_start = i;
        while i < codepoints.len() && matches!(codepoints[i].2, Cls::Pending) {
            i += 1;
        }
        if i >= codepoints.len() {
            // Trailing pending after some prior concrete run — merge into the
            // last run if any; otherwise pure-Common case (handled below).
            if let Some(last) = runs.last_mut() {
                let last_cp = &codepoints[codepoints.len() - 1];
                last.byte_range.end = last_cp.0 + last_cp.1;
            }
            break;
        }
        // i is the first concrete codepoint of this run.
        let concrete_script = match &codepoints[i].2 {
            Cls::Concrete(s) => *s,
            Cls::Pending => unreachable!(),
        };
        let run_start_byte = codepoints[pending_start].0;
        // Extend across same-concrete-script + pending codepoints.
        let mut run_end_byte = codepoints[i].0 + codepoints[i].1;
        i += 1;
        while i < codepoints.len() {
            match &codepoints[i].2 {
                Cls::Concrete(s) if *s == concrete_script => {
                    run_end_byte = codepoints[i].0 + codepoints[i].1;
                    i += 1;
                }
                Cls::Pending => {
                    // Lookahead: peek past the Pending block. If the next
                    // concrete codepoint is the SAME script, coalesce the
                    // Pending into the current run. If it's a DIFFERENT
                    // script, stop here so the Pending lookahead-merges
                    // into the next run instead.
                    let mut k = i;
                    while k < codepoints.len() && matches!(codepoints[k].2, Cls::Pending) {
                        k += 1;
                    }
                    if let Some(next) = codepoints.get(k) {
                        if let Cls::Concrete(s) = &next.2 {
                            if *s == concrete_script {
                                run_end_byte = next.0 + next.1;
                                i = k + 1;
                                continue;
                            }
                        }
                    }
                    break;
                }
                Cls::Concrete(_) => break,
            }
        }
        runs.push(ScriptRun {
            byte_range: run_start_byte..run_end_byte,
            script: script_full_name(concrete_script),
        });
    }

    // If nothing concrete was ever found, the whole text is Common.
    if runs.is_empty() {
        return vec![ScriptRun {
            byte_range: 0..text.len(),
            script: "Common".to_string(),
        }];
    }

    // Coverage guarantee: extend the last run's end to text.len() so
    // any trailing Pending bytes (handled by the trailing-pending
    // branch above) are covered.
    if let Some(last) = runs.last_mut() {
        if last.byte_range.end < text.len() {
            last.byte_range.end = text.len();
        }
    }

    runs
}

fn script_full_name(s: Script) -> String {
    // unicode-script's Script::full_name() returns the PascalCase
    // Unicode property name. Fall back to Debug if for any reason
    // full_name is unavailable.
    s.full_name().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_text_no_runs() {
        assert!(script_runs("").is_empty());
    }

    #[test]
    fn pure_latin_one_run() {
        let runs = script_runs("Hello");
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].script, "Latin");
        assert_eq!(runs[0].byte_range, 0..5);
    }

    #[test]
    fn mixed_latin_han_two_runs() {
        // "abc 你好" — Latin + Common (space) + Han
        let s = "abc 你好";
        let runs = script_runs(s);
        let scripts: Vec<&str> = runs.iter().map(|r| r.script.as_str()).collect();
        assert!(scripts.contains(&"Latin"));
        assert!(scripts.contains(&"Han"));
    }

    #[test]
    fn pure_common_text_emits_single_common_run() {
        let s = "   ";
        let runs = script_runs(s);
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].script, "Common");
        assert_eq!(runs[0].byte_range, 0..3);
    }

    #[test]
    fn leading_common_lookahead_merges_with_first_concrete() {
        // " Hello" — leading space (Common) merges with following Latin.
        let s = " Hello";
        let runs = script_runs(s);
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].script, "Latin");
        assert_eq!(runs[0].byte_range.start, 0, "lookahead merge from offset 0");
    }

    #[test]
    fn full_byte_range_coverage() {
        let s = "Hello, 你好世界!";
        let runs = script_runs(s);
        // Together cover the full input: first.start == 0, last.end == len.
        assert_eq!(runs.first().unwrap().byte_range.start, 0);
        assert_eq!(runs.last().unwrap().byte_range.end, s.len());
        // Non-overlapping.
        for w in runs.windows(2) {
            assert_eq!(w[0].byte_range.end, w[1].byte_range.start);
        }
    }

    #[test]
    fn full_name_pascalcase_not_short_name() {
        // unicode-script short_name for Han is "Hani"; full_name is "Han".
        // We must use full_name.
        let runs = script_runs("你好");
        assert_eq!(runs[0].script, "Han");
        assert_ne!(runs[0].script, "Hani");
    }
}
// CODEGEN-END
