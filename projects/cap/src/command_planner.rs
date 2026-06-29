// SPEC-MANAGED: projects/cap/tech-design/semantic/cap-src.md#schema
// CODEGEN-BEGIN
#![allow(dead_code)]
//! Same-name command planner for `cap <cmd>`.
//!
//! The planner keeps the public command shape familiar (`cap grep`,
//! `cap ls`, ...), then chooses a faster implementation only for
//! conservative subsets. Unsupported forms fall back to the original command.

use std::{
    env, fs,
    io::{self, BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::ExitCode,
};

use anyhow::{Context, Result};

const SORT_NATIVE_MIN_BYTES: u64 = 1024 * 1024;
// @spec projects/cap/tech-design/logic/add-workload-sensitive-native-command-gates.md#changes
const LS_NATIVE_MIN_ENTRIES: usize = 1024;
const FIND_NATIVE_MIN_ENTRIES: usize = 512;
const SED_NATIVE_MIN_BYTES: u64 = 1024 * 1024;
const SED_NATIVE_MIN_SPAN_LINES: usize = 1024;
const GREP_NATIVE_MIN_FILES: usize = 64;
const GREP_NATIVE_MIN_BYTES: u64 = 1024 * 1024;

/// @spec projects/cap/tech-design/logic/cap-hook-auto-command-optimizer-whitelist.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandPlan {
    External(ExternalPlan),
    Native(NativePlan),
}

/// @spec projects/cap/tech-design/logic/cap-hook-auto-command-optimizer-whitelist.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalPlan {
    pub program: String,
    pub args: Vec<String>,
    pub label: Option<String>,
    pub original: String,
    pub implementation: ExternalImplementation,
    pub reason: String,
    pub fallback: Option<String>,
}

/// @spec projects/cap/tech-design/logic/cap-hook-auto-command-optimizer-whitelist.md#changes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExternalImplementation {
    Original,
    Replacement,
}

/// @spec projects/cap/tech-design/logic/cap-hook-auto-command-optimizer-whitelist.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativePlan {
    pub command: NativeCommand,
    pub label: Option<String>,
    pub original: String,
    pub reason: String,
}

/// @spec projects/cap/tech-design/logic/cap-hook-auto-command-optimizer-whitelist.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NativeCommand {
    Ls(LsPlan),
    Sort(SortPlan),
    Cat(CatPlan),
    Find(FindPlan),
    SedPrint(SedPrintPlan),
}

/// @spec projects/cap/tech-design/logic/cap-hook-auto-command-optimizer-whitelist.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LsPlan {
    pub path: String,
    pub all: bool,
}

/// @spec projects/cap/tech-design/logic/cap-hook-auto-command-optimizer-whitelist.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SortPlan {
    pub file: String,
}

/// @spec projects/cap/tech-design/logic/cap-hook-auto-command-optimizer-whitelist.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatPlan {
    pub files: Vec<String>,
}

/// @spec projects/cap/tech-design/logic/cap-hook-auto-command-optimizer-whitelist.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FindPlan {
    pub root: String,
    pub type_filter: Option<FindType>,
    pub name_pattern: Option<String>,
}

/// @spec projects/cap/tech-design/logic/cap-hook-auto-command-optimizer-whitelist.md#changes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FindType {
    File,
    Dir,
}

/// @spec projects/cap/tech-design/logic/cap-hook-auto-command-optimizer-whitelist.md#changes
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SedPrintPlan {
    pub file: String,
    pub start_line: usize,
    pub end_line: usize,
}

/// @spec projects/cap/tech-design/logic/cap-hook-auto-command-optimizer-whitelist.md#changes
impl CommandPlan {
    pub fn explain(&self) -> String {
        match self {
            CommandPlan::External(plan) => {
                let implementation = match plan.implementation {
                    ExternalImplementation::Original => "original",
                    ExternalImplementation::Replacement => "replacement",
                };
                let mut lines = vec![
                    format!("original: {}", plan.original),
                    format!("implementation: {implementation}"),
                    format!("run: {}", render_command(&plan.program, &plan.args)),
                    format!("reason: {}", plan.reason),
                ];
                if let Some(fallback) = &plan.fallback {
                    lines.push(format!("fallback: {fallback}"));
                }
                lines.join("\n")
            }
            CommandPlan::Native(plan) => {
                let native = match &plan.command {
                    NativeCommand::Ls(_) => "cap-native ls",
                    NativeCommand::Sort(_) => "cap-native sort",
                    NativeCommand::Cat(_) => "cap-native cat",
                    NativeCommand::Find(_) => "cap-native find",
                    NativeCommand::SedPrint(_) => "cap-native sed -n",
                };
                [
                    format!("original: {}", plan.original),
                    "implementation: native".to_string(),
                    format!("run: {native}"),
                    format!("reason: {}", plan.reason),
                ]
                .join("\n")
            }
        }
    }
}

/// @spec projects/cap/tech-design/logic/cap-hook-auto-command-optimizer-whitelist.md#changes
pub fn plan(command: &[String], label: Option<String>) -> CommandPlan {
    plan_with_tool_resolver(command, label, command_on_path)
}

/// Plan one Bash command string. Simple shell-free strings are parsed into argv
/// and routed through the same replacement planner as `cap <cmd>`; strings that
/// need shell semantics stay under `bash -c`.
/// @spec projects/cap/tech-design/logic/cap-hook-auto-command-optimizer-whitelist.md#changes
pub fn plan_shell(command: &str, label: Option<String>) -> CommandPlan {
    let original = command.trim().to_string();
    let planned_label = label.or_else(|| Some(original.clone()));
    if !original.is_empty() && !has_shell_control_syntax(&original) {
        if let Some(words) = split_simple_shell_words(&original) {
            if !words.is_empty() && !words_need_shell(&words) {
                return plan(&words, planned_label);
            }
        }
    }

    CommandPlan::External(ExternalPlan {
        program: "bash".to_string(),
        args: vec!["-c".to_string(), original.clone()],
        label: planned_label,
        original,
        implementation: ExternalImplementation::Original,
        reason: "shell command string requires bash semantics; running under bash -c".to_string(),
        fallback: None,
    })
}

fn plan_with_tool_resolver(
    command: &[String],
    label: Option<String>,
    tool_available: impl Fn(&str) -> bool,
) -> CommandPlan {
    let original = render_argv(command);
    let planned_label = label.or_else(|| Some(original.clone()));

    if let Some(plan) = plan_native(command, planned_label.clone(), &original) {
        return CommandPlan::Native(plan);
    }
    if let Some(plan) = plan_grep_replacement(command, planned_label.clone(), &original, |tool| {
        tool_available(tool)
    }) {
        return CommandPlan::External(plan);
    }

    CommandPlan::External(ExternalPlan {
        program: command[0].clone(),
        args: command[1..].to_vec(),
        label: planned_label,
        original,
        implementation: ExternalImplementation::Original,
        reason: "no safe cap replacement matched; running the original command".to_string(),
        fallback: None,
    })
}

fn plan_native(command: &[String], label: Option<String>, original: &str) -> Option<NativePlan> {
    let program = basename(command.first()?);
    match program {
        "ls" => plan_ls(&command[1..], label, original),
        "cat" => plan_cat(&command[1..], label, original),
        "find" => plan_find(&command[1..], label, original),
        "sort" => plan_sort(&command[1..], label, original),
        "sed" => plan_sed(&command[1..], label, original),
        _ => None,
    }
}

fn plan_ls(args: &[String], label: Option<String>, original: &str) -> Option<NativePlan> {
    let mut all = false;
    let mut paths = Vec::new();

    for arg in args {
        if arg == "--" {
            return None;
        }
        if arg.starts_with('-') && arg.len() > 1 {
            for flag in arg[1..].chars() {
                match flag {
                    'a' | 'A' => all = true,
                    '1' => {}
                    _ => return None,
                }
            }
        } else {
            paths.push(arg.clone());
        }
    }

    if paths.len() > 1 {
        return None;
    }
    let path = paths.pop().unwrap_or_else(|| ".".to_string());
    let path_ref = Path::new(&path);
    if !path_ref.exists() {
        return None;
    }
    if !path_ref.is_dir() || !dir_entries_at_least(path_ref, LS_NATIVE_MIN_ENTRIES, all) {
        return None;
    }

    Some(NativePlan {
        command: NativeCommand::Ls(LsPlan { path, all }),
        label,
        original: original.to_string(),
        reason: "large simple non-long ls can be listed in-process".to_string(),
    })
}

fn plan_sort(args: &[String], label: Option<String>, original: &str) -> Option<NativePlan> {
    if args.len() != 1 {
        return None;
    }
    let meta = fs::metadata(&args[0]).ok()?;
    if !meta.is_file() || meta.len() < SORT_NATIVE_MIN_BYTES {
        return None;
    }

    Some(NativePlan {
        command: NativeCommand::Sort(SortPlan {
            file: args[0].clone(),
        }),
        label,
        original: original.to_string(),
        reason: "large single-file sort can use cap's buffered in-process sorter".to_string(),
    })
}

fn plan_cat(args: &[String], label: Option<String>, original: &str) -> Option<NativePlan> {
    if args.is_empty() || args.iter().any(|arg| arg.starts_with('-')) {
        return None;
    }
    if args.iter().any(|path| !Path::new(path).is_file()) {
        return None;
    }

    Some(NativePlan {
        command: NativeCommand::Cat(CatPlan {
            files: args.to_vec(),
        }),
        label,
        original: original.to_string(),
        reason: "plain cat over regular files can stream in-process".to_string(),
    })
}

fn plan_find(args: &[String], label: Option<String>, original: &str) -> Option<NativePlan> {
    let mut idx = 0;
    let root = if args.first().is_some_and(|arg| !arg.starts_with('-')) {
        idx = 1;
        args[0].clone()
    } else {
        ".".to_string()
    };
    let root_ref = Path::new(&root);
    if !root_ref.exists() || !tree_entries_at_least(root_ref, FIND_NATIVE_MIN_ENTRIES) {
        return None;
    }

    let mut type_filter = None;
    let mut name_pattern = None;
    while idx < args.len() {
        match args[idx].as_str() {
            "-type" => {
                idx += 1;
                let kind = args.get(idx)?;
                type_filter = match kind.as_str() {
                    "f" => Some(FindType::File),
                    "d" => Some(FindType::Dir),
                    _ => return None,
                };
            }
            "-name" => {
                idx += 1;
                let pattern = args.get(idx)?;
                if pattern.contains(['[', ']']) {
                    return None;
                }
                name_pattern = Some(pattern.clone());
            }
            _ => return None,
        }
        idx += 1;
    }

    Some(NativePlan {
        command: NativeCommand::Find(FindPlan {
            root,
            type_filter,
            name_pattern,
        }),
        label,
        original: original.to_string(),
        reason: "large simple find predicates can be walked in-process".to_string(),
    })
}

fn plan_sed(args: &[String], label: Option<String>, original: &str) -> Option<NativePlan> {
    if args.len() != 3 || args[0] != "-n" {
        return None;
    }
    let (start_line, end_line) = parse_sed_print_script(&args[1])?;
    let path = Path::new(&args[2]);
    let meta = fs::metadata(path).ok()?;
    if !meta.is_file()
        || (meta.len() < SED_NATIVE_MIN_BYTES
            && end_line.saturating_sub(start_line) + 1 < SED_NATIVE_MIN_SPAN_LINES)
    {
        return None;
    }

    Some(NativePlan {
        command: NativeCommand::SedPrint(SedPrintPlan {
            file: args[2].clone(),
            start_line,
            end_line,
        }),
        label,
        original: original.to_string(),
        reason: "large sed -n line print can be served as an in-process ranged read".to_string(),
    })
}

fn plan_grep_replacement(
    command: &[String],
    label: Option<String>,
    original: &str,
    tool_available: impl Fn(&str) -> bool,
) -> Option<ExternalPlan> {
    if !tool_available("rg") || command.first().map(|w| basename(w)) != Some("grep") {
        return None;
    }

    let mut recursive = false;
    let mut rg_args = vec!["--hidden".to_string(), "--no-ignore".to_string()];
    let mut positional = Vec::new();
    let mut flags_done = false;

    for word in command.iter().skip(1) {
        if !flags_done && word == "--" {
            flags_done = true;
            continue;
        }
        if !flags_done && word.starts_with("--") {
            match word.as_str() {
                "--recursive" | "--dereference-recursive" => recursive = true,
                "--line-number" => rg_args.push("--line-number".to_string()),
                "--ignore-case" => rg_args.push("--ignore-case".to_string()),
                "--fixed-strings" => rg_args.push("--fixed-strings".to_string()),
                "--word-regexp" => rg_args.push("--word-regexp".to_string()),
                _ => return None,
            }
            continue;
        }
        if !flags_done && word.starts_with('-') && word.len() > 1 {
            for flag in word[1..].chars() {
                match flag {
                    'R' | 'r' => recursive = true,
                    'n' => rg_args.push("-n".to_string()),
                    'i' => rg_args.push("-i".to_string()),
                    'F' => rg_args.push("-F".to_string()),
                    'w' => rg_args.push("-w".to_string()),
                    'H' => rg_args.push("-H".to_string()),
                    'l' => rg_args.push("-l".to_string()),
                    'q' => rg_args.push("-q".to_string()),
                    _ => return None,
                }
            }
            continue;
        }
        positional.push(word.clone());
    }

    if !recursive || positional.len() < 2 {
        return None;
    }
    if positional
        .iter()
        .any(|arg| arg.is_empty() || arg.contains(['*', '?', '[', ']']))
    {
        return None;
    }
    if positional.iter().skip(1).any(|path| path.starts_with('-')) {
        return None;
    }
    let large_enough = positional.iter().skip(1).any(|path| {
        grep_workload_at_least(
            Path::new(path),
            GREP_NATIVE_MIN_FILES,
            GREP_NATIVE_MIN_BYTES,
        )
    });
    if !large_enough {
        return None;
    }

    rg_args.push("--".to_string());
    rg_args.extend(positional);

    let optimized = render_command("rg", &rg_args);
    let fallback = render_argv(command);
    let script = format!("{optimized} || {fallback}");

    Some(ExternalPlan {
        program: "bash".to_string(),
        args: vec!["-c".to_string(), script],
        label,
        original: original.to_string(),
        implementation: ExternalImplementation::Replacement,
        reason: "recursive grep safe subset can use rg with original-command fallback".to_string(),
        fallback: Some(fallback),
    })
}

/// @spec projects/cap/tech-design/logic/add-workload-sensitive-native-command-gates.md#changes
fn dir_entries_at_least(path: &Path, min: usize, include_hidden: bool) -> bool {
    let Ok(entries) = fs::read_dir(path) else {
        return false;
    };
    let mut count = 0usize;
    for entry in entries.flatten() {
        let name = entry.file_name();
        if !include_hidden && name.to_string_lossy().starts_with('.') {
            continue;
        }
        count += 1;
        if count >= min {
            return true;
        }
    }
    false
}

/// @spec projects/cap/tech-design/logic/add-workload-sensitive-native-command-gates.md#changes
fn tree_entries_at_least(root: &Path, min: usize) -> bool {
    let mut count = 0usize;
    let mut stack = vec![root.to_path_buf()];
    while let Some(path) = stack.pop() {
        let Ok(meta) = fs::symlink_metadata(&path) else {
            continue;
        };
        if !path.as_os_str().is_empty() {
            count += 1;
            if count >= min {
                return true;
            }
        }
        if meta.file_type().is_dir() {
            let Ok(entries) = fs::read_dir(&path) else {
                continue;
            };
            for entry in entries.flatten() {
                stack.push(entry.path());
            }
        }
    }
    false
}

/// @spec projects/cap/tech-design/logic/add-workload-sensitive-native-command-gates.md#changes
fn grep_workload_at_least(root: &Path, min_files: usize, min_bytes: u64) -> bool {
    let mut files = 0usize;
    let mut bytes = 0u64;
    let mut stack = vec![root.to_path_buf()];
    while let Some(path) = stack.pop() {
        let Ok(meta) = fs::symlink_metadata(&path) else {
            continue;
        };
        if meta.file_type().is_file() {
            files += 1;
            bytes = bytes.saturating_add(meta.len());
            if files >= min_files || bytes >= min_bytes {
                return true;
            }
        } else if meta.file_type().is_dir() {
            let Ok(entries) = fs::read_dir(&path) else {
                continue;
            };
            for entry in entries.flatten() {
                stack.push(entry.path());
            }
        }
    }
    false
}

/// @spec projects/cap/tech-design/logic/cap-hook-auto-command-optimizer-whitelist.md#changes
pub fn run_native(plan: &NativePlan) -> Result<ExitCode> {
    let mut stdout = io::stdout().lock();
    let mut stderr = io::stderr().lock();
    let code = run_native_to(plan, &mut stdout, &mut stderr)?;
    Ok(exit_code_from_i32(code))
}

pub(crate) fn run_native_to(
    plan: &NativePlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<i32> {
    match &plan.command {
        NativeCommand::Ls(ls) => run_ls(ls, stdout, stderr),
        NativeCommand::Sort(sort) => run_sort(sort, stdout),
        NativeCommand::Cat(cat) => run_cat(cat, stdout, stderr),
        NativeCommand::Find(find) => run_find(find, stdout, stderr),
        NativeCommand::SedPrint(sed) => run_sed_print(sed, stdout, stderr),
    }
}

fn run_ls(plan: &LsPlan, stdout: &mut dyn Write, stderr: &mut dyn Write) -> Result<i32> {
    let path = Path::new(&plan.path);
    if path.is_dir() {
        let mut names = Vec::new();
        if plan.all {
            names.push(".".to_string());
            names.push("..".to_string());
        }
        for entry in fs::read_dir(path).with_context(|| format!("reading {}", plan.path))? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();
            if plan.all || !name.starts_with('.') {
                names.push(name);
            }
        }
        names.sort();
        for name in names {
            writeln!(stdout, "{name}")?;
        }
        return Ok(0);
    }

    if path.exists() {
        writeln!(stdout, "{}", plan.path)?;
        Ok(0)
    } else {
        writeln!(stderr, "ls: {}: No such file or directory", plan.path)?;
        Ok(1)
    }
}

fn run_sort(plan: &SortPlan, stdout: &mut dyn Write) -> Result<i32> {
    let data = fs::read(&plan.file).with_context(|| format!("reading {}", plan.file))?;
    let mut lines = Vec::new();
    let mut start = 0;
    while start < data.len() {
        let mut end = start;
        while end < data.len() && data[end] != b'\n' {
            end += 1;
        }
        let next = if end < data.len() { end + 1 } else { end };
        lines.push((start, next));
        start = next;
    }

    let mut ascending = true;
    let mut descending = true;
    for pair in lines.windows(2) {
        let previous = &data[pair[0].0..pair[0].1];
        let current = &data[pair[1].0..pair[1].1];
        if previous > current {
            ascending = false;
        }
        if previous < current {
            descending = false;
        }
    }

    if descending && !ascending {
        lines.reverse();
    } else if !ascending {
        lines.sort_unstable_by(|left, right| data[left.0..left.1].cmp(&data[right.0..right.1]));
    }

    let mut buffered = io::BufWriter::new(stdout);
    for (start, end) in lines {
        let line = &data[start..end];
        buffered.write_all(line)?;
        if !line.ends_with(b"\n") {
            buffered.write_all(b"\n")?;
        }
    }
    Ok(0)
}

fn run_cat(plan: &CatPlan, stdout: &mut dyn Write, stderr: &mut dyn Write) -> Result<i32> {
    let mut exit = 0;
    for file in &plan.files {
        match fs::File::open(file) {
            Ok(mut f) => {
                io::copy(&mut f, stdout)?;
            }
            Err(e) => {
                writeln!(stderr, "cat: {file}: {e}")?;
                exit = 1;
            }
        }
    }
    Ok(exit)
}

fn run_find(plan: &FindPlan, stdout: &mut dyn Write, stderr: &mut dyn Write) -> Result<i32> {
    let mut exit = 0;
    visit_find(PathBuf::from(&plan.root), plan, stdout, stderr, &mut exit)?;
    Ok(exit)
}

fn visit_find(
    path: PathBuf,
    plan: &FindPlan,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
    exit: &mut i32,
) -> Result<()> {
    let meta = match fs::symlink_metadata(&path) {
        Ok(meta) => meta,
        Err(e) => {
            writeln!(stderr, "find: {}: {e}", path.display())?;
            *exit = 1;
            return Ok(());
        }
    };

    if find_path_matches(&path, &meta, plan) {
        writeln!(stdout, "{}", path.display())?;
    }

    if meta.file_type().is_dir() {
        let mut children = Vec::new();
        match fs::read_dir(&path) {
            Ok(entries) => {
                for entry in entries {
                    match entry {
                        Ok(entry) => children.push(entry.path()),
                        Err(e) => {
                            writeln!(stderr, "find: {}: {e}", path.display())?;
                            *exit = 1;
                        }
                    }
                }
            }
            Err(e) => {
                writeln!(stderr, "find: {}: {e}", path.display())?;
                *exit = 1;
            }
        }
        children.sort();
        for child in children {
            visit_find(child, plan, stdout, stderr, exit)?;
        }
    }

    Ok(())
}

fn find_path_matches(path: &Path, meta: &fs::Metadata, plan: &FindPlan) -> bool {
    let type_ok = match plan.type_filter {
        Some(FindType::File) => meta.file_type().is_file(),
        Some(FindType::Dir) => meta.file_type().is_dir(),
        None => true,
    };
    let name_ok = match &plan.name_pattern {
        Some(pattern) => path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| glob_match(pattern, name)),
        None => true,
    };
    type_ok && name_ok
}

fn run_sed_print(
    plan: &SedPrintPlan,
    stdout: &mut dyn Write,
    _stderr: &mut dyn Write,
) -> Result<i32> {
    let file = fs::File::open(&plan.file).with_context(|| format!("reading {}", plan.file))?;
    let reader = BufReader::new(file);
    for (idx, line) in reader.lines().enumerate() {
        let line_no = idx + 1;
        let line = line?;
        if line_no >= plan.start_line && line_no <= plan.end_line {
            writeln!(stdout, "{line}")?;
        }
        if line_no > plan.end_line {
            break;
        }
    }
    Ok(0)
}

fn parse_sed_print_script(script: &str) -> Option<(usize, usize)> {
    let body = script.strip_suffix('p')?;
    if body.is_empty() {
        return None;
    }
    let (start, end) = if let Some((start, end)) = body.split_once(',') {
        (start.parse().ok()?, end.parse().ok()?)
    } else {
        let line = body.parse().ok()?;
        (line, line)
    };
    if start == 0 || end < start {
        return None;
    }
    Some((start, end))
}

fn glob_match(pattern: &str, text: &str) -> bool {
    fn inner(p: &[u8], t: &[u8]) -> bool {
        match p.split_first() {
            None => t.is_empty(),
            Some((&b'*', rest)) => inner(rest, t) || (!t.is_empty() && inner(p, &t[1..])),
            Some((&b'?', rest)) => !t.is_empty() && inner(rest, &t[1..]),
            Some((&pc, rest)) => t.first().is_some_and(|tc| *tc == pc) && inner(rest, &t[1..]),
        }
    }
    inner(pattern.as_bytes(), text.as_bytes())
}

/// @spec projects/cap/tech-design/logic/cap-hook-auto-command-optimizer-whitelist.md#changes
pub fn render_argv(command: &[String]) -> String {
    match command.split_first() {
        Some((program, args)) => render_command(program, args),
        None => String::new(),
    }
}

/// @spec projects/cap/tech-design/logic/cap-hook-auto-command-optimizer-whitelist.md#changes
pub fn render_command(program: &str, args: &[String]) -> String {
    std::iter::once(program.to_string())
        .chain(args.iter().map(|arg| shell_quote_arg(arg)))
        .collect::<Vec<_>>()
        .join(" ")
}

fn shell_quote_arg(s: &str) -> String {
    let safe = !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '/' | '.' | '_' | '-' | ':'));
    if safe {
        s.to_string()
    } else {
        shell_single_quote(s)
    }
}

fn shell_single_quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('\'');
    for ch in s.chars() {
        if ch == '\'' {
            out.push_str("'\\''");
        } else {
            out.push(ch);
        }
    }
    out.push('\'');
    out
}

fn command_on_path(program: &str) -> bool {
    let Some(paths) = env::var_os("PATH") else {
        return false;
    };
    env::split_paths(&paths).any(|dir| dir.join(program).is_file())
}

fn basename(p: &str) -> &str {
    p.rsplit('/').next().unwrap_or(p)
}

fn has_shell_control_syntax(command: &str) -> bool {
    #[derive(Clone, Copy)]
    enum State {
        Normal,
        Single,
        Double,
    }

    let mut chars = command.chars();
    let mut state = State::Normal;
    while let Some(ch) = chars.next() {
        match state {
            State::Normal => match ch {
                '\'' => state = State::Single,
                '"' => state = State::Double,
                '\\' => {
                    if chars.next().is_none() {
                        return true;
                    }
                }
                '\n' | '\r' | '|' | '&' | ';' | '<' | '>' | '`' | '$' | '*' | '?' | '[' | ']'
                | '{' | '}' | '~' | '(' | ')' => return true,
                _ => {}
            },
            State::Single => {
                if ch == '\'' {
                    state = State::Normal;
                }
            }
            State::Double => match ch {
                '"' => state = State::Normal,
                '\\' => {
                    if chars.next().is_none() {
                        return true;
                    }
                }
                '`' | '$' => return true,
                _ => {}
            },
        }
    }

    !matches!(state, State::Normal)
}

fn split_simple_shell_words(command: &str) -> Option<Vec<String>> {
    #[derive(Clone, Copy)]
    enum State {
        Normal,
        Single,
        Double,
    }

    let mut words = Vec::new();
    let mut current = String::new();
    let mut chars = command.chars();
    let mut state = State::Normal;
    let mut in_token = false;

    while let Some(ch) = chars.next() {
        match state {
            State::Normal => match ch {
                '\'' => {
                    in_token = true;
                    state = State::Single;
                }
                '"' => {
                    in_token = true;
                    state = State::Double;
                }
                '\\' => {
                    in_token = true;
                    current.push(chars.next()?);
                }
                c if c.is_whitespace() => {
                    if in_token {
                        words.push(std::mem::take(&mut current));
                        in_token = false;
                    }
                }
                c => {
                    in_token = true;
                    current.push(c);
                }
            },
            State::Single => match ch {
                '\'' => state = State::Normal,
                c => current.push(c),
            },
            State::Double => match ch {
                '"' => state = State::Normal,
                '\\' => current.push(chars.next()?),
                c => current.push(c),
            },
        }
    }

    match state {
        State::Normal => {
            if in_token {
                words.push(current);
            }
            Some(words)
        }
        State::Single | State::Double => None,
    }
}

fn words_need_shell(words: &[String]) -> bool {
    let first = basename(&words[0]);
    first_word_needs_shell(first) || is_var_assignment(&words[0])
}

fn first_word_needs_shell(first: &str) -> bool {
    matches!(
        first,
        "alias"
            | "bg"
            | "break"
            | "cd"
            | "continue"
            | "eval"
            | "exec"
            | "export"
            | "fc"
            | "fg"
            | "jobs"
            | "read"
            | "readonly"
            | "return"
            | "set"
            | "shift"
            | "source"
            | "times"
            | "trap"
            | "type"
            | "typeset"
            | "ulimit"
            | "umask"
            | "unalias"
            | "unset"
            | "."
    )
}

fn is_var_assignment(tok: &str) -> bool {
    let Some(eq) = tok.find('=') else {
        return false;
    };
    let name = &tok[..eq];
    if name.is_empty() {
        return false;
    }
    let mut chars = name.chars();
    let first = chars.next().unwrap();
    if !(first.is_ascii_alphabetic() || first == '_') {
        return false;
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn exit_code_from_i32(code: i32) -> ExitCode {
    if (0..=255).contains(&code) {
        ExitCode::from(code as u8)
    } else {
        ExitCode::FAILURE
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn s(args: &[&str]) -> Vec<String> {
        args.iter().map(|arg| arg.to_string()).collect()
    }

    fn plan_without_tools(args: &[&str]) -> CommandPlan {
        plan_with_tool_resolver(&s(args), None, |_| false)
    }

    #[test]
    fn shell_string_simple_commands_use_cap_planner() {
        let tmp = tempdir().unwrap();
        for idx in 0..FIND_NATIVE_MIN_ENTRIES {
            fs::write(tmp.path().join(format!("file-{idx:04}.txt")), "").unwrap();
        }
        assert!(matches!(
            plan_shell(&format!("find {} -type f", tmp.path().display()), None),
            CommandPlan::Native(NativePlan {
                command: NativeCommand::Find(_),
                ..
            })
        ));
    }

    #[test]
    fn shell_string_pipes_keep_bash_semantics() {
        let CommandPlan::External(plan) = plan_shell("find . -type f | xargs wc -l", None) else {
            panic!("expected bash fallback");
        };
        assert_eq!(plan.program, "bash");
        assert_eq!(plan.args, vec!["-c", "find . -type f | xargs wc -l"]);
        assert_eq!(
            plan.reason,
            "shell command string requires bash semantics; running under bash -c"
        );
    }

    #[test]
    fn grep_falls_back_until_resource_gate_wins() {
        let tmp = tempdir().unwrap();
        let CommandPlan::External(no_rg) = plan_with_tool_resolver(
            &s(&["grep", "-R", "TODO", tmp.path().to_str().unwrap()]),
            None,
            |tool| tool == "rg",
        ) else {
            panic!("expected original fallback");
        };
        assert_eq!(no_rg.implementation, ExternalImplementation::Original);
    }

    #[test]
    fn only_resource_winning_commands_plan_native() {
        let tmp = tempdir().unwrap();
        let file = tmp.path().join("a.txt");
        fs::write(&file, "one\ntwo\nthree\n").unwrap();
        let list_dir = tmp.path().join("list-large");
        fs::create_dir(&list_dir).unwrap();
        for idx in 0..LS_NATIVE_MIN_ENTRIES {
            fs::write(list_dir.join(format!("entry-{idx:04}")), "").unwrap();
        }
        let find_dir = tmp.path().join("find-large");
        fs::create_dir(&find_dir).unwrap();
        for idx in 0..FIND_NATIVE_MIN_ENTRIES {
            fs::write(find_dir.join(format!("file-{idx:04}.txt")), "").unwrap();
        }
        let sort_file = tmp.path().join("sort-large.txt");
        fs::write(
            &sort_file,
            "z\na\n".repeat((SORT_NATIVE_MIN_BYTES as usize / 4) + 1),
        )
        .unwrap();
        let sed_file = tmp.path().join("sed-large.txt");
        fs::write(&sed_file, "line\n".repeat(SED_NATIVE_MIN_SPAN_LINES + 1)).unwrap();

        assert!(matches!(
            plan_without_tools(&["ls", list_dir.to_str().unwrap()]),
            CommandPlan::Native(NativePlan {
                command: NativeCommand::Ls(_),
                ..
            })
        ));

        assert!(matches!(
            plan_without_tools(&["sort", sort_file.to_str().unwrap()]),
            CommandPlan::Native(NativePlan {
                command: NativeCommand::Sort(_),
                ..
            })
        ));

        assert!(matches!(
            plan_without_tools(&["cat", file.to_str().unwrap()]),
            CommandPlan::Native(NativePlan {
                command: NativeCommand::Cat(_),
                ..
            })
        ));
        assert!(matches!(
            plan_without_tools(&["find", find_dir.to_str().unwrap(), "-type", "f"]),
            CommandPlan::Native(NativePlan {
                command: NativeCommand::Find(_),
                ..
            })
        ));
        assert!(matches!(
            plan_without_tools(&[
                "sed",
                "-n",
                &format!("1,{SED_NATIVE_MIN_SPAN_LINES}p"),
                sed_file.to_str().unwrap()
            ]),
            CommandPlan::Native(NativePlan {
                command: NativeCommand::SedPrint(_),
                ..
            })
        ));

        let CommandPlan::External(plan) = plan_without_tools(&["sort", file.to_str().unwrap()])
        else {
            panic!("expected original fallback for small sort input");
        };
        assert_eq!(plan.implementation, ExternalImplementation::Original);
    }

    #[test]
    fn tiny_workloads_keep_original_path() {
        let tmp = tempdir().unwrap();
        let list_dir = tmp.path().join("list-small");
        fs::create_dir(&list_dir).unwrap();
        fs::write(list_dir.join("one"), "").unwrap();
        let sed_file = tmp.path().join("sed-small.txt");
        fs::write(&sed_file, "one\ntwo\nthree\n").unwrap();

        for args in [
            vec!["ls", list_dir.to_str().unwrap()],
            vec!["find", tmp.path().to_str().unwrap(), "-type", "f"],
            vec!["sed", "-n", "1,2p", sed_file.to_str().unwrap()],
        ] {
            let CommandPlan::External(plan) = plan_without_tools(&args) else {
                panic!("expected original fallback for {args:?}");
            };
            assert_eq!(plan.implementation, ExternalImplementation::Original);
        }
    }

    #[test]
    fn large_recursive_grep_can_use_replacement_when_rg_exists() {
        let tmp = tempdir().unwrap();
        for idx in 0..GREP_NATIVE_MIN_FILES {
            fs::write(tmp.path().join(format!("file-{idx:04}.txt")), "TODO\n").unwrap();
        }

        let CommandPlan::External(plan) = plan_with_tool_resolver(
            &s(&["grep", "-R", "TODO", tmp.path().to_str().unwrap()]),
            None,
            |tool| tool == "rg",
        ) else {
            panic!("expected grep replacement");
        };
        assert_eq!(plan.implementation, ExternalImplementation::Replacement);
    }
}
// CODEGEN-END
