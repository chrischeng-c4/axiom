// SPEC-MANAGED: apps/cap/tech-design/semantic/source/projects-cap-tests-behavior-cap-command-replacement-parity-rs.md#rust-source-unit
// CODEGEN-BEGIN
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use anyhow::{bail, Context, Result};

#[test]
fn active_replacements_match_success_and_error_behavior() -> Result<()> {
    let temp = tempfile::tempdir().context("create parity tempdir")?;
    let bin_dir = temp.path().join("bin");
    fs::create_dir(&bin_dir)?;
    let cap = build_cap_frontend(&bin_dir)?;
    let fixture = Fixture::create(temp.path())?;
    let quiet_missing = temp.path().join("quiet-missing").display().to_string();
    let hostname_output = run(Path::new("/bin/hostname"), &[])?;
    let hostname_text = String::from_utf8_lossy(&hostname_output.stdout);
    let hostname_pattern = hostname_text
        .chars()
        .find(|ch| ch.is_ascii_alphanumeric())
        .map(|ch| ch.to_string())
        .context("hostname output lacks a grep-safe pattern")?;

    let success_cases = [
        Case::new("true", vec!["true"], "/usr/bin/true", vec![]),
        Case::new("false", vec!["false"], "/usr/bin/false", vec![]),
        Case::new("pwd", vec!["pwd"], "/bin/pwd", vec![]),
        Case::new(
            "echo",
            vec!["echo", "alpha", "beta"],
            "/bin/echo",
            vec!["alpha", "beta"],
        ),
        Case::new(
            "echo-n",
            vec!["echo", "-n", "alpha", "beta"],
            "/bin/echo",
            vec!["-n", "alpha", "beta"],
        ),
        Case::new(
            "printf",
            vec!["printf", "%s\\n", "alpha", "beta"],
            "/usr/bin/printf",
            vec!["%s\\n", "alpha", "beta"],
        ),
        Case::new(
            "printf-join",
            vec!["printf", "%s", "alpha", "beta"],
            "/usr/bin/printf",
            vec!["%s", "alpha", "beta"],
        ),
        Case::new(
            "printf-literal",
            vec!["printf", "alpha\\nbeta\\n"],
            "/usr/bin/printf",
            vec!["alpha\\nbeta\\n"],
        ),
        Case::new("seq", vec!["seq", "1", "5"], "/usr/bin/seq", vec!["1", "5"]),
        Case::new(
            "seq-desc",
            vec!["seq", "5", "-2", "1"],
            "/usr/bin/seq",
            vec!["5", "-2", "1"],
        ),
        Case::new("whoami", vec!["whoami"], "/usr/bin/whoami", vec![]),
        Case::new("id", vec!["id"], "/usr/bin/id", vec![]),
        Case::new("id-u", vec!["id", "-u"], "/usr/bin/id", vec!["-u"]),
        Case::new("id-un", vec!["id", "-un"], "/usr/bin/id", vec!["-un"]),
        Case::new("id-g", vec!["id", "-g"], "/usr/bin/id", vec!["-g"]),
        Case::new("id-gn", vec!["id", "-gn"], "/usr/bin/id", vec!["-gn"]),
        Case::new("id-G", vec!["id", "-G"], "/usr/bin/id", vec!["-G"]),
        Case::new("id-Gn", vec!["id", "-Gn"], "/usr/bin/id", vec!["-Gn"]),
        Case::new("uname", vec!["uname"], "/usr/bin/uname", vec![]),
        Case::new("uname-a", vec!["uname", "-a"], "/usr/bin/uname", vec!["-a"]),
        Case::new("uname-m", vec!["uname", "-m"], "/usr/bin/uname", vec!["-m"]),
        Case::new("uname-p", vec!["uname", "-p"], "/usr/bin/uname", vec!["-p"]),
        Case::new("hostname", vec!["hostname"], "/bin/hostname", vec![]),
        Case::new(
            "test-file",
            vec!["test", "-f", fixture.cat_file()],
            "/bin/test",
            vec!["-f", fixture.cat_file()],
        ),
        Case::new(
            "test-string-eq",
            vec!["test", "alpha", "=", "alpha"],
            "/bin/test",
            vec!["alpha", "=", "alpha"],
        ),
        Case::new(
            "test-int-gt",
            vec!["test", "5", "-gt", "3"],
            "/bin/test",
            vec!["5", "-gt", "3"],
        ),
        Case::new(
            "test-negated-missing",
            vec!["test", "!", "-e", quiet_missing.as_str()],
            "/bin/test",
            vec!["!", "-e", quiet_missing.as_str()],
        ),
        Case::new(
            "bracket-dir",
            vec!["[", "-d", fixture.find_root(), "]"],
            "/bin/[",
            vec!["-d", fixture.find_root(), "]"],
        ),
        Case::new(
            "basename",
            vec!["basename", fixture.basename_path(), ".txt"],
            "/usr/bin/basename",
            vec![fixture.basename_path(), ".txt"],
        ),
        Case::new(
            "dirname",
            vec!["dirname", fixture.basename_path()],
            "/usr/bin/dirname",
            vec![fixture.basename_path()],
        ),
        Case::new(
            "ls",
            vec!["ls", "-1", fixture.list_dir()],
            "/bin/ls",
            vec!["-1", fixture.list_dir()],
        ),
        Case::new(
            "ls-almost-all",
            vec!["ls", "-A", fixture.list_dir()],
            "/bin/ls",
            vec!["-A", fixture.list_dir()],
        ),
        Case::new(
            "cat",
            vec!["cat", fixture.cat_file()],
            "/bin/cat",
            vec![fixture.cat_file()],
        ),
        Case::new(
            "head",
            vec!["head", "-n", "3", fixture.window_file()],
            "/usr/bin/head",
            vec!["-n", "3", fixture.window_file()],
        ),
        Case::new(
            "tail",
            vec!["tail", "-c", "17", fixture.window_file()],
            "/usr/bin/tail",
            vec!["-c", "17", fixture.window_file()],
        ),
        Case::new(
            "mkdir",
            vec!["mkdir", "-p", fixture.mkdir_existing()],
            "/bin/mkdir",
            vec!["-p", fixture.mkdir_existing()],
        ),
        Case::new(
            "touch",
            vec!["touch", fixture.touch_file()],
            "/usr/bin/touch",
            vec![fixture.touch_file()],
        ),
        Case::new(
            "touch-dir",
            vec!["touch", fixture.touch_dir()],
            "/usr/bin/touch",
            vec![fixture.touch_dir()],
        ),
        Case::new(
            "uniq",
            vec!["uniq", fixture.uniq_file()],
            "/usr/bin/uniq",
            vec![fixture.uniq_file()],
        ),
        Case::new(
            "find",
            vec!["find", fixture.find_root(), "-type", "f", "-name", "*.txt"],
            "/usr/bin/find",
            vec![fixture.find_root(), "-type", "f", "-name", "*.txt"],
        ),
        Case::new(
            "du",
            vec!["du", "-sk", fixture.du_root()],
            "/usr/bin/du",
            vec!["-sk", fixture.du_root()],
        ),
        Case::new(
            "sort",
            vec!["sort", fixture.sort_file()],
            "/usr/bin/sort",
            vec![fixture.sort_file()],
        ),
        Case::new(
            "cut",
            vec!["cut", "-d", ",", "-f", "1", fixture.cut_file()],
            "/usr/bin/cut",
            vec!["-d", ",", "-f", "1", fixture.cut_file()],
        ),
        Case::new(
            "cut-combined",
            vec!["cut", "-d,", "-f2", fixture.cut_file()],
            "/usr/bin/cut",
            vec!["-d,", "-f2", fixture.cut_file()],
        ),
        Case::new(
            "sed",
            vec!["sed", "-n", "1,1024p", fixture.sed_file()],
            "/usr/bin/sed",
            vec!["-n", "1,1024p", fixture.sed_file()],
        ),
        Case::new(
            "grep",
            vec!["grep", "-R", "NEEDLE", fixture.grep_root()],
            "/usr/bin/grep",
            vec!["-R", "NEEDLE", fixture.grep_root()],
        ),
        Case::new(
            "grep-file",
            vec!["grep", "NEEDLE", fixture.grep_file()],
            "/usr/bin/grep",
            vec!["NEEDLE", fixture.grep_file()],
        ),
        Case::new(
            "awk",
            vec![
                "awk",
                "/NEEDLE/ { c++ } END { print c }",
                fixture.sed_file(),
            ],
            "/usr/bin/awk",
            vec!["/NEEDLE/ { c++ } END { print c }", fixture.sed_file()],
        ),
        Case::new(
            "awk-first-field",
            vec!["awk", "{ print $1 }", fixture.sed_file()],
            "/usr/bin/awk",
            vec!["{ print $1 }", fixture.sed_file()],
        ),
        Case::new(
            "awk-second-field",
            vec!["awk", "{ print $2 }", fixture.sed_file()],
            "/usr/bin/awk",
            vec!["{ print $2 }", fixture.sed_file()],
        ),
        Case::new(
            "awk-first-field-compact",
            vec!["awk", "{print$1}", fixture.sed_file()],
            "/usr/bin/awk",
            vec!["{print$1}", fixture.sed_file()],
        ),
        Case::new(
            "awk-first-field-predicate",
            vec!["awk", "/NEEDLE/ { print $1 }", fixture.grep_file()],
            "/usr/bin/awk",
            vec!["/NEEDLE/ { print $1 }", fixture.grep_file()],
        ),
        Case::new("which", vec!["which", "sh"], "/usr/bin/which", vec!["sh"]),
        Case::new(
            "which-all",
            vec!["which", "-a", "sh", "echo"],
            "/usr/bin/which",
            vec!["-a", "sh", "echo"],
        ),
        Case::new(
            "which-builtin",
            vec!["which", "echo"],
            "/usr/bin/which",
            vec!["echo"],
        ),
        Case::new("env", vec!["env"], "/usr/bin/env", vec![]),
        Case::new("printenv", vec!["printenv"], "/usr/bin/printenv", vec![]),
        Case::new(
            "printenv-path",
            vec!["printenv", "PATH"],
            "/usr/bin/printenv",
            vec!["PATH"],
        ),
    ];

    for case in success_cases {
        assert_success_parity(&cap, &case)?;
    }
    let mut wc_cap_args = vec!["wc", "-l"];
    wc_cap_args.extend(fixture.wc_files().iter().map(String::as_str));
    let mut wc_original_args = vec!["-l"];
    wc_original_args.extend(fixture.wc_files().iter().map(String::as_str));
    let wc_case = Case::new("wc", wc_cap_args, "/usr/bin/wc", wc_original_args);
    assert_success_parity(&cap, &wc_case)?;
    let mut wc_all_cap_args = vec!["wc"];
    wc_all_cap_args.extend(fixture.wc_files().iter().map(String::as_str));
    let wc_all_original_args = fixture
        .wc_files()
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let wc_all_case = Case::new(
        "wc-all",
        wc_all_cap_args,
        "/usr/bin/wc",
        wc_all_original_args,
    );
    assert_success_parity(&cap, &wc_all_case)?;
    let mut wc_bytes_cap_args = vec!["wc", "-c"];
    wc_bytes_cap_args.extend(fixture.wc_files().iter().map(String::as_str));
    let mut wc_bytes_original_args = vec!["-c"];
    wc_bytes_original_args.extend(fixture.wc_files().iter().map(String::as_str));
    let wc_bytes_case = Case::new(
        "wc-bytes",
        wc_bytes_cap_args,
        "/usr/bin/wc",
        wc_bytes_original_args,
    );
    assert_success_parity(&cap, &wc_bytes_case)?;
    let mut wc_words_cap_args = vec!["wc", "-w"];
    wc_words_cap_args.extend(fixture.wc_files().iter().map(String::as_str));
    let mut wc_words_original_args = vec!["-w"];
    wc_words_original_args.extend(fixture.wc_files().iter().map(String::as_str));
    let wc_words_case = Case::new(
        "wc-words",
        wc_words_cap_args,
        "/usr/bin/wc",
        wc_words_original_args,
    );
    assert_success_parity(&cap, &wc_words_case)?;
    assert_stdin_success_parity(
        &cap,
        "wc-stdin-lines",
        &["wc", "-l"],
        "/usr/bin/wc",
        &["-l"],
        b"one two\nthree\n",
    )?;
    assert_stdin_success_parity(
        &cap,
        "wc-stdin-bytes",
        &["wc", "-c"],
        "/usr/bin/wc",
        &["-c"],
        b"one two\nthree\n",
    )?;
    assert_stdin_success_parity(
        &cap,
        "wc-stdin-words",
        &["wc", "-w"],
        "/usr/bin/wc",
        &["-w"],
        b"one two\nthree\n",
    )?;
    assert_stdin_success_parity(
        &cap,
        "wc-stdin-all",
        &["wc"],
        "/usr/bin/wc",
        &[],
        b"one two\nthree\n",
    )?;
    assert_stdin_success_parity(
        &cap,
        "xargs-default",
        &["xargs"],
        "/usr/bin/xargs",
        &[],
        b"one\ntwo three\n",
    )?;
    assert_stdin_success_parity(
        &cap,
        "xargs",
        &["xargs", "echo"],
        "/usr/bin/xargs",
        &["echo"],
        b"one\ntwo three\n",
    )?;
    assert_stdin_success_parity(
        &cap,
        "xargs-n1-default",
        &["xargs", "-n", "1"],
        "/usr/bin/xargs",
        &["-n", "1"],
        b"one\ntwo three\n",
    )?;
    assert_stdin_success_parity(
        &cap,
        "xargs-n1-echo",
        &["xargs", "-n", "1", "echo"],
        "/usr/bin/xargs",
        &["-n", "1", "echo"],
        b"one\ntwo three\n",
    )?;
    assert_stdin_success_parity(
        &cap,
        "xargs-n1-compact-echo",
        &["xargs", "-n1", "echo"],
        "/usr/bin/xargs",
        &["-n1", "echo"],
        b"one\ntwo three\n",
    )?;
    assert_stdin_success_parity(
        &cap,
        "xargs-n2-echo",
        &["xargs", "-n", "2", "echo"],
        "/usr/bin/xargs",
        &["-n", "2", "echo"],
        b"one\ntwo three\nfour\n",
    )?;
    assert_stdin_success_parity(
        &cap,
        "xargs-n2-compact-echo",
        &["xargs", "-n2", "echo"],
        "/usr/bin/xargs",
        &["-n2", "echo"],
        b"one\ntwo three\nfour\n",
    )?;
    let xargs_wc_input = format!("{}\n{}\n", fixture.wc_files()[0], fixture.wc_files()[1]);
    assert_stdin_success_parity(
        &cap,
        "xargs-wc",
        &["xargs", "wc", "-l"],
        "/usr/bin/xargs",
        &["wc", "-l"],
        xargs_wc_input.as_bytes(),
    )?;
    assert_stdin_success_parity(
        &cap,
        "tr",
        &["tr", "a-z", "A-Z"],
        "/usr/bin/tr",
        &["a-z", "A-Z"],
        b"alpha\nBeta\n",
    )?;
    assert_stdin_success_parity(
        &cap,
        "tr-class-upper",
        &["tr", "[:lower:]", "[:upper:]"],
        "/usr/bin/tr",
        &["[:lower:]", "[:upper:]"],
        b"alpha\nBeta\n",
    )?;
    assert_stdin_success_parity(
        &cap,
        "tr-class-lower",
        &["tr", "[:upper:]", "[:lower:]"],
        "/usr/bin/tr",
        &["[:upper:]", "[:lower:]"],
        b"alpha\nBeta\n",
    )?;
    assert_stdin_success_parity(
        &cap,
        "tr-delete",
        &["tr", "-d", "0-9"],
        "/usr/bin/tr",
        &["-d", "0-9"],
        b"a1b2c3\n",
    )?;
    assert_stdin_success_parity(
        &cap,
        "tr-delete-class-digit",
        &["tr", "-d", "[:digit:]"],
        "/usr/bin/tr",
        &["-d", "[:digit:]"],
        b"a1b2c3\n",
    )?;
    let head_tail_input = b"one\nthree words\nfive six seven\nlast\n";
    assert_stdin_success_parity(
        &cap,
        "head-stdin-default",
        &["head"],
        "/usr/bin/head",
        &[],
        head_tail_input,
    )?;
    assert_stdin_success_parity(
        &cap,
        "head-stdin-lines",
        &["head", "-n", "2"],
        "/usr/bin/head",
        &["-n", "2"],
        head_tail_input,
    )?;
    assert_stdin_success_parity(
        &cap,
        "head-stdin-bytes",
        &["head", "-c", "9"],
        "/usr/bin/head",
        &["-c", "9"],
        head_tail_input,
    )?;
    assert_stdin_success_parity(
        &cap,
        "tail-stdin-default",
        &["tail"],
        "/usr/bin/tail",
        &[],
        head_tail_input,
    )?;
    assert_stdin_success_parity(
        &cap,
        "tail-stdin-lines",
        &["tail", "-n", "2"],
        "/usr/bin/tail",
        &["-n", "2"],
        head_tail_input,
    )?;
    assert_stdin_success_parity(
        &cap,
        "tail-stdin-bytes",
        &["tail", "-c", "9"],
        "/usr/bin/tail",
        &["-c", "9"],
        head_tail_input,
    )?;
    assert_stdin_success_parity(
        &cap,
        "sort-stdin-lines",
        &["sort"],
        "/usr/bin/sort",
        &[],
        b"gamma\nalpha\nbeta\n",
    )?;
    assert_stdin_success_parity(
        &cap,
        "uniq-stdin-lines",
        &["uniq"],
        "/usr/bin/uniq",
        &[],
        b"alpha\nalpha\nbeta\nalpha\n",
    )?;
    assert_stdin_success_parity(
        &cap,
        "cut-stdin-first-field",
        &["cut", "-d,", "-f1"],
        "/usr/bin/cut",
        &["-d,", "-f1"],
        b"alpha,beta\nplain\ngamma,delta\n",
    )?;
    assert_stdin_success_parity(
        &cap,
        "awk-stdin-first-field",
        &["awk", "{ print $1 }"],
        "/usr/bin/awk",
        &["{ print $1 }"],
        b"alpha beta\n gamma\n\n",
    )?;
    assert_stdin_success_parity(
        &cap,
        "awk-stdin-second-field",
        &["awk", "{ print $2 }"],
        "/usr/bin/awk",
        &["{ print $2 }"],
        b"alpha beta\n gamma\n\na b c\n",
    )?;
    assert_stdin_success_parity(
        &cap,
        "awk-stdin-filtered-first-field",
        &["awk", "/NEEDLE/ { print $1 }"],
        "/usr/bin/awk",
        &["/NEEDLE/ { print $1 }"],
        b"alpha nope\nhit NEEDLE rest\nNEEDLE-first x\n",
    )?;
    assert_stdin_success_parity(
        &cap,
        "awk-stdin-needle-count",
        &["awk", "/NEEDLE/ { c++ } END { print c }"],
        "/usr/bin/awk",
        &["/NEEDLE/ { c++ } END { print c }"],
        b"NEEDLE one\nplain\nNEEDLE two\n",
    )?;
    assert_run_string_stdin_success_parity(
        &cap,
        "run awk stdin first-field wc pipe",
        "awk '{ print $1 }' | wc -l",
        "/bin/bash",
        &["-c", "awk '{ print $1 }' | wc -l"],
        b"alpha beta\n gamma\n\n",
    )?;
    assert_run_string_stdin_success_parity(
        &cap,
        "run awk stdin second-field wc pipe",
        "awk '{ print $2 }' | wc -l",
        "/bin/bash",
        &["-c", "awk '{ print $2 }' | wc -l"],
        b"alpha beta\n gamma\n\na b c\n",
    )?;
    assert_run_string_stdin_success_parity(
        &cap,
        "run awk stdin filtered first-field wc pipe",
        "awk '/NEEDLE/ { print $1 }' | wc -l",
        "/bin/bash",
        &["-c", "awk '/NEEDLE/ { print $1 }' | wc -l"],
        b"alpha nope\nhit NEEDLE rest\nNEEDLE-first x\n",
    )?;
    assert_stdin_success_parity(
        &cap,
        "grep-stdin-literal",
        &["grep", "NEEDLE"],
        "/usr/bin/grep",
        &["NEEDLE"],
        b"alpha\nNEEDLE one\nNEEDLE two\n",
    )?;
    assert_run_string_stdin_success_parity(
        &cap,
        "run grep stdin wc pipe",
        "grep NEEDLE | wc -l",
        "/bin/bash",
        &["-c", "grep NEEDLE | wc -l"],
        b"alpha\nNEEDLE one\nNEEDLE two\n",
    )?;
    assert_run_string_stdin_success_parity(
        &cap,
        "run grep stdin head pipe",
        "grep NEEDLE | head -n 1",
        "/bin/bash",
        &["-c", "grep NEEDLE | head -n 1"],
        b"alpha\nNEEDLE one\nNEEDLE two\n",
    )?;
    assert_run_string_stdin_success_parity(
        &cap,
        "run grep stdin xargs pipe",
        "grep NEEDLE | xargs echo",
        "/bin/bash",
        &["-c", "grep NEEDLE | xargs echo"],
        b"alpha\nNEEDLE one\nNEEDLE two\n",
    )?;

    let cat_wc_pipe = format!("cat {} | wc -l", fixture.cat_file());
    let cat_wc_bytes_pipe = format!("cat {} | wc -c", fixture.sed_file());
    let echo_wc_pipe = "echo alpha beta | wc -l".to_string();
    let echo_head_pipe = "echo -n alpha beta | head -n 1".to_string();
    let echo_tail_pipe = "echo -n alpha beta | tail -n 1".to_string();
    let echo_tr_pipe = "echo alpha beta | tr a-z A-Z".to_string();
    let echo_tr_class_pipe = "echo Alpha beta | tr '[:lower:]' '[:upper:]'".to_string();
    let echo_awk_pipe = "echo 'alpha beta' 'gamma delta' | awk '{ print $1 }'".to_string();
    let echo_awk_second_pipe = "echo 'alpha beta' 'gamma delta' | awk '{ print $2 }'".to_string();
    let echo_awk_xargs_pipe =
        "echo 'alpha beta' 'gamma delta' | awk '{ print $1 }' | xargs".to_string();
    let cut_stdin_wc_pipe = "cut -d, -f1 | wc -l".to_string();
    let wc_stdin_wc_pipe = "wc -l | wc -l".to_string();
    let wc_stdin_head_pipe = "wc -c | head -n 1".to_string();
    let wc_stdin_grep_wc_pipe = "wc -w | grep 3 | wc -l".to_string();
    let wc_stdin_sort_xargs_pipe = "wc -l | sort | xargs echo".to_string();
    let head_stdin_wc_pipe = "head -n 2 | wc -l".to_string();
    let tail_stdin_wc_pipe = "tail -n 2 | wc -l".to_string();
    let xargs_stdin_wc_pipe = "xargs echo | wc -l".to_string();
    let xargs_default_stdin_wc_pipe = "xargs | wc -l".to_string();
    let xargs_n1_stdin_wc_pipe = "xargs -n 1 echo | wc -l".to_string();
    let xargs_n2_stdin_wc_pipe = "xargs -n 2 echo | wc -l".to_string();
    let xargs_n1_grep_head_pipe = "xargs -n1 echo | grep NEEDLE | head -n 1".to_string();
    let xargs_n2_grep_head_pipe = "xargs -n2 echo | grep NEEDLE | head -n 1".to_string();
    let xargs_grep_pipe = "xargs echo | grep NEEDLE".to_string();
    let xargs_grep_wc_pipe = "xargs echo | grep NEEDLE | wc -l".to_string();
    let xargs_grep_head_pipe = "xargs echo | grep NEEDLE | head -n 1".to_string();
    let true_wc_pipe = "true | wc -l".to_string();
    let false_wc_pipe = "false | wc -l".to_string();
    let false_grep_pipe = "false | grep NEEDLE".to_string();
    let false_grep_wc_pipe = "false | grep NEEDLE | wc -l".to_string();
    let true_xargs_echo_pipe = "true | xargs echo".to_string();
    let mkdir_pipe_dir = path_string(&temp.path().join("mkdir-pipe-created"));
    let mkdir_wc_pipe = format!("mkdir -p {mkdir_pipe_dir} | wc -l");
    let mkdir_xargs_echo_pipe = format!("mkdir -p {mkdir_pipe_dir} | xargs echo");
    let touch_pipe_file = path_string(&temp.path().join("touch-pipe-created.txt"));
    let touch_wc_pipe = format!("touch {touch_pipe_file} | wc -l");
    let touch_sort_xargs_echo_pipe = format!("touch {touch_pipe_file} | sort | xargs echo");
    let test_wc_pipe = format!("test -f {} | wc -l", fixture.cat_file());
    let test_xargs_echo_pipe = format!("test ! -e {quiet_missing} | xargs echo");
    let bracket_sort_xargs_echo_pipe =
        format!("[ -d {} ] | sort | xargs echo", fixture.find_root());
    let test_grep_wc_pipe = format!("test -d {} | grep NEEDLE | wc -l", fixture.cat_file());
    let test_grep_pipe = format!("test -d {} | grep NEEDLE", fixture.cat_file());
    let wc_xargs_echo_pipe = format!("wc -l {} | xargs echo", fixture.wc_files()[0]);
    let wc_multi_wc_pipe = format!(
        "wc -c {} {} | wc -l",
        fixture.wc_files()[0],
        fixture.wc_files()[1]
    );
    let wc_grep_wc_pipe = format!(
        "wc -l {} {} | grep total | wc -l",
        fixture.wc_files()[0],
        fixture.wc_files()[1]
    );
    let wc_sort_xargs_echo_pipe = format!("wc -w {} | sort | xargs echo", fixture.wc_files()[1]);
    let wc_grep_pipe = format!(
        "wc -l {} | grep CAP_PATTERN_THAT_SHOULD_NOT_MATCH",
        fixture.wc_files()[0]
    );
    let du_wc_pipe = format!("du -sk {} | wc -l", fixture.du_root());
    let du_xargs_echo_pipe = format!("du -sk {} | xargs echo", fixture.du_root());
    let du_grep_wc_pipe = format!("du -sk {} | grep du | wc -l", fixture.du_root());
    let echo_xargs_echo_pipe = "echo alpha beta | xargs echo".to_string();
    let echo_xargs_wc_pipe = format!(
        "echo {} {} | xargs wc -l",
        fixture.wc_files()[0],
        fixture.wc_files()[1]
    );
    let printf_wc_pipe = "printf '%s\\n' alpha beta gamma | wc -l".to_string();
    let printf_literal_wc_pipe = "printf 'alpha\\nbeta\\n' | wc -l".to_string();
    let printf_literal_grep_wc_pipe = "printf 'alpha\\nbeta\\n' | grep beta | wc -l".to_string();
    let printf_literal_partial_grep_wc_pipe =
        "printf 'alpha\\nbeta' | grep beta | wc -l".to_string();
    let printf_literal_sort_xargs_echo_pipe =
        "printf 'zeta\\nalpha\\n' | sort | xargs echo".to_string();
    let printf_head_pipe = "printf '%s\\n' alpha beta gamma | head -n 2".to_string();
    let printf_tail_pipe = "printf '%s\\n' alpha beta gamma | tail -n 2".to_string();
    let printf_awk_wc_pipe =
        "printf '%s\\n' 'alpha beta' 'gamma delta' | awk '{ print $1 }' | wc -l".to_string();
    let printf_awk_second_sort_uniq_pipe =
        "printf '%s\\n' 'gamma two' 'alpha one' 'alpha three' | awk '{ print $2 }' | sort | uniq"
            .to_string();
    let printf_awk_sort_uniq_pipe =
        "printf '%s\\n' 'gamma two' 'alpha one' 'alpha three' | awk '{ print $1 }' | sort | uniq"
            .to_string();
    let printf_grep_pipe = "printf '%s\\n' alpha NEEDLE gamma | grep NEEDLE".to_string();
    let printf_grep_wc_pipe =
        "printf '%s\\n' alpha NEEDLE gamma NEEDLE | grep NEEDLE | wc -l".to_string();
    let printf_grep_head_pipe =
        "printf '%s\\n' alpha NEEDLE1 NEEDLE2 gamma | grep NEEDLE | head -n 1".to_string();
    let printf_grep_tail_pipe =
        "printf '%s\\n' alpha NEEDLE1 NEEDLE2 gamma | grep NEEDLE | tail -n 1".to_string();
    let printf_grep_sort_pipe =
        "printf '%s\\n' zeta NEEDLE2 alpha NEEDLE1 | grep NEEDLE | sort".to_string();
    let printf_grep_sort_uniq_pipe =
        "printf '%s\\n' NEEDLE2 NEEDLE1 NEEDLE1 alpha | grep NEEDLE | sort | uniq".to_string();
    let printf_grep_sort_uniq_wc_pipe =
        "printf '%s\\n' NEEDLE2 NEEDLE1 NEEDLE1 alpha | grep NEEDLE | sort | uniq | wc -l"
            .to_string();
    let printf_grep_sort_uniq_head_pipe =
        "printf '%s\\n' NEEDLE2 NEEDLE1 NEEDLE1 alpha | grep NEEDLE | sort | uniq | head -n 1"
            .to_string();
    let printf_grep_sort_uniq_sort_xargs_echo_pipe =
        "printf '%s\\n' NEEDLE2 NEEDLE1 NEEDLE1 alpha | grep NEEDLE | sort | uniq | sort | xargs echo"
            .to_string();
    let printf_grep_sort_uniq_xargs_wc_pipe = format!(
        "printf '%s\\n' {} {} {} | grep count- | sort | uniq | xargs wc -l",
        fixture.wc_files()[1],
        fixture.wc_files()[0],
        fixture.wc_files()[1]
    );
    let printf_grep_sort_wc_pipe =
        "printf '%s\\n' NEEDLE2 NEEDLE1 alpha | grep NEEDLE | sort | wc -l".to_string();
    let printf_grep_sort_head_pipe =
        "printf '%s\\n' NEEDLE2 NEEDLE1 alpha | grep NEEDLE | sort | head -n 1".to_string();
    let printf_grep_sort_tail_pipe =
        "printf '%s\\n' NEEDLE2 NEEDLE1 alpha | grep NEEDLE | sort | tail -n 1".to_string();
    let printf_grep_sort_xargs_echo_pipe =
        "printf '%s\\n' NEEDLE2 NEEDLE1 alpha | grep NEEDLE | sort | xargs echo".to_string();
    let printf_grep_xargs_echo_pipe =
        "printf '%s\\n' alpha NEEDLE1 NEEDLE2 gamma | grep NEEDLE | xargs echo".to_string();
    let printf_tr_pipe = "printf '%s\\n' alpha beta gamma | tr a-z A-Z".to_string();
    let printf_sort_pipe = "printf '%s\\n' gamma alpha beta | sort".to_string();
    let printf_sort_uniq_pipe = "printf '%s\\n' gamma alpha alpha | sort | uniq".to_string();
    let printf_sort_uniq_wc_pipe =
        "printf '%s\\n' gamma alpha alpha | sort | uniq | wc -l".to_string();
    let printf_sort_uniq_head_pipe =
        "printf '%s\\n' gamma alpha alpha | sort | uniq | head -n 1".to_string();
    let printf_sort_uniq_sort_xargs_echo_pipe =
        "printf '%s\\n' gamma alpha alpha | sort | uniq | sort | xargs echo".to_string();
    let printf_sort_uniq_xargs_wc_pipe = format!(
        "printf '%s\\n' {} {} {} | sort | uniq | xargs wc -l",
        fixture.wc_files()[1],
        fixture.wc_files()[0],
        fixture.wc_files()[1]
    );
    let printf_sort_wc_pipe = "printf '%s\\n' gamma alpha beta | sort | wc -l".to_string();
    let printf_sort_head_pipe = "printf '%s\\n' gamma alpha beta | sort | head -n 1".to_string();
    let printf_sort_tail_pipe = "printf '%s\\n' gamma alpha beta | sort | tail -n 1".to_string();
    let printf_sort_xargs_echo_pipe =
        "printf '%s\\n' gamma alpha beta | sort | xargs echo".to_string();
    let printf_sort_xargs_n1_pipe =
        "printf '%s\\n' gamma alpha beta | sort | xargs -n1 echo".to_string();
    let printf_sort_xargs_n2_pipe =
        "printf '%s\\n' gamma alpha beta delta | sort | xargs -n2 echo".to_string();
    let printf_sort_xargs_wc_pipe = format!(
        "printf '%s\\n' {} {} | sort | xargs wc -l",
        fixture.wc_files()[1],
        fixture.wc_files()[0]
    );
    let printf_xargs_echo_pipe = "printf '%s\\n' alpha beta gamma | xargs echo".to_string();
    let printf_xargs_n1_pipe = "printf '%s\\n' alpha beta gamma | xargs -n 1 echo".to_string();
    let printf_xargs_n2_pipe =
        "printf '%s\\n' alpha beta gamma delta | xargs -n 2 echo".to_string();
    let printf_xargs_wc_pipe = format!(
        "printf '%s\\n' {} {} | xargs wc -l",
        fixture.wc_files()[0],
        fixture.wc_files()[1]
    );
    let seq_wc_pipe = "seq 1 10 | wc -l".to_string();
    let seq_head_pipe = "seq 1 10 | head -n 3".to_string();
    let seq_tail_pipe = "seq 1 10 | tail -n 3".to_string();
    let seq_sort_pipe = "seq 1 10 | sort".to_string();
    let seq_sort_uniq_pipe = "seq 1 10 | sort | uniq".to_string();
    let seq_sort_uniq_wc_pipe = "seq 1 10 | sort | uniq | wc -l".to_string();
    let seq_sort_uniq_head_pipe = "seq 1 10 | sort | uniq | head -n 3".to_string();
    let seq_sort_uniq_sort_xargs_echo_pipe =
        "seq 1 10 | sort | uniq | sort | xargs echo".to_string();
    let seq_sort_wc_pipe = "seq 1 10 | sort | wc -l".to_string();
    let seq_sort_head_pipe = "seq 1 10 | sort | head -n 3".to_string();
    let seq_sort_tail_pipe = "seq 1 10 | sort | tail -n 3".to_string();
    let seq_sort_xargs_echo_pipe = "seq 1 10 | sort | xargs echo".to_string();
    let seq_grep_pipe = "seq 1 20 | grep 1".to_string();
    let seq_grep_wc_pipe = "seq 1 20 | grep 1 | wc -l".to_string();
    let seq_grep_head_pipe = "seq 1 20 | grep 1 | head -n 3".to_string();
    let seq_grep_tail_pipe = "seq 1 20 | grep 1 | tail -n 3".to_string();
    let seq_grep_sort_pipe = "seq 1 20 | grep 1 | sort".to_string();
    let seq_grep_sort_uniq_pipe = "seq 1 20 | grep 1 | sort | uniq".to_string();
    let seq_grep_sort_uniq_wc_pipe = "seq 1 20 | grep 1 | sort | uniq | wc -l".to_string();
    let seq_grep_sort_uniq_head_pipe = "seq 1 20 | grep 1 | sort | uniq | head -n 3".to_string();
    let seq_grep_sort_uniq_sort_xargs_echo_pipe =
        "seq 1 20 | grep 1 | sort | uniq | sort | xargs echo".to_string();
    let seq_grep_sort_wc_pipe = "seq 1 20 | grep 1 | sort | wc -l".to_string();
    let seq_grep_sort_head_pipe = "seq 1 20 | grep 1 | sort | head -n 3".to_string();
    let seq_grep_sort_tail_pipe = "seq 1 20 | grep 1 | sort | tail -n 3".to_string();
    let seq_grep_sort_xargs_echo_pipe = "seq 1 20 | grep 1 | sort | xargs echo".to_string();
    let seq_grep_xargs_echo_pipe = "seq 1 20 | grep 1 | xargs echo".to_string();
    let seq_xargs_echo_pipe = "seq 1 10 | xargs echo".to_string();
    let yes_default_head_pipe = "yes | head -n 5".to_string();
    let yes_head_pipe = "yes READY | head -n 5".to_string();
    let which_wc_pipe = "which sh echo | wc -l".to_string();
    let which_head_pipe = "which sh echo | head -n 1".to_string();
    let which_tail_pipe = "which sh echo | tail -n 1".to_string();
    let which_grep_wc_pipe = "which sh echo | grep / | wc -l".to_string();
    let which_grep_xargs_pipe = "which sh echo | grep / | xargs echo".to_string();
    let which_xargs_pipe = "which sh echo | xargs echo".to_string();
    let which_sort_wc_pipe = "which sh echo | sort | wc -l".to_string();
    let which_sort_xargs_pipe = "which sh echo | sort | xargs echo".to_string();
    let which_all_wc_pipe = "which -a sh echo | wc -l".to_string();
    let which_all_head_pipe = "which -a sh echo | head -n 1".to_string();
    let which_all_grep_wc_pipe = "which -a sh echo | grep / | wc -l".to_string();
    let which_all_xargs_pipe = "which -a sh echo | xargs echo".to_string();
    let which_all_sort_xargs_pipe = "which -a sh echo | sort | xargs echo".to_string();
    let command_v_wc_pipe = "command -v sh echo | wc -l".to_string();
    let command_v_head_pipe = "command -v sh echo | head -n 1".to_string();
    let command_v_tail_pipe = "command -v sh echo | tail -n 1".to_string();
    let command_v_grep_wc_pipe = "command -v sh echo | grep / | wc -l".to_string();
    let command_v_grep_head_pipe = "command -v sh echo | grep / | head -n 1".to_string();
    let command_v_xargs_pipe = "command -v sh echo | xargs echo".to_string();
    let command_v_sort_wc_pipe = "command -v sh echo | sort | wc -l".to_string();
    let command_v_sort_xargs_pipe = "command -v sh echo | sort | xargs echo".to_string();
    let printenv_path_wc_pipe = "printenv PATH | wc -l".to_string();
    let printenv_path_head_pipe = "printenv PATH | head -n 1".to_string();
    let printenv_path_tail_pipe = "printenv PATH | tail -n 1".to_string();
    let printenv_path_grep_pipe = "printenv PATH | grep /".to_string();
    let printenv_path_grep_wc_pipe = "printenv PATH | grep / | wc -l".to_string();
    let printenv_path_grep_head_pipe = "printenv PATH | grep / | head -n 1".to_string();
    let printenv_path_grep_sort_pipe = "printenv PATH | grep / | sort".to_string();
    let printenv_path_grep_xargs_pipe = "printenv PATH | grep / | xargs echo".to_string();
    let printenv_path_sort_pipe = "printenv PATH | sort".to_string();
    let printenv_path_xargs_pipe = "printenv PATH | xargs echo".to_string();
    let printenv_path_sort_xargs_pipe = "printenv PATH | sort | xargs echo".to_string();
    let hostname_wc_pipe = "hostname | wc -l".to_string();
    let hostname_head_pipe = "hostname | head -n 1".to_string();
    let hostname_tail_pipe = "hostname | tail -n 1".to_string();
    let hostname_grep_pipe = format!("hostname | grep {hostname_pattern}");
    let hostname_grep_wc_pipe = format!("hostname | grep {hostname_pattern} | wc -l");
    let hostname_grep_xargs_pipe = format!("hostname | grep {hostname_pattern} | xargs echo");
    let hostname_sort_pipe = "hostname | sort".to_string();
    let hostname_xargs_pipe = "hostname | xargs echo".to_string();
    let hostname_sort_xargs_pipe = "hostname | sort | xargs echo".to_string();
    let pwd_wc_pipe = "pwd | wc -l".to_string();
    let pwd_grep_wc_pipe = "pwd | grep / | wc -l".to_string();
    let pwd_xargs_echo_pipe = "pwd | xargs echo".to_string();
    let basename_wc_pipe = format!("basename {} .txt | wc -l", fixture.basename_path());
    let basename_grep_xargs_pipe = format!(
        "basename {} .txt | grep example | xargs echo",
        fixture.basename_path()
    );
    let dirname_sort_tail_pipe = format!("dirname {} | sort | tail -n 1", fixture.basename_path());
    let whoami_wc_pipe = "whoami | wc -l".to_string();
    let id_wc_pipe = "id | wc -l".to_string();
    let id_grep_xargs_pipe = "id | grep uid | xargs echo".to_string();
    let id_u_head_pipe = "id -u | head -n 1".to_string();
    let id_un_xargs_pipe = "id -un | xargs echo".to_string();
    let id_groups_wc_words_pipe = "id -G | wc -w".to_string();
    let id_group_names_sort_xargs_pipe = "id -Gn | sort | xargs echo".to_string();
    let uname_m_sort_pipe = "uname -m | sort".to_string();
    let uname_p_xargs_pipe = "uname -p | xargs echo".to_string();
    let uname_a_xargs_pipe = "uname -a | xargs echo".to_string();
    let sed_wc_pipe = format!("sed -n 1,12p {} | wc -l", fixture.sed_file());
    let sed_head_pipe = format!("sed -n 1,12p {} | head -n 3", fixture.sed_file());
    let sed_tail_pipe = format!("sed -n 1,12p {} | tail -n 3", fixture.sed_file());
    let sed_sort_pipe = format!("sed -n 1,12p {} | sort", fixture.sed_file());
    let sed_sort_uniq_pipe = format!("sed -n 1,12p {} | sort | uniq", fixture.sed_file());
    let sed_sort_uniq_wc_pipe =
        format!("sed -n 1,12p {} | sort | uniq | wc -l", fixture.sed_file());
    let sed_sort_wc_pipe = format!("sed -n 1,12p {} | sort | wc -l", fixture.sed_file());
    let sed_sort_head_pipe = format!("sed -n 1,12p {} | sort | head -n 3", fixture.sed_file());
    let sed_sort_tail_pipe = format!("sed -n 1,12p {} | sort | tail -n 3", fixture.sed_file());
    let sed_sort_xargs_echo_pipe =
        format!("sed -n 1,12p {} | sort | xargs echo", fixture.sed_file());
    let sed_sort_xargs_wc_pipe = format!(
        "sed -n 1,2p {} | sort | xargs wc -l",
        fixture.xargs_wc_file()
    );
    let sed_xargs_echo_pipe = format!("sed -n 1,12p {} | xargs echo", fixture.sed_file());
    let sed_xargs_wc_pipe = format!("sed -n 1,2p {} | xargs wc -l", fixture.xargs_wc_file());
    let sed_grep_pipe = format!("sed -n 1,12p {} | grep 'line 000'", fixture.sed_file());
    let sed_grep_wc_pipe = format!(
        "sed -n 1,12p {} | grep 'line 000' | wc -l",
        fixture.sed_file()
    );
    let sed_grep_head_pipe = format!(
        "sed -n 1,12p {} | grep 'line 000' | head -n 3",
        fixture.sed_file()
    );
    let sed_grep_tail_pipe = format!(
        "sed -n 1,12p {} | grep 'line 000' | tail -n 3",
        fixture.sed_file()
    );
    let sed_grep_sort_pipe = format!(
        "sed -n 1,12p {} | grep 'line 000' | sort",
        fixture.sed_file()
    );
    let sed_grep_sort_uniq_pipe = format!(
        "sed -n 1,12p {} | grep 'line 000' | sort | uniq",
        fixture.sed_file()
    );
    let sed_grep_sort_uniq_wc_pipe = format!(
        "sed -n 1,12p {} | grep 'line 000' | sort | uniq | wc -l",
        fixture.sed_file()
    );
    let sed_grep_sort_wc_pipe = format!(
        "sed -n 1,12p {} | grep 'line 000' | sort | wc -l",
        fixture.sed_file()
    );
    let sed_grep_sort_head_pipe = format!(
        "sed -n 1,12p {} | grep 'line 000' | sort | head -n 3",
        fixture.sed_file()
    );
    let sed_grep_sort_tail_pipe = format!(
        "sed -n 1,12p {} | grep 'line 000' | sort | tail -n 3",
        fixture.sed_file()
    );
    let sed_grep_sort_xargs_echo_pipe = format!(
        "sed -n 1,12p {} | grep 'line 000' | sort | xargs echo",
        fixture.sed_file()
    );
    let sed_grep_sort_xargs_wc_pipe = format!(
        "sed -n 1,2p {} | grep count | sort | xargs wc -l",
        fixture.xargs_wc_file()
    );
    let sed_grep_xargs_echo_pipe = format!(
        "sed -n 1,12p {} | grep 'line 000' | xargs echo",
        fixture.sed_file()
    );
    let sed_grep_xargs_wc_pipe = format!(
        "sed -n 1,2p {} | grep count | xargs wc -l",
        fixture.xargs_wc_file()
    );
    let cat_sed_pipe = format!("cat {} | sed -n 1,12p", fixture.sed_file());
    let cat_sed_wc_pipe = format!("cat {} | sed -n 1,12p | wc -l", fixture.sed_file());
    let cat_sed_head_pipe = format!("cat {} | sed -n 1,12p | head -n 3", fixture.sed_file());
    let cat_sed_tail_pipe = format!("cat {} | sed -n 1,12p | tail -n 3", fixture.sed_file());
    let cat_sed_sort_pipe = format!("cat {} | sed -n 1,12p | sort", fixture.sed_file());
    let cat_sed_sort_uniq_pipe = format!("cat {} | sed -n 1,12p | sort | uniq", fixture.sed_file());
    let cat_sed_sort_uniq_wc_pipe = format!(
        "cat {} | sed -n 1,12p | sort | uniq | wc -l",
        fixture.sed_file()
    );
    let cat_sed_sort_xargs_echo_pipe = format!(
        "cat {} | sed -n 1,12p | sort | xargs echo",
        fixture.sed_file()
    );
    let cat_sed_sort_xargs_wc_pipe = format!(
        "cat {} | sed -n 1,2p | sort | xargs wc -l",
        fixture.xargs_wc_file()
    );
    let cat_sed_xargs_echo_pipe = format!("cat {} | sed -n 1,12p | xargs echo", fixture.sed_file());
    let cat_sed_xargs_wc_pipe = format!(
        "cat {} | sed -n 1,2p | xargs wc -l",
        fixture.xargs_wc_file()
    );
    let cat_sed_grep_pipe = format!(
        "cat {} | sed -n 1,12p | grep 'line 000'",
        fixture.sed_file()
    );
    let cat_sed_grep_wc_pipe = format!(
        "cat {} | sed -n 1,12p | grep 'line 000' | wc -l",
        fixture.sed_file()
    );
    let cat_sed_grep_sort_uniq_wc_pipe = format!(
        "cat {} | sed -n 1,12p | grep 'line 000' | sort | uniq | wc -l",
        fixture.sed_file()
    );
    let cat_sed_grep_sort_xargs_echo_pipe = format!(
        "cat {} | sed -n 1,12p | grep 'line 000' | sort | xargs echo",
        fixture.sed_file()
    );
    let cat_sed_grep_xargs_wc_pipe = format!(
        "cat {} | sed -n 1,2p | grep count | xargs wc -l",
        fixture.xargs_wc_file()
    );
    let cut_wc_pipe = format!("cut -d, -f1 {} | wc -l", fixture.cut_file());
    let cut_head_pipe = format!("cut -d, -f1 {} | head -n 3", fixture.cut_file());
    let cut_tail_pipe = format!("cut -d, -f1 {} | tail -n 3", fixture.cut_file());
    let cut_sort_pipe = format!("cut -d, -f1 {} | sort", fixture.cut_file());
    let cut_sort_uniq_pipe = format!("cut -d, -f1 {} | sort | uniq", fixture.cut_file());
    let cut_sort_uniq_wc_pipe = format!("cut -d, -f1 {} | sort | uniq | wc -l", fixture.cut_file());
    let cut_sort_wc_pipe = format!("cut -d, -f1 {} | sort | wc -l", fixture.cut_file());
    let cut_sort_head_pipe = format!("cut -d, -f1 {} | sort | head -n 3", fixture.cut_file());
    let cut_sort_tail_pipe = format!("cut -d, -f1 {} | sort | tail -n 3", fixture.cut_file());
    let cut_sort_xargs_echo_pipe =
        format!("cut -d, -f1 {} | sort | xargs echo", fixture.cut_file());
    let cut_sort_xargs_wc_pipe = format!(
        "cut -d, -f1 {} | sort | xargs wc -l",
        fixture.xargs_wc_file()
    );
    let cut_xargs_echo_pipe = format!("cut -d, -f1 {} | xargs echo", fixture.cut_file());
    let cut_xargs_wc_pipe = format!("cut -d, -f1 {} | xargs wc -l", fixture.xargs_wc_file());
    let cut_grep_pipe = format!("cut -d, -f1 {} | grep field-000", fixture.cut_file());
    let cut_grep_wc_pipe = format!(
        "cut -d, -f1 {} | grep field-000 | wc -l",
        fixture.cut_file()
    );
    let cut_grep_head_pipe = format!(
        "cut -d, -f1 {} | grep field-000 | head -n 3",
        fixture.cut_file()
    );
    let cut_grep_tail_pipe = format!(
        "cut -d, -f1 {} | grep field-000 | tail -n 3",
        fixture.cut_file()
    );
    let cut_grep_sort_pipe = format!("cut -d, -f1 {} | grep field-000 | sort", fixture.cut_file());
    let cut_grep_sort_uniq_pipe = format!(
        "cut -d, -f1 {} | grep field-000 | sort | uniq",
        fixture.cut_file()
    );
    let cut_grep_sort_uniq_wc_pipe = format!(
        "cut -d, -f1 {} | grep field-000 | sort | uniq | wc -l",
        fixture.cut_file()
    );
    let cut_grep_sort_wc_pipe = format!(
        "cut -d, -f1 {} | grep field-000 | sort | wc -l",
        fixture.cut_file()
    );
    let cut_grep_sort_head_pipe = format!(
        "cut -d, -f1 {} | grep field-000 | sort | head -n 3",
        fixture.cut_file()
    );
    let cut_grep_sort_tail_pipe = format!(
        "cut -d, -f1 {} | grep field-000 | sort | tail -n 3",
        fixture.cut_file()
    );
    let cut_grep_sort_xargs_echo_pipe = format!(
        "cut -d, -f1 {} | grep field-000 | sort | xargs echo",
        fixture.cut_file()
    );
    let cut_grep_sort_xargs_wc_pipe = format!(
        "cut -d, -f1 {} | grep count | sort | xargs wc -l",
        fixture.xargs_wc_file()
    );
    let cut_grep_xargs_echo_pipe = format!(
        "cut -d, -f1 {} | grep field-000 | xargs echo",
        fixture.cut_file()
    );
    let cut_grep_xargs_wc_pipe = format!(
        "cut -d, -f1 {} | grep count | xargs wc -l",
        fixture.xargs_wc_file()
    );
    let ls_wc_pipe = format!("ls -1 {} | wc -l", fixture.list_dir());
    let ls_head_pipe = format!("ls -1 {} | head -n 5", fixture.list_dir());
    let ls_tail_pipe = format!("ls -1 {} | tail -n 5", fixture.list_dir());
    let ls_sort_pipe = format!("ls -1 {} | sort", fixture.list_dir());
    let ls_sort_uniq_pipe = format!("ls -1 {} | sort | uniq", fixture.list_dir());
    let ls_sort_uniq_wc_pipe = format!("ls -1 {} | sort | uniq | wc -l", fixture.list_dir());
    let ls_sort_uniq_head_pipe = format!("ls -1 {} | sort | uniq | head -n 5", fixture.list_dir());
    let ls_sort_uniq_sort_uniq_wc_pipe = format!(
        "ls -1 {} | sort | uniq | sort | uniq | wc -l",
        fixture.list_dir()
    );
    let ls_sort_uniq_xargs_echo_pipe =
        format!("ls -1 {} | sort | uniq | xargs echo", fixture.list_dir());
    let ls_sort_uniq_grep_pipe =
        format!("ls -1 {} | sort | uniq | grep item-000", fixture.list_dir());
    let ls_sort_uniq_grep_sort_xargs_echo_pipe = format!(
        "ls -1 {} | sort | uniq | grep item-000 | sort | xargs echo",
        fixture.list_dir()
    );
    let ls_sort_wc_pipe = format!("ls -1 {} | sort | wc -l", fixture.list_dir());
    let ls_sort_head_pipe = format!("ls -1 {} | sort | head -n 5", fixture.list_dir());
    let ls_sort_tail_pipe = format!("ls -1 {} | sort | tail -n 5", fixture.list_dir());
    let ls_sort_xargs_echo_pipe = format!("ls -1 {} | sort | xargs echo", fixture.list_dir());
    let ls_grep_pipe = format!("ls -1 {} | grep item-000", fixture.list_dir());
    let ls_grep_wc_pipe = format!("ls -1 {} | grep item-000 | wc -l", fixture.list_dir());
    let ls_grep_head_pipe = format!("ls -1 {} | grep item-000 | head -n 2", fixture.list_dir());
    let ls_grep_tail_pipe = format!("ls -1 {} | grep item-000 | tail -n 2", fixture.list_dir());
    let ls_grep_sort_pipe = format!("ls -1 {} | grep item-000 | sort", fixture.list_dir());
    let ls_grep_sort_uniq_wc_pipe = format!(
        "ls -1 {} | grep item-000 | sort | uniq | wc -l",
        fixture.list_dir()
    );
    let ls_grep_xargs_echo_pipe =
        format!("ls -1 {} | grep item-000 | xargs echo", fixture.list_dir());
    let ls_grep_sort_xargs_echo_pipe = format!(
        "ls -1 {} | grep item-000 | sort | xargs echo",
        fixture.list_dir()
    );
    let ls_xargs_echo_pipe = format!("ls -1 {} | xargs echo", fixture.list_dir());
    let ls_all_wc_pipe = format!("ls -a {} | wc -l", fixture.list_dir());
    let ls_all_grep_wc_pipe = format!("ls -a {} | grep hidden | wc -l", fixture.list_dir());
    let ls_all_sort_tail_pipe = format!("ls -a {} | sort | tail -n 1", fixture.list_dir());
    let ls_all_xargs_echo_pipe = format!("ls -a {} | xargs echo", fixture.list_dir());
    let ls_all_sort_xargs_echo_pipe = format!("ls -a {} | sort | xargs echo", fixture.list_dir());
    let ls_almost_wc_pipe = format!("ls -A {} | wc -l", fixture.list_dir());
    let ls_almost_grep_wc_pipe = format!("ls -A {} | grep hidden | wc -l", fixture.list_dir());
    let ls_almost_sort_tail_pipe = format!("ls -A {} | sort | tail -n 1", fixture.list_dir());
    let ls_almost_xargs_echo_pipe = format!("ls -A {} | xargs echo", fixture.list_dir());
    let ls_almost_sort_xargs_echo_pipe =
        format!("ls -A {} | sort | xargs echo", fixture.list_dir());
    let sort_uniq_pipe = format!("sort {} | uniq", fixture.uniq_file());
    let sort_uniq_wc_pipe = format!("sort {} | uniq | wc -l", fixture.uniq_file());
    let sort_uniq_wc_bytes_pipe = format!("sort {} | uniq | wc -c", fixture.sort_file());
    let sort_uniq_head_pipe = format!("sort {} | uniq | head -n 2", fixture.uniq_file());
    let sort_uniq_sort_uniq_wc_pipe =
        format!("sort {} | uniq | sort | uniq | wc -l", fixture.uniq_file());
    let sort_uniq_xargs_wc_pipe = format!("sort {} | uniq | xargs wc -l", fixture.xargs_wc_file());
    let sort_uniq_grep_pipe = format!("sort {} | uniq | grep same", fixture.uniq_file());
    let sort_uniq_grep_sort_xargs_echo_pipe = format!(
        "sort {} | uniq | grep same | sort | xargs echo",
        fixture.uniq_file()
    );
    let sort_uniq_grep_xargs_wc_pipe = format!(
        "sort {} | uniq | grep count | xargs wc -l",
        fixture.xargs_wc_file()
    );
    let sort_grep_pipe = format!("sort {} | grep same", fixture.sort_file());
    let sort_grep_wc_pipe = format!("sort {} | grep same | wc -l", fixture.sort_file());
    let sort_grep_sort_xargs_echo_pipe = format!(
        "sort {} | grep same | sort | xargs echo",
        fixture.sort_file()
    );
    let sort_grep_xargs_wc_pipe = format!(
        "sort {} | grep count | xargs wc -l",
        fixture.xargs_wc_file()
    );
    let sort_head_pipe = format!("sort {} | head -n 5", fixture.sort_file());
    let sort_tail_pipe = format!("sort {} | tail -n 5", fixture.sort_file());
    let sort_wc_pipe = format!("sort {} | wc -l", fixture.sort_file());
    let head_wc_pipe = format!("head -n 4 {} | wc -l", fixture.window_file());
    let head_no_newline_wc_pipe = format!("head -n 2 {} | wc -l", fixture.no_newline_file());
    let head_head_pipe = format!("head -n 4 {} | head -n 2", fixture.window_file());
    let head_tail_pipe = format!("head -n 4 {} | tail -n 2", fixture.window_file());
    let head_sort_pipe = format!("head -n 4 {} | sort", fixture.window_file());
    let head_sort_uniq_pipe = format!("head -n 5 {} | sort | uniq", fixture.uniq_file());
    let head_sort_uniq_wc_pipe = format!("head -n 5 {} | sort | uniq | wc -l", fixture.uniq_file());
    let head_sort_wc_pipe = format!("head -n 4 {} | sort | wc -l", fixture.window_file());
    let head_sort_head_pipe = format!("head -n 4 {} | sort | head -n 2", fixture.window_file());
    let head_sort_tail_pipe = format!("head -n 4 {} | sort | tail -n 2", fixture.window_file());
    let head_sort_xargs_echo_pipe =
        format!("head -n 4 {} | sort | xargs echo", fixture.window_file());
    let head_sort_xargs_wc_pipe =
        format!("head -n 2 {} | sort | xargs wc -l", fixture.xargs_wc_file());
    let head_xargs_echo_pipe = format!("head -n 4 {} | xargs echo", fixture.window_file());
    let head_xargs_wc_pipe = format!("head -n 2 {} | xargs wc -l", fixture.xargs_wc_file());
    let head_grep_pipe = format!("head -n 4 {} | grep t", fixture.window_file());
    let head_grep_no_newline_pipe =
        format!("head -n 2 {} | grep NEEDLE", fixture.no_newline_file());
    let head_grep_wc_pipe = format!("head -n 4 {} | grep t | wc -l", fixture.window_file());
    let head_grep_wc_bytes_pipe = format!("head -n 3 {} | grep z | wc -c", fixture.sort_file());
    let head_grep_head_pipe = format!("head -n 4 {} | grep t | head -n 2", fixture.window_file());
    let head_grep_tail_pipe = format!("head -n 4 {} | grep t | tail -n 2", fixture.window_file());
    let head_grep_sort_pipe = format!("head -n 4 {} | grep t | sort", fixture.window_file());
    let head_grep_sort_uniq_pipe = format!(
        "head -n 5 {} | grep same | sort | uniq",
        fixture.uniq_file()
    );
    let head_grep_sort_uniq_wc_pipe = format!(
        "head -n 5 {} | grep same | sort | uniq | wc -l",
        fixture.uniq_file()
    );
    let head_grep_sort_wc_pipe = format!(
        "head -n 4 {} | grep t | sort | wc -l",
        fixture.window_file()
    );
    let head_grep_sort_head_pipe = format!(
        "head -n 4 {} | grep t | sort | head -n 2",
        fixture.window_file()
    );
    let head_grep_sort_tail_pipe = format!(
        "head -n 4 {} | grep t | sort | tail -n 2",
        fixture.window_file()
    );
    let head_grep_sort_xargs_echo_pipe = format!(
        "head -n 4 {} | grep t | sort | xargs echo",
        fixture.window_file()
    );
    let head_grep_sort_xargs_wc_pipe = format!(
        "head -n 2 {} | grep count | sort | xargs wc -l",
        fixture.xargs_wc_file()
    );
    let head_grep_xargs_echo_pipe =
        format!("head -n 4 {} | grep t | xargs echo", fixture.window_file());
    let head_grep_xargs_wc_pipe = format!(
        "head -n 2 {} | grep count | xargs wc -l",
        fixture.xargs_wc_file()
    );
    let tail_wc_pipe = format!("tail -n 4 {} | wc -l", fixture.window_file());
    let tail_no_newline_wc_pipe = format!("tail -n 2 {} | wc -l", fixture.no_newline_file());
    let tail_zero_wc_pipe = format!("tail -n 0 {} | wc -l", fixture.window_file());
    let tail_head_pipe = format!("tail -n 4 {} | head -n 2", fixture.window_file());
    let tail_tail_pipe = format!("tail -n 4 {} | tail -n 2", fixture.window_file());
    let tail_sort_pipe = format!("tail -n 4 {} | sort", fixture.window_file());
    let tail_sort_uniq_pipe = format!("tail -n 5 {} | sort | uniq", fixture.uniq_file());
    let tail_sort_uniq_wc_pipe = format!("tail -n 5 {} | sort | uniq | wc -l", fixture.uniq_file());
    let tail_sort_wc_pipe = format!("tail -n 4 {} | sort | wc -l", fixture.window_file());
    let tail_sort_head_pipe = format!("tail -n 4 {} | sort | head -n 2", fixture.window_file());
    let tail_sort_tail_pipe = format!("tail -n 4 {} | sort | tail -n 2", fixture.window_file());
    let tail_sort_xargs_echo_pipe =
        format!("tail -n 4 {} | sort | xargs echo", fixture.window_file());
    let tail_sort_xargs_wc_pipe =
        format!("tail -n 2 {} | sort | xargs wc -l", fixture.xargs_wc_file());
    let tail_xargs_echo_pipe = format!("tail -n 4 {} | xargs echo", fixture.window_file());
    let tail_xargs_wc_pipe = format!("tail -n 2 {} | xargs wc -l", fixture.xargs_wc_file());
    let tail_grep_pipe = format!("tail -n 4 {} | grep t", fixture.window_file());
    let tail_grep_no_newline_pipe =
        format!("tail -n 2 {} | grep NEEDLE", fixture.no_newline_file());
    let tail_grep_wc_pipe = format!("tail -n 4 {} | grep t | wc -l", fixture.window_file());
    let tail_grep_head_pipe = format!("tail -n 4 {} | grep t | head -n 2", fixture.window_file());
    let tail_grep_tail_pipe = format!("tail -n 4 {} | grep t | tail -n 2", fixture.window_file());
    let tail_grep_sort_pipe = format!("tail -n 4 {} | grep t | sort", fixture.window_file());
    let tail_grep_sort_uniq_pipe = format!(
        "tail -n 5 {} | grep same | sort | uniq",
        fixture.uniq_file()
    );
    let tail_grep_sort_uniq_wc_pipe = format!(
        "tail -n 5 {} | grep same | sort | uniq | wc -l",
        fixture.uniq_file()
    );
    let tail_grep_sort_wc_pipe = format!(
        "tail -n 4 {} | grep t | sort | wc -l",
        fixture.window_file()
    );
    let tail_grep_sort_head_pipe = format!(
        "tail -n 4 {} | grep t | sort | head -n 2",
        fixture.window_file()
    );
    let tail_grep_sort_tail_pipe = format!(
        "tail -n 4 {} | grep t | sort | tail -n 2",
        fixture.window_file()
    );
    let tail_grep_sort_xargs_echo_pipe = format!(
        "tail -n 4 {} | grep t | sort | xargs echo",
        fixture.window_file()
    );
    let tail_grep_sort_xargs_wc_pipe = format!(
        "tail -n 2 {} | grep count | sort | xargs wc -l",
        fixture.xargs_wc_file()
    );
    let tail_grep_xargs_echo_pipe =
        format!("tail -n 4 {} | grep t | xargs echo", fixture.window_file());
    let tail_grep_xargs_wc_pipe = format!(
        "tail -n 2 {} | grep count | xargs wc -l",
        fixture.xargs_wc_file()
    );
    let cat_head_pipe = format!("cat {} | head -n 3", fixture.window_file());
    let cat_tail_pipe = format!("cat {} | tail -n 3", fixture.window_file());
    let cat_head_default_pipe = format!("cat {} | head", fixture.window_file());
    let cat_tail_default_pipe = format!("cat {} | tail", fixture.window_file());
    let cat_head_short_pipe = format!("cat {} | head -3", fixture.window_file());
    let cat_tail_short_pipe = format!("cat {} | tail -3", fixture.window_file());
    let cat_head_wc_pipe = format!("cat {} | head -n 4 | wc -l", fixture.window_file());
    let cat_tail_wc_pipe = format!("cat {} | tail -n 4 | wc -l", fixture.window_file());
    let cat_head_sort_uniq_wc_pipe = format!(
        "cat {} | head -n 5 | sort | uniq | wc -l",
        fixture.window_file()
    );
    let cat_tail_sort_uniq_wc_pipe = format!(
        "cat {} | tail -n 5 | sort | uniq | wc -l",
        fixture.window_file()
    );
    let cat_head_grep_sort_xargs_pipe = format!(
        "cat {} | head -n 5 | grep t | sort | xargs echo",
        fixture.window_file()
    );
    let cat_tail_grep_sort_xargs_pipe = format!(
        "cat {} | tail -n 5 | grep t | sort | xargs echo",
        fixture.window_file()
    );
    let cat_head_xargs_wc_pipe =
        format!("cat {} | head -n 2 | xargs wc -l", fixture.xargs_wc_file());
    let cat_tail_xargs_wc_pipe =
        format!("cat {} | tail -n 2 | xargs wc -l", fixture.xargs_wc_file());
    let cat_grep_pipe = format!("cat {} | grep three", fixture.window_file());
    let cat_grep_wc_pipe = format!("cat {} | grep t | wc -l", fixture.window_file());
    let cat_grep_head_pipe = format!("cat {} | grep t | head -n 2", fixture.window_file());
    let cat_grep_tail_pipe = format!("cat {} | grep t | tail -n 2", fixture.window_file());
    let cat_grep_sort_pipe = format!("cat {} | grep t | sort", fixture.window_file());
    let cat_grep_sort_uniq_pipe = format!("cat {} | grep e | sort | uniq", fixture.uniq_file());
    let cat_grep_sort_uniq_wc_pipe =
        format!("cat {} | grep e | sort | uniq | wc -l", fixture.uniq_file());
    let cat_grep_sort_uniq_head_pipe = format!(
        "cat {} | grep e | sort | uniq | head -n 2",
        fixture.uniq_file()
    );
    let cat_grep_sort_uniq_tail_pipe = format!(
        "cat {} | grep e | sort | uniq | tail -n 2",
        fixture.uniq_file()
    );
    let cat_grep_sort_uniq_sort_xargs_pipe = format!(
        "cat {} | grep e | sort | uniq | sort | xargs echo",
        fixture.uniq_file()
    );
    let cat_grep_sort_uniq_xargs_wc_pipe = format!(
        "cat {} | grep count- | sort | uniq | xargs wc -l",
        fixture.xargs_wc_file()
    );
    let cat_grep_sort_uniq_sort_xargs_wc_pipe = format!(
        "cat {} | grep count- | sort | uniq | sort | xargs wc -l",
        fixture.xargs_wc_file()
    );
    let cat_grep_sort_wc_pipe = format!("cat {} | grep t | sort | wc -l", fixture.window_file());
    let cat_grep_sort_head_pipe =
        format!("cat {} | grep t | sort | head -n 2", fixture.window_file());
    let cat_grep_sort_tail_pipe =
        format!("cat {} | grep t | sort | tail -n 2", fixture.window_file());
    let cat_cut_pipe = format!("cat {} | cut -d, -f1", fixture.cut_file());
    let cat_cut_wc_pipe = format!("cat {} | cut -d, -f1 | wc -l", fixture.cut_file());
    let cat_cut_sort_uniq_wc_pipe = format!(
        "cat {} | cut -d, -f1 | sort | uniq | wc -l",
        fixture.cut_file()
    );
    let cat_cut_xargs_wc_pipe = format!(
        "cat {} | cut -d, -f1 | xargs wc -l",
        fixture.xargs_wc_file()
    );
    let cat_cut_grep_pipe = format!("cat {} | cut -d, -f1 | grep field-000", fixture.cut_file());
    let cat_cut_grep_sort_xargs_echo_pipe = format!(
        "cat {} | cut -d, -f1 | grep field-000 | sort | xargs echo",
        fixture.cut_file()
    );
    let cat_cut_grep_xargs_wc_pipe = format!(
        "cat {} | cut -d, -f1 | grep count | xargs wc -l",
        fixture.xargs_wc_file()
    );
    let cat_tr_pipe = format!("cat {} | tr a-z A-Z", fixture.window_file());
    let cat_tr_class_pipe = format!("cat {} | tr '[:lower:]' '[:upper:]'", fixture.window_file());
    let cat_tr_wc_pipe = format!("cat {} | tr a-z A-Z | wc -l", fixture.window_file());
    let cat_tr_sort_uniq_wc_pipe = format!(
        "cat {} | tr a-z A-Z | sort | uniq | wc -l",
        fixture.window_file()
    );
    let cat_tr_xargs_wc_pipe =
        format!("cat {} | tr a-z a-z | xargs wc -l", fixture.xargs_wc_file());
    let cat_tr_grep_pipe = format!("cat {} | tr a-z A-Z | grep THREE", fixture.window_file());
    let cat_tr_grep_sort_xargs_echo_pipe = format!(
        "cat {} | tr a-z A-Z | grep T | sort | xargs echo",
        fixture.window_file()
    );
    let cat_tr_grep_xargs_wc_pipe = format!(
        "cat {} | tr a-z a-z | grep count | xargs wc -l",
        fixture.xargs_wc_file()
    );
    let cat_uniq_pipe = format!("cat {} | uniq", fixture.uniq_file());
    let cat_uniq_wc_pipe = format!("cat {} | uniq | wc -l", fixture.uniq_file());
    let cat_uniq_sort_uniq_wc_pipe =
        format!("cat {} | uniq | sort | uniq | wc -l", fixture.uniq_file());
    let cat_uniq_xargs_wc_pipe = format!("cat {} | uniq | xargs wc -l", fixture.xargs_wc_file());
    let cat_uniq_grep_pipe = format!("cat {} | uniq | grep same", fixture.uniq_file());
    let cat_uniq_grep_sort_xargs_echo_pipe = format!(
        "cat {} | uniq | grep same | sort | xargs echo",
        fixture.uniq_file()
    );
    let cat_uniq_grep_xargs_wc_pipe = format!(
        "cat {} | uniq | grep count | xargs wc -l",
        fixture.xargs_wc_file()
    );
    let uniq_wc_pipe = format!("uniq {} | wc -l", fixture.uniq_file());
    let uniq_sort_uniq_wc_pipe = format!("uniq {} | sort | uniq | wc -l", fixture.uniq_file());
    let uniq_xargs_wc_pipe = format!("uniq {} | xargs wc -l", fixture.xargs_wc_file());
    let uniq_grep_pipe = format!("uniq {} | grep same", fixture.uniq_file());
    let uniq_grep_sort_xargs_echo_pipe = format!(
        "uniq {} | grep same | sort | xargs echo",
        fixture.uniq_file()
    );
    let uniq_grep_xargs_wc_pipe = format!(
        "uniq {} | grep count | xargs wc -l",
        fixture.xargs_wc_file()
    );
    let cat_sort_pipe = format!("cat {} | sort", fixture.sort_file());
    let cat_sort_uniq_pipe = format!("cat {} | sort | uniq", fixture.sort_file());
    let cat_sort_uniq_wc_pipe = format!("cat {} | sort | uniq | wc -l", fixture.sort_file());
    let cat_sort_uniq_head_pipe = format!("cat {} | sort | uniq | head -n 2", fixture.sort_file());
    let cat_sort_uniq_sort_uniq_wc_pipe = format!(
        "cat {} | sort | uniq | sort | uniq | wc -l",
        fixture.sort_file()
    );
    let cat_sort_uniq_xargs_wc_pipe = format!(
        "cat {} | sort | uniq | xargs wc -l",
        fixture.xargs_wc_file()
    );
    let cat_sort_uniq_grep_pipe = format!("cat {} | sort | uniq | grep same", fixture.sort_file());
    let cat_sort_uniq_grep_sort_xargs_echo_pipe = format!(
        "cat {} | sort | uniq | grep same | sort | xargs echo",
        fixture.sort_file()
    );
    let cat_sort_uniq_grep_xargs_wc_pipe = format!(
        "cat {} | sort | uniq | grep count | xargs wc -l",
        fixture.xargs_wc_file()
    );
    let cat_sort_grep_pipe = format!("cat {} | sort | grep same", fixture.sort_file());
    let cat_sort_grep_wc_pipe = format!("cat {} | sort | grep same | wc -l", fixture.sort_file());
    let cat_sort_grep_sort_xargs_echo_pipe = format!(
        "cat {} | sort | grep same | sort | xargs echo",
        fixture.sort_file()
    );
    let cat_sort_grep_xargs_wc_pipe = format!(
        "cat {} | sort | grep count | xargs wc -l",
        fixture.xargs_wc_file()
    );
    let cat_sort_wc_pipe = format!("cat {} | sort | wc -l", fixture.sort_file());
    let cat_sort_wc_words_pipe = format!("cat {} | sort | wc -w", fixture.sort_file());
    let cat_sort_head_pipe = format!("cat {} | sort | head -n 5", fixture.sort_file());
    let cat_sort_tail_pipe = format!("cat {} | sort | tail -n 5", fixture.sort_file());
    let cat_xargs_echo_pipe = format!("cat {} | xargs echo", fixture.window_file());
    let sort_xargs_echo_pipe = format!("sort {} | xargs echo", fixture.uniq_file());
    let cat_sort_xargs_echo_pipe = format!("cat {} | sort | xargs echo", fixture.uniq_file());
    let cat_xargs_wc_pipe = format!("cat {} | xargs wc -l", fixture.xargs_wc_file());
    let cat_xargs_wc_sort_pipe = format!("cat {} | xargs wc -l | sort", fixture.xargs_wc_file());
    let sort_xargs_wc_pipe = format!("sort {} | xargs wc -l", fixture.xargs_wc_file());
    let sort_xargs_wc_sort_tail_pipe = format!(
        "sort {} | xargs wc -l | sort | tail -n 1",
        fixture.xargs_wc_file()
    );
    let cat_sort_xargs_wc_pipe = format!("cat {} | sort | xargs wc -l", fixture.xargs_wc_file());
    let cat_sort_xargs_wc_sort_pipe = format!(
        "cat {} | sort | xargs wc -l | sort",
        fixture.xargs_wc_file()
    );
    let cat_grep_xargs_echo_pipe = format!("cat {} | grep t | xargs echo", fixture.window_file());
    let cat_grep_xargs_wc_pipe = format!(
        "cat {} | grep count- | xargs wc -l",
        fixture.xargs_wc_file()
    );
    let cat_grep_sort_xargs_echo_pipe =
        format!("cat {} | grep t | sort | xargs echo", fixture.window_file());
    let cat_grep_sort_xargs_wc_pipe = format!(
        "cat {} | grep count- | sort | xargs wc -l",
        fixture.xargs_wc_file()
    );
    let grep_head_pipe = format!("grep -R NEEDLE {} | head -n 5", fixture.grep_root());
    let grep_tail_pipe = format!("grep -R NEEDLE {} | tail -n 5", fixture.grep_root());
    let grep_sort_pipe = format!("grep -R NEEDLE {} | sort", fixture.grep_root());
    let grep_sort_uniq_pipe = format!("grep -R NEEDLE {} | sort | uniq", fixture.grep_root());
    let grep_sort_uniq_wc_pipe = format!(
        "grep -R NEEDLE {} | sort | uniq | wc -l",
        fixture.grep_root()
    );
    let grep_sort_uniq_head_pipe = format!(
        "grep -R NEEDLE {} | sort | uniq | head -n 3",
        fixture.grep_root()
    );
    let grep_sort_uniq_tail_pipe = format!(
        "grep -R NEEDLE {} | sort | uniq | tail -n 3",
        fixture.grep_root()
    );
    let grep_sort_uniq_sort_xargs_pipe = format!(
        "grep -R NEEDLE {} | sort | uniq | sort | xargs echo",
        fixture.grep_root()
    );
    let grep_sort_wc_pipe = format!("grep -R NEEDLE {} | sort | wc -l", fixture.grep_root());
    let grep_sort_head_pipe = format!("grep -R NEEDLE {} | sort | head -n 5", fixture.grep_root());
    let grep_sort_tail_pipe = format!("grep -R NEEDLE {} | sort | tail -n 5", fixture.grep_root());
    let grep_wc_pipe = format!("grep -R NEEDLE {} | wc -l", fixture.grep_root());
    let grep_file_wc_pipe = format!("grep NEEDLE {} | wc -l", fixture.grep_file());
    let grep_file_head_pipe = format!("grep NEEDLE {} | head -n 2", fixture.grep_file());
    let grep_file_tail_pipe = format!("grep NEEDLE {} | tail -n 2", fixture.grep_file());
    let grep_file_sort_pipe = format!("grep NEEDLE {} | sort", fixture.grep_file());
    let grep_file_sort_uniq_pipe = format!("grep NEEDLE {} | sort | uniq", fixture.grep_file());
    let grep_file_sort_uniq_wc_pipe =
        format!("grep NEEDLE {} | sort | uniq | wc -l", fixture.grep_file());
    let grep_file_sort_uniq_head_pipe = format!(
        "grep NEEDLE {} | sort | uniq | head -n 2",
        fixture.grep_file()
    );
    let grep_file_sort_uniq_tail_pipe = format!(
        "grep NEEDLE {} | sort | uniq | tail -n 2",
        fixture.grep_file()
    );
    let grep_file_sort_uniq_sort_xargs_pipe = format!(
        "grep NEEDLE {} | sort | uniq | sort | xargs echo",
        fixture.grep_file()
    );
    let grep_file_sort_uniq_xargs_wc_pipe = format!(
        "grep count- {} | sort | uniq | xargs wc -l",
        fixture.xargs_wc_file()
    );
    let grep_file_sort_uniq_sort_xargs_wc_pipe = format!(
        "grep count- {} | sort | uniq | sort | xargs wc -l",
        fixture.xargs_wc_file()
    );
    let grep_file_sort_wc_pipe = format!("grep NEEDLE {} | sort | wc -l", fixture.grep_file());
    let grep_file_sort_head_pipe =
        format!("grep NEEDLE {} | sort | head -n 2", fixture.grep_file());
    let grep_file_sort_tail_pipe =
        format!("grep NEEDLE {} | sort | tail -n 2", fixture.grep_file());
    let grep_file_xargs_pipe = format!("grep NEEDLE {} | xargs echo", fixture.grep_file());
    let grep_file_xargs_wc_pipe = format!("grep count- {} | xargs wc -l", fixture.xargs_wc_file());
    let grep_file_xargs_wc_sort_pipe = format!(
        "grep count- {} | xargs wc -l | sort",
        fixture.xargs_wc_file()
    );
    let grep_file_sort_xargs_pipe =
        format!("grep NEEDLE {} | sort | xargs echo", fixture.grep_file());
    let grep_file_sort_xargs_wc_pipe = format!(
        "grep count- {} | sort | xargs wc -l",
        fixture.xargs_wc_file()
    );
    let grep_file_sort_xargs_wc_sort_tail_pipe = format!(
        "grep count- {} | sort | xargs wc -l | sort | tail -n 1",
        fixture.xargs_wc_file()
    );
    let grep_file_cut_pipe = format!("grep NEEDLE {} | cut -d ' ' -f1", fixture.grep_file());
    let grep_file_cut_wc_pipe = format!(
        "grep NEEDLE {} | cut -d ' ' -f1 | wc -l",
        fixture.grep_file()
    );
    let grep_file_cut_sort_pipe = format!(
        "grep NEEDLE {} | cut -d ' ' -f1 | sort",
        fixture.grep_file()
    );
    let grep_file_cut_sort_uniq_wc_pipe = format!(
        "grep NEEDLE {} | cut -d ' ' -f1 | sort | uniq | wc -l",
        fixture.grep_file()
    );
    let grep_file_cut_grep_wc_pipe = format!(
        "grep NEEDLE {} | cut -d ' ' -f1 | grep NEEDLE | wc -l",
        fixture.grep_file()
    );
    let grep_file_cut_xargs_pipe = format!(
        "grep NEEDLE {} | cut -d ' ' -f1 | xargs echo",
        fixture.grep_file()
    );
    let grep_file_cut_xargs_wc_pipe = format!(
        "grep count- {} | cut -d ' ' -f1 | xargs wc -l",
        fixture.xargs_wc_file()
    );
    let grep_file_awk_pipe = format!("grep NEEDLE {} | awk '{{ print $1 }}'", fixture.grep_file());
    let grep_file_awk_second_wc_pipe = format!(
        "grep NEEDLE {} | awk '{{ print $2 }}' | wc -l",
        fixture.grep_file()
    );
    let grep_file_awk_predicate_pipe = format!(
        "grep NEEDLE {} | awk '/NEEDLE/ {{ print $1 }}'",
        fixture.grep_file()
    );
    let grep_file_awk_wc_pipe = format!(
        "grep NEEDLE {} | awk '{{ print $1 }}' | wc -l",
        fixture.grep_file()
    );
    let grep_file_awk_compact_wc_pipe = format!(
        "grep NEEDLE {} | awk '{{print$1}}' | wc -l",
        fixture.grep_file()
    );
    let grep_file_awk_sort_pipe = format!(
        "grep NEEDLE {} | awk '{{ print $1 }}' | sort",
        fixture.grep_file()
    );
    let grep_file_awk_sort_uniq_wc_pipe = format!(
        "grep NEEDLE {} | awk '{{ print $1 }}' | sort | uniq | wc -l",
        fixture.grep_file()
    );
    let grep_file_awk_grep_wc_pipe = format!(
        "grep NEEDLE {} | awk '{{ print $1 }}' | grep NEEDLE | wc -l",
        fixture.grep_file()
    );
    let grep_file_awk_xargs_pipe = format!(
        "grep NEEDLE {} | awk '{{ print $1 }}' | xargs echo",
        fixture.grep_file()
    );
    let grep_file_awk_xargs_wc_pipe = format!(
        "grep count- {} | awk '{{ print $1 }}' | xargs wc -l",
        fixture.xargs_wc_file()
    );
    let awk_first_wc_pipe = format!("awk '{{ print $1 }}' {} | wc -l", fixture.sed_file());
    let awk_second_sort_pipe = format!("awk '{{ print $2 }}' {} | sort", fixture.sed_file());
    let awk_first_compact_wc_pipe = format!("awk '{{print $1}}' {} | wc -l", fixture.sed_file());
    let awk_first_sort_uniq_wc_pipe = format!(
        "awk '{{ print $1 }}' {} | sort | uniq | wc -l",
        fixture.sed_file()
    );
    let awk_first_xargs_pipe = format!("awk '{{ print $1 }}' {} | xargs echo", fixture.sed_file());
    let awk_first_xargs_wc_pipe = format!(
        "awk '{{ print $1 }}' {} | xargs wc -l",
        fixture.xargs_wc_file()
    );
    let awk_first_xargs_wc_sort_pipe = format!(
        "awk '{{ print $1 }}' {} | xargs wc -l | sort",
        fixture.xargs_wc_file()
    );
    let awk_first_grep_pipe = format!("awk '{{ print $1 }}' {} | grep line", fixture.sed_file());
    let awk_first_grep_wc_pipe = format!(
        "awk '{{ print $1 }}' {} | grep line | wc -l",
        fixture.sed_file()
    );
    let awk_first_grep_sort_uniq_wc_pipe = format!(
        "awk '{{ print $1 }}' {} | grep line | sort | uniq | wc -l",
        fixture.sed_file()
    );
    let awk_first_grep_xargs_wc_pipe = format!(
        "awk '{{ print $1 }}' {} | grep count- | xargs wc -l",
        fixture.awk_xargs_wc_file()
    );
    let awk_first_grep_xargs_wc_sort_pipe = format!(
        "awk '{{ print $1 }}' {} | grep count- | xargs wc -l | sort",
        fixture.awk_xargs_wc_file()
    );
    let awk_xargs_pipe = format!(
        "awk '/NEEDLE/ {{ print $1 }}' {} | xargs echo",
        fixture.sed_file()
    );
    let awk_xargs_wc_pipe = format!(
        "awk '/NEEDLE/ {{ print $1 }}' {} | xargs wc -l",
        fixture.awk_xargs_wc_file()
    );
    let awk_xargs_wc_sort_pipe = format!(
        "awk '/NEEDLE/ {{ print $1 }}' {} | xargs wc -l | sort",
        fixture.awk_xargs_wc_file()
    );
    let awk_wc_pipe = format!(
        "awk '/NEEDLE/ {{ print $1 }}' {} | wc -l",
        fixture.sed_file()
    );
    let awk_head_pipe = format!(
        "awk '/NEEDLE/ {{ print $1 }}' {} | head -n 2",
        fixture.sed_file()
    );
    let awk_tail_pipe = format!(
        "awk '/NEEDLE/ {{ print $1 }}' {} | tail -n 2",
        fixture.sed_file()
    );
    let awk_sort_pipe = format!(
        "awk '/NEEDLE/ {{ print $1 }}' {} | sort",
        fixture.sed_file()
    );
    let awk_sort_uniq_pipe = format!(
        "awk '/NEEDLE/ {{ print $1 }}' {} | sort | uniq",
        fixture.sed_file()
    );
    let awk_sort_uniq_wc_pipe = format!(
        "awk '/NEEDLE/ {{ print $1 }}' {} | sort | uniq | wc -l",
        fixture.sed_file()
    );
    let awk_sort_uniq_head_pipe = format!(
        "awk '/NEEDLE/ {{ print $1 }}' {} | sort | uniq | head -n 2",
        fixture.sed_file()
    );
    let awk_sort_uniq_sort_xargs_pipe = format!(
        "awk '/NEEDLE/ {{ print $1 }}' {} | sort | uniq | sort | xargs echo",
        fixture.sed_file()
    );
    let awk_sort_uniq_xargs_wc_pipe = format!(
        "awk '/NEEDLE/ {{ print $1 }}' {} | sort | uniq | xargs wc -l",
        fixture.awk_xargs_wc_file()
    );
    let awk_sort_uniq_sort_xargs_wc_pipe = format!(
        "awk '/NEEDLE/ {{ print $1 }}' {} | sort | uniq | sort | xargs wc -l",
        fixture.awk_xargs_wc_file()
    );
    let awk_sort_wc_pipe = format!(
        "awk '/NEEDLE/ {{ print $1 }}' {} | sort | wc -l",
        fixture.sed_file()
    );
    let awk_sort_head_pipe = format!(
        "awk '/NEEDLE/ {{ print $1 }}' {} | sort | head -n 2",
        fixture.sed_file()
    );
    let awk_sort_tail_pipe = format!(
        "awk '/NEEDLE/ {{ print $1 }}' {} | sort | tail -n 2",
        fixture.sed_file()
    );
    let awk_sort_xargs_pipe = format!(
        "awk '/NEEDLE/ {{ print $1 }}' {} | sort | xargs echo",
        fixture.sed_file()
    );
    let awk_sort_xargs_wc_pipe = format!(
        "awk '/NEEDLE/ {{ print $1 }}' {} | sort | xargs wc -l",
        fixture.awk_xargs_wc_file()
    );
    let awk_sort_xargs_wc_sort_tail_pipe = format!(
        "awk '/NEEDLE/ {{ print $1 }}' {} | sort | xargs wc -l | sort | tail -n 1",
        fixture.awk_xargs_wc_file()
    );
    let cat_awk_first_pipe = format!("cat {} | awk '{{ print $1 }}'", fixture.sed_file());
    let cat_awk_second_wc_pipe =
        format!("cat {} | awk '{{ print $2 }}' | wc -l", fixture.sed_file());
    let cat_awk_first_compact_pipe = format!("cat {} | awk '{{print$1}}'", fixture.sed_file());
    let cat_awk_first_wc_pipe =
        format!("cat {} | awk '{{ print $1 }}' | wc -l", fixture.sed_file());
    let cat_awk_first_xargs_wc_pipe = format!(
        "cat {} | awk '{{ print $1 }}' | xargs wc -l",
        fixture.xargs_wc_file()
    );
    let cat_awk_first_xargs_wc_sort_pipe = format!(
        "cat {} | awk '{{ print $1 }}' | xargs wc -l | sort",
        fixture.xargs_wc_file()
    );
    let cat_awk_first_grep_tail_pipe = format!(
        "cat {} | awk '{{ print $1 }}' | grep line | tail -n 2",
        fixture.sed_file()
    );
    let cat_awk_first_grep_sort_xargs_wc_pipe = format!(
        "cat {} | awk '{{ print $1 }}' | grep count- | sort | xargs wc -l",
        fixture.awk_xargs_wc_file()
    );
    let cat_awk_first_grep_sort_xargs_wc_sort_tail_pipe = format!(
        "cat {} | awk '{{ print $1 }}' | grep count- | sort | xargs wc -l | sort | tail -n 1",
        fixture.awk_xargs_wc_file()
    );
    let cat_awk_pipe = format!("cat {} | awk '/NEEDLE/ {{ print $1 }}'", fixture.sed_file());
    let cat_awk_wc_pipe = format!(
        "cat {} | awk '/NEEDLE/ {{ print $1 }}' | wc -l",
        fixture.sed_file()
    );
    let cat_awk_head_pipe = format!(
        "cat {} | awk '/NEEDLE/ {{ print $1 }}' | head -n 2",
        fixture.sed_file()
    );
    let cat_awk_tail_pipe = format!(
        "cat {} | awk '/NEEDLE/ {{ print $1 }}' | tail -n 2",
        fixture.sed_file()
    );
    let cat_awk_sort_pipe = format!(
        "cat {} | awk '/NEEDLE/ {{ print $1 }}' | sort",
        fixture.sed_file()
    );
    let cat_awk_sort_uniq_wc_pipe = format!(
        "cat {} | awk '/NEEDLE/ {{ print $1 }}' | sort | uniq | wc -l",
        fixture.sed_file()
    );
    let cat_awk_xargs_pipe = format!(
        "cat {} | awk '/NEEDLE/ {{ print $1 }}' | xargs echo",
        fixture.sed_file()
    );
    let cat_awk_xargs_wc_pipe = format!(
        "cat {} | awk '/NEEDLE/ {{ print $1 }}' | xargs wc -l",
        fixture.awk_xargs_wc_file()
    );
    let cat_awk_xargs_wc_sort_pipe = format!(
        "cat {} | awk '/NEEDLE/ {{ print $1 }}' | xargs wc -l | sort",
        fixture.awk_xargs_wc_file()
    );
    let cat_awk_sort_xargs_pipe = format!(
        "cat {} | awk '/NEEDLE/ {{ print $1 }}' | sort | xargs echo",
        fixture.sed_file()
    );
    let cat_awk_sort_xargs_wc_pipe = format!(
        "cat {} | awk '/NEEDLE/ {{ print $1 }}' | sort | xargs wc -l",
        fixture.awk_xargs_wc_file()
    );
    let cat_awk_sort_xargs_wc_sort_tail_pipe = format!(
        "cat {} | awk '/NEEDLE/ {{ print $1 }}' | sort | xargs wc -l | sort | tail -n 1",
        fixture.awk_xargs_wc_file()
    );
    let find_all_xargs_pipe = format!("find {} -type f | xargs wc -l", fixture.find_root());
    let find_all_xargs_wc_pipe =
        format!("find {} -type f | xargs wc -l | wc -l", fixture.find_root());
    let find_all_xargs_sort_pipe =
        format!("find {} -type f | xargs wc -l | sort", fixture.find_root());
    let find_all_xargs_sort_tail_pipe = format!(
        "find {} -type f | xargs wc -l | sort | tail -n 1",
        fixture.find_root()
    );
    let find_all_xargs_echo_pipe = format!("find {} -type f | xargs echo", fixture.find_root());
    let find_all_xargs_default_pipe = format!("find {} -type f | xargs", fixture.find_root());
    let find_all_wc_pipe = format!("find {} -type f | wc -l", fixture.find_root());
    let find_all_head_pipe = format!("find {} -type f | head -n 5", fixture.find_root());
    let find_all_tail_pipe = format!("find {} -type f | tail -n 5", fixture.find_root());
    let find_all_sort_pipe = format!("find {} -type f | sort", fixture.find_root());
    let find_maxdepth_wc_pipe = format!("find {} -maxdepth 1 -type f | wc -l", fixture.find_root());
    let find_maxdepth_head_pipe = format!(
        "find {} -maxdepth 1 -type f | head -n 5",
        fixture.find_root()
    );
    let find_maxdepth_grep_wc_pipe = format!(
        "find {} -maxdepth 1 -type f | grep source-00 | wc -l",
        fixture.find_root()
    );
    let find_maxdepth_xargs_echo_pipe = format!(
        "find {} -maxdepth 1 -type f | xargs echo",
        fixture.find_root()
    );
    let find_maxdepth_two_sort_tail_pipe = format!(
        "find {} -maxdepth 2 -type f | sort | tail -n 1",
        fixture.find_root()
    );
    let find_maxdepth_two_name_grep_wc_pipe = format!(
        "find {} -maxdepth 2 -type f -name '*.rs' | grep nested-source | wc -l",
        fixture.find_root()
    );
    let find_all_sort_uniq_pipe = format!("find {} -type f | sort | uniq", fixture.find_root());
    let find_all_sort_uniq_wc_pipe =
        format!("find {} -type f | sort | uniq | wc -l", fixture.find_root());
    let find_all_sort_uniq_head_pipe = format!(
        "find {} -type f | sort | uniq | head -n 5",
        fixture.find_root()
    );
    let find_all_sort_uniq_sort_uniq_wc_pipe = format!(
        "find {} -type f | sort | uniq | sort | uniq | wc -l",
        fixture.find_root()
    );
    let find_all_sort_uniq_xargs_wc_pipe = format!(
        "find {} -type f | sort | uniq | xargs wc -l",
        fixture.find_root()
    );
    let find_all_sort_uniq_xargs_sort_wc_pipe = format!(
        "find {} -type f | sort | uniq | xargs wc -l | sort | wc -l",
        fixture.find_root()
    );
    let find_all_sort_uniq_grep_pipe = format!(
        "find {} -type f | sort | uniq | grep entry",
        fixture.find_root()
    );
    let find_all_sort_uniq_grep_sort_xargs_wc_pipe = format!(
        "find {} -type f | sort | uniq | grep entry | sort | xargs wc -l",
        fixture.find_root()
    );
    let find_all_sort_wc_pipe = format!("find {} -type f | sort | wc -l", fixture.find_root());
    let find_all_sort_xargs_echo_pipe =
        format!("find {} -type f | sort | xargs echo", fixture.find_root());
    let find_all_sort_xargs_pipe =
        format!("find {} -type f | sort | xargs wc -l", fixture.find_root());
    let find_all_sort_xargs_sort_tail_pipe = format!(
        "find {} -type f | sort | xargs wc -l | sort | tail -n 1",
        fixture.find_root()
    );
    let find_all_sort_head_pipe =
        format!("find {} -type f | sort | head -n 5", fixture.find_root());
    let find_all_sort_tail_pipe =
        format!("find {} -type f | sort | tail -n 5", fixture.find_root());
    let find_xargs_pipe = format!(
        "find {} -type f -name '*.rs' | xargs wc -l",
        fixture.find_root()
    );
    let find_xargs_sort_pipe = format!(
        "find {} -type f -name '*.rs' | xargs wc -l | sort",
        fixture.find_root()
    );
    let find_xargs_echo_pipe = format!(
        "find {} -type f -name '*.rs' | xargs echo",
        fixture.find_root()
    );
    let find_xargs_default_pipe =
        format!("find {} -type f -name '*.rs' | xargs", fixture.find_root());
    let find_grep_xargs_echo_pipe = format!(
        "find {} -type f -name '*.rs' | grep source-00 | xargs echo",
        fixture.find_root()
    );
    let find_grep_xargs_pipe = format!(
        "find {} -type f -name '*.rs' | grep source-00 | xargs wc -l",
        fixture.find_root()
    );
    let find_grep_wc_pipe = format!(
        "find {} -type f -name '*.rs' | grep source-00 | wc -l",
        fixture.find_root()
    );
    let find_grep_head_pipe = format!(
        "find {} -type f -name '*.rs' | grep source-00 | head -n 5",
        fixture.find_root()
    );
    let find_grep_tail_pipe = format!(
        "find {} -type f -name '*.rs' | grep source-00 | tail -n 5",
        fixture.find_root()
    );
    let find_grep_sort_pipe = format!(
        "find {} -type f -name '*.rs' | grep source-00 | sort",
        fixture.find_root()
    );
    let find_grep_sort_uniq_wc_pipe = format!(
        "find {} -type f -name '*.rs' | grep source-00 | sort | uniq | wc -l",
        fixture.find_root()
    );
    let find_grep_sort_xargs_echo_pipe = format!(
        "find {} -type f -name '*.rs' | grep source-00 | sort | xargs echo",
        fixture.find_root()
    );
    let find_grep_sort_xargs_pipe = format!(
        "find {} -type f -name '*.rs' | grep source-00 | sort | xargs wc -l",
        fixture.find_root()
    );
    let find_grep_sort_xargs_sort_pipe = format!(
        "find {} -type f -name '*.rs' | grep source-00 | sort | xargs wc -l | sort",
        fixture.find_root()
    );
    let find_grep_sort_uniq_xargs_sort_tail_pipe = format!(
        "find {} -type f -name '*.rs' | grep source-00 | sort | uniq | xargs wc -l | sort | tail -n 1",
        fixture.find_root()
    );
    let find_wc_pipe = format!("find {} -type f -name '*.rs' | wc -l", fixture.find_root());
    let find_head_pipe = format!(
        "find {} -type f -name '*.rs' | head -n 5",
        fixture.find_root()
    );
    let find_tail_pipe = format!(
        "find {} -type f -name '*.rs' | tail -n 5",
        fixture.find_root()
    );
    let find_sort_pipe = format!("find {} -type f -name '*.rs' | sort", fixture.find_root());
    let find_sort_uniq_pipe = format!(
        "find {} -type f -name '*.rs' | sort | uniq",
        fixture.find_root()
    );
    let find_sort_uniq_wc_pipe = format!(
        "find {} -type f -name '*.rs' | sort | uniq | wc -l",
        fixture.find_root()
    );
    let find_sort_uniq_xargs_wc_pipe = format!(
        "find {} -type f -name '*.rs' | sort | uniq | xargs wc -l",
        fixture.find_root()
    );
    let find_sort_uniq_xargs_sort_wc_pipe = format!(
        "find {} -type f -name '*.rs' | sort | uniq | xargs wc -l | sort | wc -l",
        fixture.find_root()
    );
    let find_sort_uniq_grep_sort_xargs_wc_pipe = format!(
        "find {} -type f -name '*.rs' | sort | uniq | grep entry | sort | xargs wc -l",
        fixture.find_root()
    );
    let find_sort_wc_pipe = format!(
        "find {} -type f -name '*.rs' | sort | wc -l",
        fixture.find_root()
    );
    let find_sort_xargs_echo_pipe = format!(
        "find {} -type f -name '*.rs' | sort | xargs echo",
        fixture.find_root()
    );
    let find_sort_xargs_pipe = format!(
        "find {} -type f -name '*.rs' | sort | xargs wc -l",
        fixture.find_root()
    );
    let find_sort_xargs_sort_tail_pipe = format!(
        "find {} -type f -name '*.rs' | sort | xargs wc -l | sort | tail -n 1",
        fixture.find_root()
    );
    let find_sort_head_pipe = format!(
        "find {} -type f -name '*.rs' | sort | head -n 5",
        fixture.find_root()
    );
    let find_sort_tail_pipe = format!(
        "find {} -type f -name '*.rs' | sort | tail -n 5",
        fixture.find_root()
    );
    let run_success_cases = [
        ("run true", "true".to_string(), "/usr/bin/true", vec![]),
        ("run false", "false".to_string(), "/usr/bin/false", vec![]),
        ("run pwd", "pwd".to_string(), "/bin/pwd", vec![]),
        (
            "run echo",
            "echo alpha beta".to_string(),
            "/bin/bash",
            vec!["-c", "echo alpha beta"],
        ),
        (
            "run echo n",
            "echo -n alpha beta".to_string(),
            "/bin/bash",
            vec!["-c", "echo -n alpha beta"],
        ),
        (
            "run printf",
            "printf '%s\\n' alpha beta".to_string(),
            "/bin/bash",
            vec!["-c", "printf '%s\\n' alpha beta"],
        ),
        (
            "run printf join",
            "printf '%s' alpha beta".to_string(),
            "/bin/bash",
            vec!["-c", "printf '%s' alpha beta"],
        ),
        (
            "run printf literal",
            "printf 'alpha\\nbeta\\n'".to_string(),
            "/bin/bash",
            vec!["-c", "printf 'alpha\\nbeta\\n'"],
        ),
        (
            "run seq",
            "seq 1 5".to_string(),
            "/usr/bin/seq",
            vec!["1", "5"],
        ),
        (
            "run seq desc",
            "seq 5 -2 1".to_string(),
            "/usr/bin/seq",
            vec!["5", "-2", "1"],
        ),
        (
            "run whoami",
            "whoami".to_string(),
            "/usr/bin/whoami",
            vec![],
        ),
        ("run id", "id".to_string(), "/usr/bin/id", vec![]),
        ("run id u", "id -u".to_string(), "/usr/bin/id", vec!["-u"]),
        (
            "run id un",
            "id -un".to_string(),
            "/usr/bin/id",
            vec!["-un"],
        ),
        ("run id g", "id -g".to_string(), "/usr/bin/id", vec!["-g"]),
        (
            "run id gn",
            "id -gn".to_string(),
            "/usr/bin/id",
            vec!["-gn"],
        ),
        ("run id G", "id -G".to_string(), "/usr/bin/id", vec!["-G"]),
        (
            "run id Gn",
            "id -Gn".to_string(),
            "/usr/bin/id",
            vec!["-Gn"],
        ),
        ("run uname", "uname".to_string(), "/usr/bin/uname", vec![]),
        (
            "run uname a",
            "uname -a".to_string(),
            "/usr/bin/uname",
            vec!["-a"],
        ),
        (
            "run uname m",
            "uname -m".to_string(),
            "/usr/bin/uname",
            vec!["-m"],
        ),
        (
            "run uname p",
            "uname -p".to_string(),
            "/usr/bin/uname",
            vec!["-p"],
        ),
        (
            "run hostname",
            "hostname".to_string(),
            "/bin/hostname",
            vec![],
        ),
        (
            "run test file",
            format!("test -f {}", fixture.cat_file()),
            "/bin/test",
            vec!["-f", fixture.cat_file()],
        ),
        (
            "run test string eq",
            "test alpha = alpha".to_string(),
            "/bin/test",
            vec!["alpha", "=", "alpha"],
        ),
        (
            "run test int gt",
            "test 5 -gt 3".to_string(),
            "/bin/test",
            vec!["5", "-gt", "3"],
        ),
        (
            "run basename",
            format!("basename {} .txt", fixture.basename_path()),
            "/usr/bin/basename",
            vec![fixture.basename_path(), ".txt"],
        ),
        (
            "run dirname",
            format!("dirname {}", fixture.basename_path()),
            "/usr/bin/dirname",
            vec![fixture.basename_path()],
        ),
        (
            "run ls",
            format!("ls -1 {}", fixture.list_dir()),
            "/bin/ls",
            vec!["-1", fixture.list_dir()],
        ),
        (
            "run ls -A",
            format!("ls -A {}", fixture.list_dir()),
            "/bin/ls",
            vec!["-A", fixture.list_dir()],
        ),
        (
            "run cat",
            format!("cat {}", fixture.cat_file()),
            "/bin/cat",
            vec![fixture.cat_file()],
        ),
        (
            "run head",
            format!("head -n 3 {}", fixture.window_file()),
            "/usr/bin/head",
            vec!["-n", "3", fixture.window_file()],
        ),
        (
            "run tail",
            format!("tail -c 17 {}", fixture.window_file()),
            "/usr/bin/tail",
            vec!["-c", "17", fixture.window_file()],
        ),
        (
            "run mkdir",
            format!("mkdir -p {}", fixture.mkdir_existing()),
            "/bin/mkdir",
            vec!["-p", fixture.mkdir_existing()],
        ),
        (
            "run touch",
            format!("touch {}", fixture.touch_file()),
            "/usr/bin/touch",
            vec![fixture.touch_file()],
        ),
        (
            "run touch dir",
            format!("touch {}", fixture.touch_dir()),
            "/usr/bin/touch",
            vec![fixture.touch_dir()],
        ),
        (
            "run uniq",
            format!("uniq {}", fixture.uniq_file()),
            "/usr/bin/uniq",
            vec![fixture.uniq_file()],
        ),
        (
            "run find",
            format!("find {} -type f -name '*.txt'", fixture.find_root()),
            "/usr/bin/find",
            vec![fixture.find_root(), "-type", "f", "-name", "*.txt"],
        ),
        (
            "run du",
            format!("du -sk {}", fixture.du_root()),
            "/usr/bin/du",
            vec!["-sk", fixture.du_root()],
        ),
        (
            "run sort",
            format!("sort {}", fixture.sort_file()),
            "/usr/bin/sort",
            vec![fixture.sort_file()],
        ),
        (
            "run cut",
            format!("cut -d, -f1 {}", fixture.cut_file()),
            "/usr/bin/cut",
            vec!["-d,", "-f1", fixture.cut_file()],
        ),
        (
            "run sed",
            format!("sed -n 1,1024p {}", fixture.sed_file()),
            "/usr/bin/sed",
            vec!["-n", "1,1024p", fixture.sed_file()],
        ),
        (
            "run grep",
            format!("grep -R NEEDLE {}", fixture.grep_root()),
            "/usr/bin/grep",
            vec!["-R", "NEEDLE", fixture.grep_root()],
        ),
        (
            "run grep file",
            format!("grep NEEDLE {}", fixture.grep_file()),
            "/usr/bin/grep",
            vec!["NEEDLE", fixture.grep_file()],
        ),
        (
            "run awk",
            format!(
                "awk '/NEEDLE/ {{ c++ }} END {{ print c }}' {}",
                fixture.sed_file()
            ),
            "/usr/bin/awk",
            vec!["/NEEDLE/ { c++ } END { print c }", fixture.sed_file()],
        ),
        (
            "run which",
            "which sh echo".to_string(),
            "/usr/bin/which",
            vec!["sh", "echo"],
        ),
        (
            "run command v",
            "command -v sh echo".to_string(),
            "/bin/bash",
            vec!["-c", "command -v sh echo"],
        ),
        (
            "run echo wc pipe",
            echo_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", echo_wc_pipe.as_str()],
        ),
        (
            "run echo head pipe",
            echo_head_pipe.clone(),
            "/bin/bash",
            vec!["-c", echo_head_pipe.as_str()],
        ),
        (
            "run echo tail pipe",
            echo_tail_pipe.clone(),
            "/bin/bash",
            vec!["-c", echo_tail_pipe.as_str()],
        ),
        (
            "run echo tr pipe",
            echo_tr_pipe.clone(),
            "/bin/bash",
            vec!["-c", echo_tr_pipe.as_str()],
        ),
        (
            "run echo tr class pipe",
            echo_tr_class_pipe.clone(),
            "/bin/bash",
            vec!["-c", echo_tr_class_pipe.as_str()],
        ),
        (
            "run echo awk pipe",
            echo_awk_pipe.clone(),
            "/bin/bash",
            vec!["-c", echo_awk_pipe.as_str()],
        ),
        (
            "run echo awk second-field pipe",
            echo_awk_second_pipe.clone(),
            "/bin/bash",
            vec!["-c", echo_awk_second_pipe.as_str()],
        ),
        (
            "run echo awk xargs pipe",
            echo_awk_xargs_pipe.clone(),
            "/bin/bash",
            vec!["-c", echo_awk_xargs_pipe.as_str()],
        ),
        (
            "run echo xargs echo pipe",
            echo_xargs_echo_pipe.clone(),
            "/bin/bash",
            vec!["-c", echo_xargs_echo_pipe.as_str()],
        ),
        (
            "run echo xargs wc pipe",
            echo_xargs_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", echo_xargs_wc_pipe.as_str()],
        ),
        (
            "run printf wc pipe",
            printf_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", printf_wc_pipe.as_str()],
        ),
        (
            "run printf literal wc pipe",
            printf_literal_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", printf_literal_wc_pipe.as_str()],
        ),
        (
            "run printf literal grep wc pipe",
            printf_literal_grep_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", printf_literal_grep_wc_pipe.as_str()],
        ),
        (
            "run printf literal partial grep wc pipe",
            printf_literal_partial_grep_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", printf_literal_partial_grep_wc_pipe.as_str()],
        ),
        (
            "run printf literal sort xargs echo pipe",
            printf_literal_sort_xargs_echo_pipe.clone(),
            "/bin/bash",
            vec!["-c", printf_literal_sort_xargs_echo_pipe.as_str()],
        ),
        (
            "run printf head pipe",
            printf_head_pipe.clone(),
            "/bin/bash",
            vec!["-c", printf_head_pipe.as_str()],
        ),
        (
            "run printf tail pipe",
            printf_tail_pipe.clone(),
            "/bin/bash",
            vec!["-c", printf_tail_pipe.as_str()],
        ),
        (
            "run printf awk wc pipe",
            printf_awk_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", printf_awk_wc_pipe.as_str()],
        ),
        (
            "run printf awk second-field sort uniq pipe",
            printf_awk_second_sort_uniq_pipe.clone(),
            "/bin/bash",
            vec!["-c", printf_awk_second_sort_uniq_pipe.as_str()],
        ),
        (
            "run printf awk sort uniq pipe",
            printf_awk_sort_uniq_pipe.clone(),
            "/bin/bash",
            vec!["-c", printf_awk_sort_uniq_pipe.as_str()],
        ),
        (
            "run printf grep pipe",
            printf_grep_pipe.clone(),
            "/bin/bash",
            vec!["-c", printf_grep_pipe.as_str()],
        ),
        (
            "run printf grep wc pipe",
            printf_grep_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", printf_grep_wc_pipe.as_str()],
        ),
        (
            "run printf grep head pipe",
            printf_grep_head_pipe.clone(),
            "/bin/bash",
            vec!["-c", printf_grep_head_pipe.as_str()],
        ),
        (
            "run printf grep tail pipe",
            printf_grep_tail_pipe.clone(),
            "/bin/bash",
            vec!["-c", printf_grep_tail_pipe.as_str()],
        ),
        (
            "run printf grep sort pipe",
            printf_grep_sort_pipe.clone(),
            "/bin/bash",
            vec!["-c", printf_grep_sort_pipe.as_str()],
        ),
        (
            "run printf grep sort uniq pipe",
            printf_grep_sort_uniq_pipe.clone(),
            "/bin/bash",
            vec!["-c", printf_grep_sort_uniq_pipe.as_str()],
        ),
        (
            "run printf grep sort uniq wc pipe",
            printf_grep_sort_uniq_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", printf_grep_sort_uniq_wc_pipe.as_str()],
        ),
        (
            "run printf grep sort uniq head pipe",
            printf_grep_sort_uniq_head_pipe.clone(),
            "/bin/bash",
            vec!["-c", printf_grep_sort_uniq_head_pipe.as_str()],
        ),
        (
            "run printf grep sort uniq sort xargs echo pipe",
            printf_grep_sort_uniq_sort_xargs_echo_pipe.clone(),
            "/bin/bash",
            vec!["-c", printf_grep_sort_uniq_sort_xargs_echo_pipe.as_str()],
        ),
        (
            "run printf grep sort uniq xargs wc pipe",
            printf_grep_sort_uniq_xargs_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", printf_grep_sort_uniq_xargs_wc_pipe.as_str()],
        ),
        (
            "run printf grep sort wc pipe",
            printf_grep_sort_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", printf_grep_sort_wc_pipe.as_str()],
        ),
        (
            "run printf grep sort head pipe",
            printf_grep_sort_head_pipe.clone(),
            "/bin/bash",
            vec!["-c", printf_grep_sort_head_pipe.as_str()],
        ),
        (
            "run printf grep sort tail pipe",
            printf_grep_sort_tail_pipe.clone(),
            "/bin/bash",
            vec!["-c", printf_grep_sort_tail_pipe.as_str()],
        ),
        (
            "run printf grep sort xargs echo pipe",
            printf_grep_sort_xargs_echo_pipe.clone(),
            "/bin/bash",
            vec!["-c", printf_grep_sort_xargs_echo_pipe.as_str()],
        ),
        (
            "run printf grep xargs echo pipe",
            printf_grep_xargs_echo_pipe.clone(),
            "/bin/bash",
            vec!["-c", printf_grep_xargs_echo_pipe.as_str()],
        ),
        (
            "run printf tr pipe",
            printf_tr_pipe.clone(),
            "/bin/bash",
            vec!["-c", printf_tr_pipe.as_str()],
        ),
        (
            "run printf sort pipe",
            printf_sort_pipe.clone(),
            "/bin/bash",
            vec!["-c", printf_sort_pipe.as_str()],
        ),
        (
            "run printf sort uniq pipe",
            printf_sort_uniq_pipe.clone(),
            "/bin/bash",
            vec!["-c", printf_sort_uniq_pipe.as_str()],
        ),
        (
            "run printf sort uniq wc pipe",
            printf_sort_uniq_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", printf_sort_uniq_wc_pipe.as_str()],
        ),
        (
            "run printf sort uniq head pipe",
            printf_sort_uniq_head_pipe.clone(),
            "/bin/bash",
            vec!["-c", printf_sort_uniq_head_pipe.as_str()],
        ),
        (
            "run printf sort uniq sort xargs echo pipe",
            printf_sort_uniq_sort_xargs_echo_pipe.clone(),
            "/bin/bash",
            vec!["-c", printf_sort_uniq_sort_xargs_echo_pipe.as_str()],
        ),
        (
            "run printf sort uniq xargs wc pipe",
            printf_sort_uniq_xargs_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", printf_sort_uniq_xargs_wc_pipe.as_str()],
        ),
        (
            "run printf sort wc pipe",
            printf_sort_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", printf_sort_wc_pipe.as_str()],
        ),
        (
            "run printf sort head pipe",
            printf_sort_head_pipe.clone(),
            "/bin/bash",
            vec!["-c", printf_sort_head_pipe.as_str()],
        ),
        (
            "run printf sort tail pipe",
            printf_sort_tail_pipe.clone(),
            "/bin/bash",
            vec!["-c", printf_sort_tail_pipe.as_str()],
        ),
        (
            "run printf sort xargs echo pipe",
            printf_sort_xargs_echo_pipe.clone(),
            "/bin/bash",
            vec!["-c", printf_sort_xargs_echo_pipe.as_str()],
        ),
        (
            "run printf sort xargs n1 pipe",
            printf_sort_xargs_n1_pipe.clone(),
            "/bin/bash",
            vec!["-c", printf_sort_xargs_n1_pipe.as_str()],
        ),
        (
            "run printf sort xargs n2 pipe",
            printf_sort_xargs_n2_pipe.clone(),
            "/bin/bash",
            vec!["-c", printf_sort_xargs_n2_pipe.as_str()],
        ),
        (
            "run printf sort xargs wc pipe",
            printf_sort_xargs_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", printf_sort_xargs_wc_pipe.as_str()],
        ),
        (
            "run printf xargs echo pipe",
            printf_xargs_echo_pipe.clone(),
            "/bin/bash",
            vec!["-c", printf_xargs_echo_pipe.as_str()],
        ),
        (
            "run printf xargs n1 pipe",
            printf_xargs_n1_pipe.clone(),
            "/bin/bash",
            vec!["-c", printf_xargs_n1_pipe.as_str()],
        ),
        (
            "run printf xargs n2 pipe",
            printf_xargs_n2_pipe.clone(),
            "/bin/bash",
            vec!["-c", printf_xargs_n2_pipe.as_str()],
        ),
        (
            "run printf xargs wc pipe",
            printf_xargs_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", printf_xargs_wc_pipe.as_str()],
        ),
        (
            "run seq wc pipe",
            seq_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", seq_wc_pipe.as_str()],
        ),
        (
            "run seq head pipe",
            seq_head_pipe.clone(),
            "/bin/bash",
            vec!["-c", seq_head_pipe.as_str()],
        ),
        (
            "run seq tail pipe",
            seq_tail_pipe.clone(),
            "/bin/bash",
            vec!["-c", seq_tail_pipe.as_str()],
        ),
        (
            "run seq sort pipe",
            seq_sort_pipe.clone(),
            "/bin/bash",
            vec!["-c", seq_sort_pipe.as_str()],
        ),
        (
            "run seq sort uniq pipe",
            seq_sort_uniq_pipe.clone(),
            "/bin/bash",
            vec!["-c", seq_sort_uniq_pipe.as_str()],
        ),
        (
            "run seq sort uniq wc pipe",
            seq_sort_uniq_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", seq_sort_uniq_wc_pipe.as_str()],
        ),
        (
            "run seq sort uniq head pipe",
            seq_sort_uniq_head_pipe.clone(),
            "/bin/bash",
            vec!["-c", seq_sort_uniq_head_pipe.as_str()],
        ),
        (
            "run seq sort uniq sort xargs echo pipe",
            seq_sort_uniq_sort_xargs_echo_pipe.clone(),
            "/bin/bash",
            vec!["-c", seq_sort_uniq_sort_xargs_echo_pipe.as_str()],
        ),
        (
            "run seq sort wc pipe",
            seq_sort_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", seq_sort_wc_pipe.as_str()],
        ),
        (
            "run seq sort head pipe",
            seq_sort_head_pipe.clone(),
            "/bin/bash",
            vec!["-c", seq_sort_head_pipe.as_str()],
        ),
        (
            "run seq sort tail pipe",
            seq_sort_tail_pipe.clone(),
            "/bin/bash",
            vec!["-c", seq_sort_tail_pipe.as_str()],
        ),
        (
            "run seq sort xargs echo pipe",
            seq_sort_xargs_echo_pipe.clone(),
            "/bin/bash",
            vec!["-c", seq_sort_xargs_echo_pipe.as_str()],
        ),
        (
            "run seq grep pipe",
            seq_grep_pipe.clone(),
            "/bin/bash",
            vec!["-c", seq_grep_pipe.as_str()],
        ),
        (
            "run seq grep wc pipe",
            seq_grep_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", seq_grep_wc_pipe.as_str()],
        ),
        (
            "run seq grep head pipe",
            seq_grep_head_pipe.clone(),
            "/bin/bash",
            vec!["-c", seq_grep_head_pipe.as_str()],
        ),
        (
            "run seq grep tail pipe",
            seq_grep_tail_pipe.clone(),
            "/bin/bash",
            vec!["-c", seq_grep_tail_pipe.as_str()],
        ),
        (
            "run seq grep sort pipe",
            seq_grep_sort_pipe.clone(),
            "/bin/bash",
            vec!["-c", seq_grep_sort_pipe.as_str()],
        ),
        (
            "run seq grep sort uniq pipe",
            seq_grep_sort_uniq_pipe.clone(),
            "/bin/bash",
            vec!["-c", seq_grep_sort_uniq_pipe.as_str()],
        ),
        (
            "run seq grep sort uniq wc pipe",
            seq_grep_sort_uniq_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", seq_grep_sort_uniq_wc_pipe.as_str()],
        ),
        (
            "run seq grep sort uniq head pipe",
            seq_grep_sort_uniq_head_pipe.clone(),
            "/bin/bash",
            vec!["-c", seq_grep_sort_uniq_head_pipe.as_str()],
        ),
        (
            "run seq grep sort uniq sort xargs echo pipe",
            seq_grep_sort_uniq_sort_xargs_echo_pipe.clone(),
            "/bin/bash",
            vec!["-c", seq_grep_sort_uniq_sort_xargs_echo_pipe.as_str()],
        ),
        (
            "run seq grep sort wc pipe",
            seq_grep_sort_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", seq_grep_sort_wc_pipe.as_str()],
        ),
        (
            "run seq grep sort head pipe",
            seq_grep_sort_head_pipe.clone(),
            "/bin/bash",
            vec!["-c", seq_grep_sort_head_pipe.as_str()],
        ),
        (
            "run seq grep sort tail pipe",
            seq_grep_sort_tail_pipe.clone(),
            "/bin/bash",
            vec!["-c", seq_grep_sort_tail_pipe.as_str()],
        ),
        (
            "run seq grep sort xargs echo pipe",
            seq_grep_sort_xargs_echo_pipe.clone(),
            "/bin/bash",
            vec!["-c", seq_grep_sort_xargs_echo_pipe.as_str()],
        ),
        (
            "run seq grep xargs echo pipe",
            seq_grep_xargs_echo_pipe.clone(),
            "/bin/bash",
            vec!["-c", seq_grep_xargs_echo_pipe.as_str()],
        ),
        (
            "run seq xargs echo pipe",
            seq_xargs_echo_pipe.clone(),
            "/bin/bash",
            vec!["-c", seq_xargs_echo_pipe.as_str()],
        ),
        (
            "run yes default head pipe",
            yes_default_head_pipe.clone(),
            "/bin/bash",
            vec!["-c", yes_default_head_pipe.as_str()],
        ),
        (
            "run yes head pipe",
            yes_head_pipe.clone(),
            "/bin/bash",
            vec!["-c", yes_head_pipe.as_str()],
        ),
        (
            "run true wc pipe",
            true_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", true_wc_pipe.as_str()],
        ),
        (
            "run false wc pipe",
            false_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", false_wc_pipe.as_str()],
        ),
        (
            "run false grep wc pipe",
            false_grep_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", false_grep_wc_pipe.as_str()],
        ),
        (
            "run true xargs echo pipe",
            true_xargs_echo_pipe.clone(),
            "/bin/bash",
            vec!["-c", true_xargs_echo_pipe.as_str()],
        ),
        (
            "run mkdir wc pipe",
            mkdir_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", mkdir_wc_pipe.as_str()],
        ),
        (
            "run mkdir xargs echo pipe",
            mkdir_xargs_echo_pipe.clone(),
            "/bin/bash",
            vec!["-c", mkdir_xargs_echo_pipe.as_str()],
        ),
        (
            "run touch wc pipe",
            touch_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", touch_wc_pipe.as_str()],
        ),
        (
            "run touch sort xargs echo pipe",
            touch_sort_xargs_echo_pipe.clone(),
            "/bin/bash",
            vec!["-c", touch_sort_xargs_echo_pipe.as_str()],
        ),
        (
            "run test wc pipe",
            test_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", test_wc_pipe.as_str()],
        ),
        (
            "run test xargs echo pipe",
            test_xargs_echo_pipe.clone(),
            "/bin/bash",
            vec!["-c", test_xargs_echo_pipe.as_str()],
        ),
        (
            "run bracket sort xargs echo pipe",
            bracket_sort_xargs_echo_pipe.clone(),
            "/bin/bash",
            vec!["-c", bracket_sort_xargs_echo_pipe.as_str()],
        ),
        (
            "run test grep wc pipe",
            test_grep_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", test_grep_wc_pipe.as_str()],
        ),
        (
            "run wc xargs echo pipe",
            wc_xargs_echo_pipe.clone(),
            "/bin/bash",
            vec!["-c", wc_xargs_echo_pipe.as_str()],
        ),
        (
            "run wc multi wc pipe",
            wc_multi_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", wc_multi_wc_pipe.as_str()],
        ),
        (
            "run wc grep wc pipe",
            wc_grep_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", wc_grep_wc_pipe.as_str()],
        ),
        (
            "run wc sort xargs echo pipe",
            wc_sort_xargs_echo_pipe.clone(),
            "/bin/bash",
            vec!["-c", wc_sort_xargs_echo_pipe.as_str()],
        ),
        (
            "run du wc pipe",
            du_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", du_wc_pipe.as_str()],
        ),
        (
            "run du xargs echo pipe",
            du_xargs_echo_pipe.clone(),
            "/bin/bash",
            vec!["-c", du_xargs_echo_pipe.as_str()],
        ),
        (
            "run du grep wc pipe",
            du_grep_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", du_grep_wc_pipe.as_str()],
        ),
        (
            "run which wc pipe",
            which_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", which_wc_pipe.as_str()],
        ),
        (
            "run which head pipe",
            which_head_pipe.clone(),
            "/bin/bash",
            vec!["-c", which_head_pipe.as_str()],
        ),
        (
            "run which tail pipe",
            which_tail_pipe.clone(),
            "/bin/bash",
            vec!["-c", which_tail_pipe.as_str()],
        ),
        (
            "run which grep wc pipe",
            which_grep_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", which_grep_wc_pipe.as_str()],
        ),
        (
            "run which grep xargs pipe",
            which_grep_xargs_pipe.clone(),
            "/bin/bash",
            vec!["-c", which_grep_xargs_pipe.as_str()],
        ),
        (
            "run which xargs pipe",
            which_xargs_pipe.clone(),
            "/bin/bash",
            vec!["-c", which_xargs_pipe.as_str()],
        ),
        (
            "run which sort wc pipe",
            which_sort_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", which_sort_wc_pipe.as_str()],
        ),
        (
            "run which sort xargs pipe",
            which_sort_xargs_pipe.clone(),
            "/bin/bash",
            vec!["-c", which_sort_xargs_pipe.as_str()],
        ),
        (
            "run which all wc pipe",
            which_all_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", which_all_wc_pipe.as_str()],
        ),
        (
            "run which all head pipe",
            which_all_head_pipe.clone(),
            "/bin/bash",
            vec!["-c", which_all_head_pipe.as_str()],
        ),
        (
            "run which all grep wc pipe",
            which_all_grep_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", which_all_grep_wc_pipe.as_str()],
        ),
        (
            "run which all xargs pipe",
            which_all_xargs_pipe.clone(),
            "/bin/bash",
            vec!["-c", which_all_xargs_pipe.as_str()],
        ),
        (
            "run which all sort xargs pipe",
            which_all_sort_xargs_pipe.clone(),
            "/bin/bash",
            vec!["-c", which_all_sort_xargs_pipe.as_str()],
        ),
        (
            "run command v wc pipe",
            command_v_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", command_v_wc_pipe.as_str()],
        ),
        (
            "run command v head pipe",
            command_v_head_pipe.clone(),
            "/bin/bash",
            vec!["-c", command_v_head_pipe.as_str()],
        ),
        (
            "run command v tail pipe",
            command_v_tail_pipe.clone(),
            "/bin/bash",
            vec!["-c", command_v_tail_pipe.as_str()],
        ),
        (
            "run command v grep wc pipe",
            command_v_grep_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", command_v_grep_wc_pipe.as_str()],
        ),
        (
            "run command v grep head pipe",
            command_v_grep_head_pipe.clone(),
            "/bin/bash",
            vec!["-c", command_v_grep_head_pipe.as_str()],
        ),
        (
            "run command v xargs pipe",
            command_v_xargs_pipe.clone(),
            "/bin/bash",
            vec!["-c", command_v_xargs_pipe.as_str()],
        ),
        (
            "run command v sort wc pipe",
            command_v_sort_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", command_v_sort_wc_pipe.as_str()],
        ),
        (
            "run command v sort xargs pipe",
            command_v_sort_xargs_pipe.clone(),
            "/bin/bash",
            vec!["-c", command_v_sort_xargs_pipe.as_str()],
        ),
        ("run env", "env".to_string(), "/usr/bin/env", vec![]),
        (
            "run printenv",
            "printenv".to_string(),
            "/usr/bin/printenv",
            vec![],
        ),
        (
            "run printenv path",
            "printenv PATH".to_string(),
            "/usr/bin/printenv",
            vec!["PATH"],
        ),
        (
            "run printenv path wc pipe",
            printenv_path_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", printenv_path_wc_pipe.as_str()],
        ),
        (
            "run printenv path head pipe",
            printenv_path_head_pipe.clone(),
            "/bin/bash",
            vec!["-c", printenv_path_head_pipe.as_str()],
        ),
        (
            "run printenv path tail pipe",
            printenv_path_tail_pipe.clone(),
            "/bin/bash",
            vec!["-c", printenv_path_tail_pipe.as_str()],
        ),
        (
            "run printenv path grep pipe",
            printenv_path_grep_pipe.clone(),
            "/bin/bash",
            vec!["-c", printenv_path_grep_pipe.as_str()],
        ),
        (
            "run printenv path grep wc pipe",
            printenv_path_grep_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", printenv_path_grep_wc_pipe.as_str()],
        ),
        (
            "run printenv path grep head pipe",
            printenv_path_grep_head_pipe.clone(),
            "/bin/bash",
            vec!["-c", printenv_path_grep_head_pipe.as_str()],
        ),
        (
            "run printenv path grep sort pipe",
            printenv_path_grep_sort_pipe.clone(),
            "/bin/bash",
            vec!["-c", printenv_path_grep_sort_pipe.as_str()],
        ),
        (
            "run printenv path grep xargs pipe",
            printenv_path_grep_xargs_pipe.clone(),
            "/bin/bash",
            vec!["-c", printenv_path_grep_xargs_pipe.as_str()],
        ),
        (
            "run printenv path sort pipe",
            printenv_path_sort_pipe.clone(),
            "/bin/bash",
            vec!["-c", printenv_path_sort_pipe.as_str()],
        ),
        (
            "run printenv path xargs pipe",
            printenv_path_xargs_pipe.clone(),
            "/bin/bash",
            vec!["-c", printenv_path_xargs_pipe.as_str()],
        ),
        (
            "run printenv path sort xargs pipe",
            printenv_path_sort_xargs_pipe.clone(),
            "/bin/bash",
            vec!["-c", printenv_path_sort_xargs_pipe.as_str()],
        ),
        (
            "run hostname wc pipe",
            hostname_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", hostname_wc_pipe.as_str()],
        ),
        (
            "run hostname head pipe",
            hostname_head_pipe.clone(),
            "/bin/bash",
            vec!["-c", hostname_head_pipe.as_str()],
        ),
        (
            "run hostname tail pipe",
            hostname_tail_pipe.clone(),
            "/bin/bash",
            vec!["-c", hostname_tail_pipe.as_str()],
        ),
        (
            "run hostname grep pipe",
            hostname_grep_pipe.clone(),
            "/bin/bash",
            vec!["-c", hostname_grep_pipe.as_str()],
        ),
        (
            "run hostname grep wc pipe",
            hostname_grep_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", hostname_grep_wc_pipe.as_str()],
        ),
        (
            "run hostname grep xargs pipe",
            hostname_grep_xargs_pipe.clone(),
            "/bin/bash",
            vec!["-c", hostname_grep_xargs_pipe.as_str()],
        ),
        (
            "run hostname sort pipe",
            hostname_sort_pipe.clone(),
            "/bin/bash",
            vec!["-c", hostname_sort_pipe.as_str()],
        ),
        (
            "run hostname xargs pipe",
            hostname_xargs_pipe.clone(),
            "/bin/bash",
            vec!["-c", hostname_xargs_pipe.as_str()],
        ),
        (
            "run hostname sort xargs pipe",
            hostname_sort_xargs_pipe.clone(),
            "/bin/bash",
            vec!["-c", hostname_sort_xargs_pipe.as_str()],
        ),
        (
            "run pwd wc pipe",
            pwd_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", pwd_wc_pipe.as_str()],
        ),
        (
            "run pwd grep wc pipe",
            pwd_grep_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", pwd_grep_wc_pipe.as_str()],
        ),
        (
            "run pwd xargs echo pipe",
            pwd_xargs_echo_pipe.clone(),
            "/bin/bash",
            vec!["-c", pwd_xargs_echo_pipe.as_str()],
        ),
        (
            "run basename wc pipe",
            basename_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", basename_wc_pipe.as_str()],
        ),
        (
            "run basename grep xargs pipe",
            basename_grep_xargs_pipe.clone(),
            "/bin/bash",
            vec!["-c", basename_grep_xargs_pipe.as_str()],
        ),
        (
            "run dirname sort tail pipe",
            dirname_sort_tail_pipe.clone(),
            "/bin/bash",
            vec!["-c", dirname_sort_tail_pipe.as_str()],
        ),
        (
            "run whoami wc pipe",
            whoami_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", whoami_wc_pipe.as_str()],
        ),
        (
            "run id wc pipe",
            id_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", id_wc_pipe.as_str()],
        ),
        (
            "run id grep xargs pipe",
            id_grep_xargs_pipe.clone(),
            "/bin/bash",
            vec!["-c", id_grep_xargs_pipe.as_str()],
        ),
        (
            "run id u head pipe",
            id_u_head_pipe.clone(),
            "/bin/bash",
            vec!["-c", id_u_head_pipe.as_str()],
        ),
        (
            "run id un xargs pipe",
            id_un_xargs_pipe.clone(),
            "/bin/bash",
            vec!["-c", id_un_xargs_pipe.as_str()],
        ),
        (
            "run id G wc words pipe",
            id_groups_wc_words_pipe.clone(),
            "/bin/bash",
            vec!["-c", id_groups_wc_words_pipe.as_str()],
        ),
        (
            "run id Gn sort xargs pipe",
            id_group_names_sort_xargs_pipe.clone(),
            "/bin/bash",
            vec!["-c", id_group_names_sort_xargs_pipe.as_str()],
        ),
        (
            "run uname m sort pipe",
            uname_m_sort_pipe.clone(),
            "/bin/bash",
            vec!["-c", uname_m_sort_pipe.as_str()],
        ),
        (
            "run uname p xargs pipe",
            uname_p_xargs_pipe.clone(),
            "/bin/bash",
            vec!["-c", uname_p_xargs_pipe.as_str()],
        ),
        (
            "run uname a xargs pipe",
            uname_a_xargs_pipe.clone(),
            "/bin/bash",
            vec!["-c", uname_a_xargs_pipe.as_str()],
        ),
        (
            "run sed wc pipe",
            sed_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", sed_wc_pipe.as_str()],
        ),
        (
            "run sed head pipe",
            sed_head_pipe.clone(),
            "/bin/bash",
            vec!["-c", sed_head_pipe.as_str()],
        ),
        (
            "run sed tail pipe",
            sed_tail_pipe.clone(),
            "/bin/bash",
            vec!["-c", sed_tail_pipe.as_str()],
        ),
        (
            "run sed sort pipe",
            sed_sort_pipe.clone(),
            "/bin/bash",
            vec!["-c", sed_sort_pipe.as_str()],
        ),
        (
            "run sed sort uniq pipe",
            sed_sort_uniq_pipe.clone(),
            "/bin/bash",
            vec!["-c", sed_sort_uniq_pipe.as_str()],
        ),
        (
            "run sed sort uniq wc pipe",
            sed_sort_uniq_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", sed_sort_uniq_wc_pipe.as_str()],
        ),
        (
            "run sed sort wc pipe",
            sed_sort_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", sed_sort_wc_pipe.as_str()],
        ),
        (
            "run sed sort head pipe",
            sed_sort_head_pipe.clone(),
            "/bin/bash",
            vec!["-c", sed_sort_head_pipe.as_str()],
        ),
        (
            "run sed sort tail pipe",
            sed_sort_tail_pipe.clone(),
            "/bin/bash",
            vec!["-c", sed_sort_tail_pipe.as_str()],
        ),
        (
            "run sed sort xargs echo pipe",
            sed_sort_xargs_echo_pipe.clone(),
            "/bin/bash",
            vec!["-c", sed_sort_xargs_echo_pipe.as_str()],
        ),
        (
            "run sed sort xargs wc pipe",
            sed_sort_xargs_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", sed_sort_xargs_wc_pipe.as_str()],
        ),
        (
            "run sed xargs echo pipe",
            sed_xargs_echo_pipe.clone(),
            "/bin/bash",
            vec!["-c", sed_xargs_echo_pipe.as_str()],
        ),
        (
            "run sed xargs wc pipe",
            sed_xargs_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", sed_xargs_wc_pipe.as_str()],
        ),
        (
            "run sed grep pipe",
            sed_grep_pipe.clone(),
            "/bin/bash",
            vec!["-c", sed_grep_pipe.as_str()],
        ),
        (
            "run sed grep wc pipe",
            sed_grep_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", sed_grep_wc_pipe.as_str()],
        ),
        (
            "run sed grep head pipe",
            sed_grep_head_pipe.clone(),
            "/bin/bash",
            vec!["-c", sed_grep_head_pipe.as_str()],
        ),
        (
            "run sed grep tail pipe",
            sed_grep_tail_pipe.clone(),
            "/bin/bash",
            vec!["-c", sed_grep_tail_pipe.as_str()],
        ),
        (
            "run sed grep sort pipe",
            sed_grep_sort_pipe.clone(),
            "/bin/bash",
            vec!["-c", sed_grep_sort_pipe.as_str()],
        ),
        (
            "run sed grep sort uniq pipe",
            sed_grep_sort_uniq_pipe.clone(),
            "/bin/bash",
            vec!["-c", sed_grep_sort_uniq_pipe.as_str()],
        ),
        (
            "run sed grep sort uniq wc pipe",
            sed_grep_sort_uniq_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", sed_grep_sort_uniq_wc_pipe.as_str()],
        ),
        (
            "run sed grep sort wc pipe",
            sed_grep_sort_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", sed_grep_sort_wc_pipe.as_str()],
        ),
        (
            "run sed grep sort head pipe",
            sed_grep_sort_head_pipe.clone(),
            "/bin/bash",
            vec!["-c", sed_grep_sort_head_pipe.as_str()],
        ),
        (
            "run sed grep sort tail pipe",
            sed_grep_sort_tail_pipe.clone(),
            "/bin/bash",
            vec!["-c", sed_grep_sort_tail_pipe.as_str()],
        ),
        (
            "run sed grep sort xargs echo pipe",
            sed_grep_sort_xargs_echo_pipe.clone(),
            "/bin/bash",
            vec!["-c", sed_grep_sort_xargs_echo_pipe.as_str()],
        ),
        (
            "run sed grep sort xargs wc pipe",
            sed_grep_sort_xargs_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", sed_grep_sort_xargs_wc_pipe.as_str()],
        ),
        (
            "run sed grep xargs echo pipe",
            sed_grep_xargs_echo_pipe.clone(),
            "/bin/bash",
            vec!["-c", sed_grep_xargs_echo_pipe.as_str()],
        ),
        (
            "run sed grep xargs wc pipe",
            sed_grep_xargs_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", sed_grep_xargs_wc_pipe.as_str()],
        ),
        (
            "run cat sed pipe",
            cat_sed_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_sed_pipe.as_str()],
        ),
        (
            "run cat sed wc pipe",
            cat_sed_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_sed_wc_pipe.as_str()],
        ),
        (
            "run cat sed head pipe",
            cat_sed_head_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_sed_head_pipe.as_str()],
        ),
        (
            "run cat sed tail pipe",
            cat_sed_tail_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_sed_tail_pipe.as_str()],
        ),
        (
            "run cat sed sort pipe",
            cat_sed_sort_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_sed_sort_pipe.as_str()],
        ),
        (
            "run cat sed sort uniq pipe",
            cat_sed_sort_uniq_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_sed_sort_uniq_pipe.as_str()],
        ),
        (
            "run cat sed sort uniq wc pipe",
            cat_sed_sort_uniq_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_sed_sort_uniq_wc_pipe.as_str()],
        ),
        (
            "run cat sed sort xargs echo pipe",
            cat_sed_sort_xargs_echo_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_sed_sort_xargs_echo_pipe.as_str()],
        ),
        (
            "run cat sed sort xargs wc pipe",
            cat_sed_sort_xargs_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_sed_sort_xargs_wc_pipe.as_str()],
        ),
        (
            "run cat sed xargs echo pipe",
            cat_sed_xargs_echo_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_sed_xargs_echo_pipe.as_str()],
        ),
        (
            "run cat sed xargs wc pipe",
            cat_sed_xargs_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_sed_xargs_wc_pipe.as_str()],
        ),
        (
            "run cat sed grep pipe",
            cat_sed_grep_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_sed_grep_pipe.as_str()],
        ),
        (
            "run cat sed grep wc pipe",
            cat_sed_grep_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_sed_grep_wc_pipe.as_str()],
        ),
        (
            "run cat sed grep sort uniq wc pipe",
            cat_sed_grep_sort_uniq_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_sed_grep_sort_uniq_wc_pipe.as_str()],
        ),
        (
            "run cat sed grep sort xargs echo pipe",
            cat_sed_grep_sort_xargs_echo_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_sed_grep_sort_xargs_echo_pipe.as_str()],
        ),
        (
            "run cat sed grep xargs wc pipe",
            cat_sed_grep_xargs_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_sed_grep_xargs_wc_pipe.as_str()],
        ),
        (
            "run cut wc pipe",
            cut_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", cut_wc_pipe.as_str()],
        ),
        (
            "run cut head pipe",
            cut_head_pipe.clone(),
            "/bin/bash",
            vec!["-c", cut_head_pipe.as_str()],
        ),
        (
            "run cut tail pipe",
            cut_tail_pipe.clone(),
            "/bin/bash",
            vec!["-c", cut_tail_pipe.as_str()],
        ),
        (
            "run cut sort pipe",
            cut_sort_pipe.clone(),
            "/bin/bash",
            vec!["-c", cut_sort_pipe.as_str()],
        ),
        (
            "run cut sort uniq pipe",
            cut_sort_uniq_pipe.clone(),
            "/bin/bash",
            vec!["-c", cut_sort_uniq_pipe.as_str()],
        ),
        (
            "run cut sort uniq wc pipe",
            cut_sort_uniq_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", cut_sort_uniq_wc_pipe.as_str()],
        ),
        (
            "run cut sort wc pipe",
            cut_sort_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", cut_sort_wc_pipe.as_str()],
        ),
        (
            "run cut sort head pipe",
            cut_sort_head_pipe.clone(),
            "/bin/bash",
            vec!["-c", cut_sort_head_pipe.as_str()],
        ),
        (
            "run cut sort tail pipe",
            cut_sort_tail_pipe.clone(),
            "/bin/bash",
            vec!["-c", cut_sort_tail_pipe.as_str()],
        ),
        (
            "run cut sort xargs echo pipe",
            cut_sort_xargs_echo_pipe.clone(),
            "/bin/bash",
            vec!["-c", cut_sort_xargs_echo_pipe.as_str()],
        ),
        (
            "run cut sort xargs wc pipe",
            cut_sort_xargs_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", cut_sort_xargs_wc_pipe.as_str()],
        ),
        (
            "run cut xargs echo pipe",
            cut_xargs_echo_pipe.clone(),
            "/bin/bash",
            vec!["-c", cut_xargs_echo_pipe.as_str()],
        ),
        (
            "run cut xargs wc pipe",
            cut_xargs_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", cut_xargs_wc_pipe.as_str()],
        ),
        (
            "run cut grep pipe",
            cut_grep_pipe.clone(),
            "/bin/bash",
            vec!["-c", cut_grep_pipe.as_str()],
        ),
        (
            "run cut grep wc pipe",
            cut_grep_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", cut_grep_wc_pipe.as_str()],
        ),
        (
            "run cut grep head pipe",
            cut_grep_head_pipe.clone(),
            "/bin/bash",
            vec!["-c", cut_grep_head_pipe.as_str()],
        ),
        (
            "run cut grep tail pipe",
            cut_grep_tail_pipe.clone(),
            "/bin/bash",
            vec!["-c", cut_grep_tail_pipe.as_str()],
        ),
        (
            "run cut grep sort pipe",
            cut_grep_sort_pipe.clone(),
            "/bin/bash",
            vec!["-c", cut_grep_sort_pipe.as_str()],
        ),
        (
            "run cut grep sort uniq pipe",
            cut_grep_sort_uniq_pipe.clone(),
            "/bin/bash",
            vec!["-c", cut_grep_sort_uniq_pipe.as_str()],
        ),
        (
            "run cut grep sort uniq wc pipe",
            cut_grep_sort_uniq_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", cut_grep_sort_uniq_wc_pipe.as_str()],
        ),
        (
            "run cut grep sort wc pipe",
            cut_grep_sort_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", cut_grep_sort_wc_pipe.as_str()],
        ),
        (
            "run cut grep sort head pipe",
            cut_grep_sort_head_pipe.clone(),
            "/bin/bash",
            vec!["-c", cut_grep_sort_head_pipe.as_str()],
        ),
        (
            "run cut grep sort tail pipe",
            cut_grep_sort_tail_pipe.clone(),
            "/bin/bash",
            vec!["-c", cut_grep_sort_tail_pipe.as_str()],
        ),
        (
            "run cut grep sort xargs echo pipe",
            cut_grep_sort_xargs_echo_pipe.clone(),
            "/bin/bash",
            vec!["-c", cut_grep_sort_xargs_echo_pipe.as_str()],
        ),
        (
            "run cut grep sort xargs wc pipe",
            cut_grep_sort_xargs_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", cut_grep_sort_xargs_wc_pipe.as_str()],
        ),
        (
            "run cut grep xargs echo pipe",
            cut_grep_xargs_echo_pipe.clone(),
            "/bin/bash",
            vec!["-c", cut_grep_xargs_echo_pipe.as_str()],
        ),
        (
            "run cut grep xargs wc pipe",
            cut_grep_xargs_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", cut_grep_xargs_wc_pipe.as_str()],
        ),
        (
            "run ls wc pipe",
            ls_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", ls_wc_pipe.as_str()],
        ),
        (
            "run ls head pipe",
            ls_head_pipe.clone(),
            "/bin/bash",
            vec!["-c", ls_head_pipe.as_str()],
        ),
        (
            "run ls tail pipe",
            ls_tail_pipe.clone(),
            "/bin/bash",
            vec!["-c", ls_tail_pipe.as_str()],
        ),
        (
            "run ls sort pipe",
            ls_sort_pipe.clone(),
            "/bin/bash",
            vec!["-c", ls_sort_pipe.as_str()],
        ),
        (
            "run ls sort uniq pipe",
            ls_sort_uniq_pipe.clone(),
            "/bin/bash",
            vec!["-c", ls_sort_uniq_pipe.as_str()],
        ),
        (
            "run ls sort uniq wc pipe",
            ls_sort_uniq_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", ls_sort_uniq_wc_pipe.as_str()],
        ),
        (
            "run ls sort uniq head pipe",
            ls_sort_uniq_head_pipe.clone(),
            "/bin/bash",
            vec!["-c", ls_sort_uniq_head_pipe.as_str()],
        ),
        (
            "run ls sort uniq sort uniq wc pipe",
            ls_sort_uniq_sort_uniq_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", ls_sort_uniq_sort_uniq_wc_pipe.as_str()],
        ),
        (
            "run ls sort uniq xargs echo pipe",
            ls_sort_uniq_xargs_echo_pipe.clone(),
            "/bin/bash",
            vec!["-c", ls_sort_uniq_xargs_echo_pipe.as_str()],
        ),
        (
            "run ls sort uniq grep pipe",
            ls_sort_uniq_grep_pipe.clone(),
            "/bin/bash",
            vec!["-c", ls_sort_uniq_grep_pipe.as_str()],
        ),
        (
            "run ls sort uniq grep sort xargs echo pipe",
            ls_sort_uniq_grep_sort_xargs_echo_pipe.clone(),
            "/bin/bash",
            vec!["-c", ls_sort_uniq_grep_sort_xargs_echo_pipe.as_str()],
        ),
        (
            "run ls sort wc pipe",
            ls_sort_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", ls_sort_wc_pipe.as_str()],
        ),
        (
            "run ls sort head pipe",
            ls_sort_head_pipe.clone(),
            "/bin/bash",
            vec!["-c", ls_sort_head_pipe.as_str()],
        ),
        (
            "run ls sort tail pipe",
            ls_sort_tail_pipe.clone(),
            "/bin/bash",
            vec!["-c", ls_sort_tail_pipe.as_str()],
        ),
        (
            "run ls sort xargs echo pipe",
            ls_sort_xargs_echo_pipe.clone(),
            "/bin/bash",
            vec!["-c", ls_sort_xargs_echo_pipe.as_str()],
        ),
        (
            "run ls grep pipe",
            ls_grep_pipe.clone(),
            "/bin/bash",
            vec!["-c", ls_grep_pipe.as_str()],
        ),
        (
            "run ls grep wc pipe",
            ls_grep_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", ls_grep_wc_pipe.as_str()],
        ),
        (
            "run ls grep head pipe",
            ls_grep_head_pipe.clone(),
            "/bin/bash",
            vec!["-c", ls_grep_head_pipe.as_str()],
        ),
        (
            "run ls grep tail pipe",
            ls_grep_tail_pipe.clone(),
            "/bin/bash",
            vec!["-c", ls_grep_tail_pipe.as_str()],
        ),
        (
            "run ls grep sort pipe",
            ls_grep_sort_pipe.clone(),
            "/bin/bash",
            vec!["-c", ls_grep_sort_pipe.as_str()],
        ),
        (
            "run ls grep sort uniq wc pipe",
            ls_grep_sort_uniq_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", ls_grep_sort_uniq_wc_pipe.as_str()],
        ),
        (
            "run ls grep xargs echo pipe",
            ls_grep_xargs_echo_pipe.clone(),
            "/bin/bash",
            vec!["-c", ls_grep_xargs_echo_pipe.as_str()],
        ),
        (
            "run ls grep sort xargs echo pipe",
            ls_grep_sort_xargs_echo_pipe.clone(),
            "/bin/bash",
            vec!["-c", ls_grep_sort_xargs_echo_pipe.as_str()],
        ),
        (
            "run ls xargs echo pipe",
            ls_xargs_echo_pipe.clone(),
            "/bin/bash",
            vec!["-c", ls_xargs_echo_pipe.as_str()],
        ),
        (
            "run ls -a wc pipe",
            ls_all_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", ls_all_wc_pipe.as_str()],
        ),
        (
            "run ls -a grep wc pipe",
            ls_all_grep_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", ls_all_grep_wc_pipe.as_str()],
        ),
        (
            "run ls -a sort tail pipe",
            ls_all_sort_tail_pipe.clone(),
            "/bin/bash",
            vec!["-c", ls_all_sort_tail_pipe.as_str()],
        ),
        (
            "run ls -a xargs echo pipe",
            ls_all_xargs_echo_pipe.clone(),
            "/bin/bash",
            vec!["-c", ls_all_xargs_echo_pipe.as_str()],
        ),
        (
            "run ls -a sort xargs echo pipe",
            ls_all_sort_xargs_echo_pipe.clone(),
            "/bin/bash",
            vec!["-c", ls_all_sort_xargs_echo_pipe.as_str()],
        ),
        (
            "run ls -A wc pipe",
            ls_almost_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", ls_almost_wc_pipe.as_str()],
        ),
        (
            "run ls -A grep wc pipe",
            ls_almost_grep_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", ls_almost_grep_wc_pipe.as_str()],
        ),
        (
            "run ls -A sort tail pipe",
            ls_almost_sort_tail_pipe.clone(),
            "/bin/bash",
            vec!["-c", ls_almost_sort_tail_pipe.as_str()],
        ),
        (
            "run ls -A xargs echo pipe",
            ls_almost_xargs_echo_pipe.clone(),
            "/bin/bash",
            vec!["-c", ls_almost_xargs_echo_pipe.as_str()],
        ),
        (
            "run ls -A sort xargs echo pipe",
            ls_almost_sort_xargs_echo_pipe.clone(),
            "/bin/bash",
            vec!["-c", ls_almost_sort_xargs_echo_pipe.as_str()],
        ),
        (
            "run sort uniq pipe",
            sort_uniq_pipe.clone(),
            "/bin/bash",
            vec!["-c", sort_uniq_pipe.as_str()],
        ),
        (
            "run sort uniq wc pipe",
            sort_uniq_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", sort_uniq_wc_pipe.as_str()],
        ),
        (
            "run sort uniq head pipe",
            sort_uniq_head_pipe.clone(),
            "/bin/bash",
            vec!["-c", sort_uniq_head_pipe.as_str()],
        ),
        (
            "run sort uniq sort uniq wc pipe",
            sort_uniq_sort_uniq_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", sort_uniq_sort_uniq_wc_pipe.as_str()],
        ),
        (
            "run sort uniq xargs wc pipe",
            sort_uniq_xargs_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", sort_uniq_xargs_wc_pipe.as_str()],
        ),
        (
            "run sort uniq grep pipe",
            sort_uniq_grep_pipe.clone(),
            "/bin/bash",
            vec!["-c", sort_uniq_grep_pipe.as_str()],
        ),
        (
            "run sort uniq grep sort xargs echo pipe",
            sort_uniq_grep_sort_xargs_echo_pipe.clone(),
            "/bin/bash",
            vec!["-c", sort_uniq_grep_sort_xargs_echo_pipe.as_str()],
        ),
        (
            "run sort uniq grep xargs wc pipe",
            sort_uniq_grep_xargs_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", sort_uniq_grep_xargs_wc_pipe.as_str()],
        ),
        (
            "run sort grep pipe",
            sort_grep_pipe.clone(),
            "/bin/bash",
            vec!["-c", sort_grep_pipe.as_str()],
        ),
        (
            "run sort grep wc pipe",
            sort_grep_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", sort_grep_wc_pipe.as_str()],
        ),
        (
            "run sort grep sort xargs echo pipe",
            sort_grep_sort_xargs_echo_pipe.clone(),
            "/bin/bash",
            vec!["-c", sort_grep_sort_xargs_echo_pipe.as_str()],
        ),
        (
            "run sort grep xargs wc pipe",
            sort_grep_xargs_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", sort_grep_xargs_wc_pipe.as_str()],
        ),
        (
            "run sort head pipe",
            sort_head_pipe.clone(),
            "/bin/bash",
            vec!["-c", sort_head_pipe.as_str()],
        ),
        (
            "run sort tail pipe",
            sort_tail_pipe.clone(),
            "/bin/bash",
            vec!["-c", sort_tail_pipe.as_str()],
        ),
        (
            "run sort wc pipe",
            sort_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", sort_wc_pipe.as_str()],
        ),
        (
            "run head wc pipe",
            head_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", head_wc_pipe.as_str()],
        ),
        (
            "run head no-newline wc pipe",
            head_no_newline_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", head_no_newline_wc_pipe.as_str()],
        ),
        (
            "run head head pipe",
            head_head_pipe.clone(),
            "/bin/bash",
            vec!["-c", head_head_pipe.as_str()],
        ),
        (
            "run head tail pipe",
            head_tail_pipe.clone(),
            "/bin/bash",
            vec!["-c", head_tail_pipe.as_str()],
        ),
        (
            "run head sort pipe",
            head_sort_pipe.clone(),
            "/bin/bash",
            vec!["-c", head_sort_pipe.as_str()],
        ),
        (
            "run head sort uniq pipe",
            head_sort_uniq_pipe.clone(),
            "/bin/bash",
            vec!["-c", head_sort_uniq_pipe.as_str()],
        ),
        (
            "run head sort uniq wc pipe",
            head_sort_uniq_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", head_sort_uniq_wc_pipe.as_str()],
        ),
        (
            "run head sort wc pipe",
            head_sort_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", head_sort_wc_pipe.as_str()],
        ),
        (
            "run head sort head pipe",
            head_sort_head_pipe.clone(),
            "/bin/bash",
            vec!["-c", head_sort_head_pipe.as_str()],
        ),
        (
            "run head sort tail pipe",
            head_sort_tail_pipe.clone(),
            "/bin/bash",
            vec!["-c", head_sort_tail_pipe.as_str()],
        ),
        (
            "run head sort xargs echo pipe",
            head_sort_xargs_echo_pipe.clone(),
            "/bin/bash",
            vec!["-c", head_sort_xargs_echo_pipe.as_str()],
        ),
        (
            "run head sort xargs wc pipe",
            head_sort_xargs_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", head_sort_xargs_wc_pipe.as_str()],
        ),
        (
            "run head xargs echo pipe",
            head_xargs_echo_pipe.clone(),
            "/bin/bash",
            vec!["-c", head_xargs_echo_pipe.as_str()],
        ),
        (
            "run head xargs wc pipe",
            head_xargs_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", head_xargs_wc_pipe.as_str()],
        ),
        (
            "run head grep pipe",
            head_grep_pipe.clone(),
            "/bin/bash",
            vec!["-c", head_grep_pipe.as_str()],
        ),
        (
            "run head grep no-newline pipe",
            head_grep_no_newline_pipe.clone(),
            "/bin/bash",
            vec!["-c", head_grep_no_newline_pipe.as_str()],
        ),
        (
            "run head grep wc pipe",
            head_grep_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", head_grep_wc_pipe.as_str()],
        ),
        (
            "run head grep head pipe",
            head_grep_head_pipe.clone(),
            "/bin/bash",
            vec!["-c", head_grep_head_pipe.as_str()],
        ),
        (
            "run head grep tail pipe",
            head_grep_tail_pipe.clone(),
            "/bin/bash",
            vec!["-c", head_grep_tail_pipe.as_str()],
        ),
        (
            "run head grep sort pipe",
            head_grep_sort_pipe.clone(),
            "/bin/bash",
            vec!["-c", head_grep_sort_pipe.as_str()],
        ),
        (
            "run head grep sort uniq pipe",
            head_grep_sort_uniq_pipe.clone(),
            "/bin/bash",
            vec!["-c", head_grep_sort_uniq_pipe.as_str()],
        ),
        (
            "run head grep sort uniq wc pipe",
            head_grep_sort_uniq_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", head_grep_sort_uniq_wc_pipe.as_str()],
        ),
        (
            "run head grep sort wc pipe",
            head_grep_sort_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", head_grep_sort_wc_pipe.as_str()],
        ),
        (
            "run head grep sort head pipe",
            head_grep_sort_head_pipe.clone(),
            "/bin/bash",
            vec!["-c", head_grep_sort_head_pipe.as_str()],
        ),
        (
            "run head grep sort tail pipe",
            head_grep_sort_tail_pipe.clone(),
            "/bin/bash",
            vec!["-c", head_grep_sort_tail_pipe.as_str()],
        ),
        (
            "run head grep sort xargs echo pipe",
            head_grep_sort_xargs_echo_pipe.clone(),
            "/bin/bash",
            vec!["-c", head_grep_sort_xargs_echo_pipe.as_str()],
        ),
        (
            "run head grep sort xargs wc pipe",
            head_grep_sort_xargs_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", head_grep_sort_xargs_wc_pipe.as_str()],
        ),
        (
            "run head grep xargs echo pipe",
            head_grep_xargs_echo_pipe.clone(),
            "/bin/bash",
            vec!["-c", head_grep_xargs_echo_pipe.as_str()],
        ),
        (
            "run head grep xargs wc pipe",
            head_grep_xargs_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", head_grep_xargs_wc_pipe.as_str()],
        ),
        (
            "run tail wc pipe",
            tail_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", tail_wc_pipe.as_str()],
        ),
        (
            "run tail no-newline wc pipe",
            tail_no_newline_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", tail_no_newline_wc_pipe.as_str()],
        ),
        (
            "run tail zero wc pipe",
            tail_zero_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", tail_zero_wc_pipe.as_str()],
        ),
        (
            "run tail head pipe",
            tail_head_pipe.clone(),
            "/bin/bash",
            vec!["-c", tail_head_pipe.as_str()],
        ),
        (
            "run tail tail pipe",
            tail_tail_pipe.clone(),
            "/bin/bash",
            vec!["-c", tail_tail_pipe.as_str()],
        ),
        (
            "run tail sort pipe",
            tail_sort_pipe.clone(),
            "/bin/bash",
            vec!["-c", tail_sort_pipe.as_str()],
        ),
        (
            "run tail sort uniq pipe",
            tail_sort_uniq_pipe.clone(),
            "/bin/bash",
            vec!["-c", tail_sort_uniq_pipe.as_str()],
        ),
        (
            "run tail sort uniq wc pipe",
            tail_sort_uniq_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", tail_sort_uniq_wc_pipe.as_str()],
        ),
        (
            "run tail sort wc pipe",
            tail_sort_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", tail_sort_wc_pipe.as_str()],
        ),
        (
            "run tail sort head pipe",
            tail_sort_head_pipe.clone(),
            "/bin/bash",
            vec!["-c", tail_sort_head_pipe.as_str()],
        ),
        (
            "run tail sort tail pipe",
            tail_sort_tail_pipe.clone(),
            "/bin/bash",
            vec!["-c", tail_sort_tail_pipe.as_str()],
        ),
        (
            "run tail sort xargs echo pipe",
            tail_sort_xargs_echo_pipe.clone(),
            "/bin/bash",
            vec!["-c", tail_sort_xargs_echo_pipe.as_str()],
        ),
        (
            "run tail sort xargs wc pipe",
            tail_sort_xargs_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", tail_sort_xargs_wc_pipe.as_str()],
        ),
        (
            "run tail xargs echo pipe",
            tail_xargs_echo_pipe.clone(),
            "/bin/bash",
            vec!["-c", tail_xargs_echo_pipe.as_str()],
        ),
        (
            "run tail xargs wc pipe",
            tail_xargs_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", tail_xargs_wc_pipe.as_str()],
        ),
        (
            "run tail grep pipe",
            tail_grep_pipe.clone(),
            "/bin/bash",
            vec!["-c", tail_grep_pipe.as_str()],
        ),
        (
            "run tail grep no-newline pipe",
            tail_grep_no_newline_pipe.clone(),
            "/bin/bash",
            vec!["-c", tail_grep_no_newline_pipe.as_str()],
        ),
        (
            "run tail grep wc pipe",
            tail_grep_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", tail_grep_wc_pipe.as_str()],
        ),
        (
            "run tail grep head pipe",
            tail_grep_head_pipe.clone(),
            "/bin/bash",
            vec!["-c", tail_grep_head_pipe.as_str()],
        ),
        (
            "run tail grep tail pipe",
            tail_grep_tail_pipe.clone(),
            "/bin/bash",
            vec!["-c", tail_grep_tail_pipe.as_str()],
        ),
        (
            "run tail grep sort pipe",
            tail_grep_sort_pipe.clone(),
            "/bin/bash",
            vec!["-c", tail_grep_sort_pipe.as_str()],
        ),
        (
            "run tail grep sort uniq pipe",
            tail_grep_sort_uniq_pipe.clone(),
            "/bin/bash",
            vec!["-c", tail_grep_sort_uniq_pipe.as_str()],
        ),
        (
            "run tail grep sort uniq wc pipe",
            tail_grep_sort_uniq_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", tail_grep_sort_uniq_wc_pipe.as_str()],
        ),
        (
            "run tail grep sort wc pipe",
            tail_grep_sort_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", tail_grep_sort_wc_pipe.as_str()],
        ),
        (
            "run tail grep sort head pipe",
            tail_grep_sort_head_pipe.clone(),
            "/bin/bash",
            vec!["-c", tail_grep_sort_head_pipe.as_str()],
        ),
        (
            "run tail grep sort tail pipe",
            tail_grep_sort_tail_pipe.clone(),
            "/bin/bash",
            vec!["-c", tail_grep_sort_tail_pipe.as_str()],
        ),
        (
            "run tail grep sort xargs echo pipe",
            tail_grep_sort_xargs_echo_pipe.clone(),
            "/bin/bash",
            vec!["-c", tail_grep_sort_xargs_echo_pipe.as_str()],
        ),
        (
            "run tail grep sort xargs wc pipe",
            tail_grep_sort_xargs_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", tail_grep_sort_xargs_wc_pipe.as_str()],
        ),
        (
            "run tail grep xargs echo pipe",
            tail_grep_xargs_echo_pipe.clone(),
            "/bin/bash",
            vec!["-c", tail_grep_xargs_echo_pipe.as_str()],
        ),
        (
            "run tail grep xargs wc pipe",
            tail_grep_xargs_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", tail_grep_xargs_wc_pipe.as_str()],
        ),
        (
            "run cat wc pipe",
            cat_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_wc_pipe.as_str()],
        ),
        (
            "run cat head pipe",
            cat_head_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_head_pipe.as_str()],
        ),
        (
            "run cat tail pipe",
            cat_tail_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_tail_pipe.as_str()],
        ),
        (
            "run cat head default pipe",
            cat_head_default_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_head_default_pipe.as_str()],
        ),
        (
            "run cat tail default pipe",
            cat_tail_default_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_tail_default_pipe.as_str()],
        ),
        (
            "run cat head short pipe",
            cat_head_short_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_head_short_pipe.as_str()],
        ),
        (
            "run cat tail short pipe",
            cat_tail_short_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_tail_short_pipe.as_str()],
        ),
        (
            "run cat head wc pipe",
            cat_head_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_head_wc_pipe.as_str()],
        ),
        (
            "run cat tail wc pipe",
            cat_tail_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_tail_wc_pipe.as_str()],
        ),
        (
            "run cat head sort uniq wc pipe",
            cat_head_sort_uniq_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_head_sort_uniq_wc_pipe.as_str()],
        ),
        (
            "run cat tail sort uniq wc pipe",
            cat_tail_sort_uniq_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_tail_sort_uniq_wc_pipe.as_str()],
        ),
        (
            "run cat head grep sort xargs pipe",
            cat_head_grep_sort_xargs_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_head_grep_sort_xargs_pipe.as_str()],
        ),
        (
            "run cat tail grep sort xargs pipe",
            cat_tail_grep_sort_xargs_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_tail_grep_sort_xargs_pipe.as_str()],
        ),
        (
            "run cat head xargs wc pipe",
            cat_head_xargs_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_head_xargs_wc_pipe.as_str()],
        ),
        (
            "run cat tail xargs wc pipe",
            cat_tail_xargs_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_tail_xargs_wc_pipe.as_str()],
        ),
        (
            "run cat grep pipe",
            cat_grep_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_grep_pipe.as_str()],
        ),
        (
            "run cat grep wc pipe",
            cat_grep_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_grep_wc_pipe.as_str()],
        ),
        (
            "run cat grep head pipe",
            cat_grep_head_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_grep_head_pipe.as_str()],
        ),
        (
            "run cat grep tail pipe",
            cat_grep_tail_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_grep_tail_pipe.as_str()],
        ),
        (
            "run cat grep sort pipe",
            cat_grep_sort_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_grep_sort_pipe.as_str()],
        ),
        (
            "run cat grep sort uniq pipe",
            cat_grep_sort_uniq_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_grep_sort_uniq_pipe.as_str()],
        ),
        (
            "run cat grep sort uniq wc pipe",
            cat_grep_sort_uniq_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_grep_sort_uniq_wc_pipe.as_str()],
        ),
        (
            "run cat grep sort uniq head pipe",
            cat_grep_sort_uniq_head_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_grep_sort_uniq_head_pipe.as_str()],
        ),
        (
            "run cat grep sort uniq tail pipe",
            cat_grep_sort_uniq_tail_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_grep_sort_uniq_tail_pipe.as_str()],
        ),
        (
            "run cat grep sort uniq sort xargs pipe",
            cat_grep_sort_uniq_sort_xargs_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_grep_sort_uniq_sort_xargs_pipe.as_str()],
        ),
        (
            "run cat grep sort uniq xargs wc pipe",
            cat_grep_sort_uniq_xargs_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_grep_sort_uniq_xargs_wc_pipe.as_str()],
        ),
        (
            "run cat grep sort uniq sort xargs wc pipe",
            cat_grep_sort_uniq_sort_xargs_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_grep_sort_uniq_sort_xargs_wc_pipe.as_str()],
        ),
        (
            "run cat grep sort wc pipe",
            cat_grep_sort_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_grep_sort_wc_pipe.as_str()],
        ),
        (
            "run cat grep sort head pipe",
            cat_grep_sort_head_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_grep_sort_head_pipe.as_str()],
        ),
        (
            "run cat grep sort tail pipe",
            cat_grep_sort_tail_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_grep_sort_tail_pipe.as_str()],
        ),
        (
            "run cat cut pipe",
            cat_cut_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_cut_pipe.as_str()],
        ),
        (
            "run cat cut wc pipe",
            cat_cut_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_cut_wc_pipe.as_str()],
        ),
        (
            "run cat cut sort uniq wc pipe",
            cat_cut_sort_uniq_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_cut_sort_uniq_wc_pipe.as_str()],
        ),
        (
            "run cat cut xargs wc pipe",
            cat_cut_xargs_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_cut_xargs_wc_pipe.as_str()],
        ),
        (
            "run cat cut grep pipe",
            cat_cut_grep_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_cut_grep_pipe.as_str()],
        ),
        (
            "run cat cut grep sort xargs echo pipe",
            cat_cut_grep_sort_xargs_echo_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_cut_grep_sort_xargs_echo_pipe.as_str()],
        ),
        (
            "run cat cut grep xargs wc pipe",
            cat_cut_grep_xargs_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_cut_grep_xargs_wc_pipe.as_str()],
        ),
        (
            "run cat tr pipe",
            cat_tr_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_tr_pipe.as_str()],
        ),
        (
            "run cat tr class pipe",
            cat_tr_class_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_tr_class_pipe.as_str()],
        ),
        (
            "run cat tr wc pipe",
            cat_tr_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_tr_wc_pipe.as_str()],
        ),
        (
            "run cat tr sort uniq wc pipe",
            cat_tr_sort_uniq_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_tr_sort_uniq_wc_pipe.as_str()],
        ),
        (
            "run cat tr xargs wc pipe",
            cat_tr_xargs_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_tr_xargs_wc_pipe.as_str()],
        ),
        (
            "run cat tr grep pipe",
            cat_tr_grep_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_tr_grep_pipe.as_str()],
        ),
        (
            "run cat tr grep sort xargs echo pipe",
            cat_tr_grep_sort_xargs_echo_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_tr_grep_sort_xargs_echo_pipe.as_str()],
        ),
        (
            "run cat tr grep xargs wc pipe",
            cat_tr_grep_xargs_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_tr_grep_xargs_wc_pipe.as_str()],
        ),
        (
            "run cat uniq pipe",
            cat_uniq_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_uniq_pipe.as_str()],
        ),
        (
            "run cat uniq wc pipe",
            cat_uniq_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_uniq_wc_pipe.as_str()],
        ),
        (
            "run cat uniq sort uniq wc pipe",
            cat_uniq_sort_uniq_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_uniq_sort_uniq_wc_pipe.as_str()],
        ),
        (
            "run cat uniq xargs wc pipe",
            cat_uniq_xargs_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_uniq_xargs_wc_pipe.as_str()],
        ),
        (
            "run cat uniq grep pipe",
            cat_uniq_grep_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_uniq_grep_pipe.as_str()],
        ),
        (
            "run cat uniq grep sort xargs echo pipe",
            cat_uniq_grep_sort_xargs_echo_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_uniq_grep_sort_xargs_echo_pipe.as_str()],
        ),
        (
            "run cat uniq grep xargs wc pipe",
            cat_uniq_grep_xargs_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_uniq_grep_xargs_wc_pipe.as_str()],
        ),
        (
            "run uniq wc pipe",
            uniq_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", uniq_wc_pipe.as_str()],
        ),
        (
            "run uniq sort uniq wc pipe",
            uniq_sort_uniq_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", uniq_sort_uniq_wc_pipe.as_str()],
        ),
        (
            "run uniq xargs wc pipe",
            uniq_xargs_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", uniq_xargs_wc_pipe.as_str()],
        ),
        (
            "run uniq grep pipe",
            uniq_grep_pipe.clone(),
            "/bin/bash",
            vec!["-c", uniq_grep_pipe.as_str()],
        ),
        (
            "run uniq grep sort xargs echo pipe",
            uniq_grep_sort_xargs_echo_pipe.clone(),
            "/bin/bash",
            vec!["-c", uniq_grep_sort_xargs_echo_pipe.as_str()],
        ),
        (
            "run uniq grep xargs wc pipe",
            uniq_grep_xargs_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", uniq_grep_xargs_wc_pipe.as_str()],
        ),
        (
            "run cat sort pipe",
            cat_sort_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_sort_pipe.as_str()],
        ),
        (
            "run cat sort uniq pipe",
            cat_sort_uniq_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_sort_uniq_pipe.as_str()],
        ),
        (
            "run cat sort uniq wc pipe",
            cat_sort_uniq_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_sort_uniq_wc_pipe.as_str()],
        ),
        (
            "run cat sort uniq head pipe",
            cat_sort_uniq_head_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_sort_uniq_head_pipe.as_str()],
        ),
        (
            "run cat sort uniq sort uniq wc pipe",
            cat_sort_uniq_sort_uniq_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_sort_uniq_sort_uniq_wc_pipe.as_str()],
        ),
        (
            "run cat sort uniq xargs wc pipe",
            cat_sort_uniq_xargs_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_sort_uniq_xargs_wc_pipe.as_str()],
        ),
        (
            "run cat sort uniq grep pipe",
            cat_sort_uniq_grep_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_sort_uniq_grep_pipe.as_str()],
        ),
        (
            "run cat sort uniq grep sort xargs echo pipe",
            cat_sort_uniq_grep_sort_xargs_echo_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_sort_uniq_grep_sort_xargs_echo_pipe.as_str()],
        ),
        (
            "run cat sort uniq grep xargs wc pipe",
            cat_sort_uniq_grep_xargs_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_sort_uniq_grep_xargs_wc_pipe.as_str()],
        ),
        (
            "run cat sort grep pipe",
            cat_sort_grep_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_sort_grep_pipe.as_str()],
        ),
        (
            "run cat sort grep wc pipe",
            cat_sort_grep_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_sort_grep_wc_pipe.as_str()],
        ),
        (
            "run cat sort grep sort xargs echo pipe",
            cat_sort_grep_sort_xargs_echo_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_sort_grep_sort_xargs_echo_pipe.as_str()],
        ),
        (
            "run cat sort grep xargs wc pipe",
            cat_sort_grep_xargs_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_sort_grep_xargs_wc_pipe.as_str()],
        ),
        (
            "run cat sort wc pipe",
            cat_sort_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_sort_wc_pipe.as_str()],
        ),
        (
            "run cat sort head pipe",
            cat_sort_head_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_sort_head_pipe.as_str()],
        ),
        (
            "run cat sort tail pipe",
            cat_sort_tail_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_sort_tail_pipe.as_str()],
        ),
        (
            "run cat xargs echo pipe",
            cat_xargs_echo_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_xargs_echo_pipe.as_str()],
        ),
        (
            "run sort xargs echo pipe",
            sort_xargs_echo_pipe.clone(),
            "/bin/bash",
            vec!["-c", sort_xargs_echo_pipe.as_str()],
        ),
        (
            "run cat sort xargs echo pipe",
            cat_sort_xargs_echo_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_sort_xargs_echo_pipe.as_str()],
        ),
        (
            "run cat xargs wc pipe",
            cat_xargs_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_xargs_wc_pipe.as_str()],
        ),
        (
            "run cat xargs wc sort pipe",
            cat_xargs_wc_sort_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_xargs_wc_sort_pipe.as_str()],
        ),
        (
            "run sort xargs wc pipe",
            sort_xargs_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", sort_xargs_wc_pipe.as_str()],
        ),
        (
            "run sort xargs wc sort tail pipe",
            sort_xargs_wc_sort_tail_pipe.clone(),
            "/bin/bash",
            vec!["-c", sort_xargs_wc_sort_tail_pipe.as_str()],
        ),
        (
            "run cat sort xargs wc pipe",
            cat_sort_xargs_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_sort_xargs_wc_pipe.as_str()],
        ),
        (
            "run cat sort xargs wc sort pipe",
            cat_sort_xargs_wc_sort_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_sort_xargs_wc_sort_pipe.as_str()],
        ),
        (
            "run cat grep xargs echo pipe",
            cat_grep_xargs_echo_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_grep_xargs_echo_pipe.as_str()],
        ),
        (
            "run cat grep xargs wc pipe",
            cat_grep_xargs_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_grep_xargs_wc_pipe.as_str()],
        ),
        (
            "run cat grep sort xargs echo pipe",
            cat_grep_sort_xargs_echo_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_grep_sort_xargs_echo_pipe.as_str()],
        ),
        (
            "run cat grep sort xargs wc pipe",
            cat_grep_sort_xargs_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_grep_sort_xargs_wc_pipe.as_str()],
        ),
        (
            "run grep head pipe",
            grep_head_pipe.clone(),
            "/bin/bash",
            vec!["-c", grep_head_pipe.as_str()],
        ),
        (
            "run grep tail pipe",
            grep_tail_pipe.clone(),
            "/bin/bash",
            vec!["-c", grep_tail_pipe.as_str()],
        ),
        (
            "run grep sort pipe",
            grep_sort_pipe.clone(),
            "/bin/bash",
            vec!["-c", grep_sort_pipe.as_str()],
        ),
        (
            "run grep sort uniq pipe",
            grep_sort_uniq_pipe.clone(),
            "/bin/bash",
            vec!["-c", grep_sort_uniq_pipe.as_str()],
        ),
        (
            "run grep sort uniq wc pipe",
            grep_sort_uniq_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", grep_sort_uniq_wc_pipe.as_str()],
        ),
        (
            "run grep sort uniq head pipe",
            grep_sort_uniq_head_pipe.clone(),
            "/bin/bash",
            vec!["-c", grep_sort_uniq_head_pipe.as_str()],
        ),
        (
            "run grep sort uniq tail pipe",
            grep_sort_uniq_tail_pipe.clone(),
            "/bin/bash",
            vec!["-c", grep_sort_uniq_tail_pipe.as_str()],
        ),
        (
            "run grep sort uniq sort xargs pipe",
            grep_sort_uniq_sort_xargs_pipe.clone(),
            "/bin/bash",
            vec!["-c", grep_sort_uniq_sort_xargs_pipe.as_str()],
        ),
        (
            "run grep sort wc pipe",
            grep_sort_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", grep_sort_wc_pipe.as_str()],
        ),
        (
            "run grep sort head pipe",
            grep_sort_head_pipe.clone(),
            "/bin/bash",
            vec!["-c", grep_sort_head_pipe.as_str()],
        ),
        (
            "run grep sort tail pipe",
            grep_sort_tail_pipe.clone(),
            "/bin/bash",
            vec!["-c", grep_sort_tail_pipe.as_str()],
        ),
        (
            "run grep wc pipe",
            grep_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", grep_wc_pipe.as_str()],
        ),
        (
            "run grep file wc pipe",
            grep_file_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", grep_file_wc_pipe.as_str()],
        ),
        (
            "run grep file head pipe",
            grep_file_head_pipe.clone(),
            "/bin/bash",
            vec!["-c", grep_file_head_pipe.as_str()],
        ),
        (
            "run grep file tail pipe",
            grep_file_tail_pipe.clone(),
            "/bin/bash",
            vec!["-c", grep_file_tail_pipe.as_str()],
        ),
        (
            "run grep file sort pipe",
            grep_file_sort_pipe.clone(),
            "/bin/bash",
            vec!["-c", grep_file_sort_pipe.as_str()],
        ),
        (
            "run grep file sort uniq pipe",
            grep_file_sort_uniq_pipe.clone(),
            "/bin/bash",
            vec!["-c", grep_file_sort_uniq_pipe.as_str()],
        ),
        (
            "run grep file sort uniq wc pipe",
            grep_file_sort_uniq_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", grep_file_sort_uniq_wc_pipe.as_str()],
        ),
        (
            "run grep file sort uniq head pipe",
            grep_file_sort_uniq_head_pipe.clone(),
            "/bin/bash",
            vec!["-c", grep_file_sort_uniq_head_pipe.as_str()],
        ),
        (
            "run grep file sort uniq tail pipe",
            grep_file_sort_uniq_tail_pipe.clone(),
            "/bin/bash",
            vec!["-c", grep_file_sort_uniq_tail_pipe.as_str()],
        ),
        (
            "run grep file sort uniq sort xargs pipe",
            grep_file_sort_uniq_sort_xargs_pipe.clone(),
            "/bin/bash",
            vec!["-c", grep_file_sort_uniq_sort_xargs_pipe.as_str()],
        ),
        (
            "run grep file sort uniq xargs wc pipe",
            grep_file_sort_uniq_xargs_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", grep_file_sort_uniq_xargs_wc_pipe.as_str()],
        ),
        (
            "run grep file sort uniq sort xargs wc pipe",
            grep_file_sort_uniq_sort_xargs_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", grep_file_sort_uniq_sort_xargs_wc_pipe.as_str()],
        ),
        (
            "run grep file sort wc pipe",
            grep_file_sort_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", grep_file_sort_wc_pipe.as_str()],
        ),
        (
            "run grep file sort head pipe",
            grep_file_sort_head_pipe.clone(),
            "/bin/bash",
            vec!["-c", grep_file_sort_head_pipe.as_str()],
        ),
        (
            "run grep file sort tail pipe",
            grep_file_sort_tail_pipe.clone(),
            "/bin/bash",
            vec!["-c", grep_file_sort_tail_pipe.as_str()],
        ),
        (
            "run grep file xargs pipe",
            grep_file_xargs_pipe.clone(),
            "/bin/bash",
            vec!["-c", grep_file_xargs_pipe.as_str()],
        ),
        (
            "run grep file xargs wc pipe",
            grep_file_xargs_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", grep_file_xargs_wc_pipe.as_str()],
        ),
        (
            "run grep file xargs wc sort pipe",
            grep_file_xargs_wc_sort_pipe.clone(),
            "/bin/bash",
            vec!["-c", grep_file_xargs_wc_sort_pipe.as_str()],
        ),
        (
            "run grep file sort xargs pipe",
            grep_file_sort_xargs_pipe.clone(),
            "/bin/bash",
            vec!["-c", grep_file_sort_xargs_pipe.as_str()],
        ),
        (
            "run grep file sort xargs wc pipe",
            grep_file_sort_xargs_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", grep_file_sort_xargs_wc_pipe.as_str()],
        ),
        (
            "run grep file sort xargs wc sort tail pipe",
            grep_file_sort_xargs_wc_sort_tail_pipe.clone(),
            "/bin/bash",
            vec!["-c", grep_file_sort_xargs_wc_sort_tail_pipe.as_str()],
        ),
        (
            "run grep file cut pipe",
            grep_file_cut_pipe.clone(),
            "/bin/bash",
            vec!["-c", grep_file_cut_pipe.as_str()],
        ),
        (
            "run grep file cut wc pipe",
            grep_file_cut_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", grep_file_cut_wc_pipe.as_str()],
        ),
        (
            "run grep file cut sort pipe",
            grep_file_cut_sort_pipe.clone(),
            "/bin/bash",
            vec!["-c", grep_file_cut_sort_pipe.as_str()],
        ),
        (
            "run grep file cut sort uniq wc pipe",
            grep_file_cut_sort_uniq_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", grep_file_cut_sort_uniq_wc_pipe.as_str()],
        ),
        (
            "run grep file cut grep wc pipe",
            grep_file_cut_grep_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", grep_file_cut_grep_wc_pipe.as_str()],
        ),
        (
            "run grep file cut xargs pipe",
            grep_file_cut_xargs_pipe.clone(),
            "/bin/bash",
            vec!["-c", grep_file_cut_xargs_pipe.as_str()],
        ),
        (
            "run grep file cut xargs wc pipe",
            grep_file_cut_xargs_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", grep_file_cut_xargs_wc_pipe.as_str()],
        ),
        (
            "run grep file awk pipe",
            grep_file_awk_pipe.clone(),
            "/bin/bash",
            vec!["-c", grep_file_awk_pipe.as_str()],
        ),
        (
            "run grep file awk predicate pipe",
            grep_file_awk_predicate_pipe.clone(),
            "/bin/bash",
            vec!["-c", grep_file_awk_predicate_pipe.as_str()],
        ),
        (
            "run grep file awk second-field wc pipe",
            grep_file_awk_second_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", grep_file_awk_second_wc_pipe.as_str()],
        ),
        (
            "run grep file awk wc pipe",
            grep_file_awk_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", grep_file_awk_wc_pipe.as_str()],
        ),
        (
            "run grep file awk compact wc pipe",
            grep_file_awk_compact_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", grep_file_awk_compact_wc_pipe.as_str()],
        ),
        (
            "run grep file awk sort pipe",
            grep_file_awk_sort_pipe.clone(),
            "/bin/bash",
            vec!["-c", grep_file_awk_sort_pipe.as_str()],
        ),
        (
            "run grep file awk sort uniq wc pipe",
            grep_file_awk_sort_uniq_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", grep_file_awk_sort_uniq_wc_pipe.as_str()],
        ),
        (
            "run grep file awk grep wc pipe",
            grep_file_awk_grep_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", grep_file_awk_grep_wc_pipe.as_str()],
        ),
        (
            "run grep file awk xargs pipe",
            grep_file_awk_xargs_pipe.clone(),
            "/bin/bash",
            vec!["-c", grep_file_awk_xargs_pipe.as_str()],
        ),
        (
            "run grep file awk xargs wc pipe",
            grep_file_awk_xargs_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", grep_file_awk_xargs_wc_pipe.as_str()],
        ),
        (
            "run awk first wc pipe",
            awk_first_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", awk_first_wc_pipe.as_str()],
        ),
        (
            "run awk first compact wc pipe",
            awk_first_compact_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", awk_first_compact_wc_pipe.as_str()],
        ),
        (
            "run awk second sort pipe",
            awk_second_sort_pipe.clone(),
            "/bin/bash",
            vec!["-c", awk_second_sort_pipe.as_str()],
        ),
        (
            "run awk first sort uniq wc pipe",
            awk_first_sort_uniq_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", awk_first_sort_uniq_wc_pipe.as_str()],
        ),
        (
            "run awk first xargs pipe",
            awk_first_xargs_pipe.clone(),
            "/bin/bash",
            vec!["-c", awk_first_xargs_pipe.as_str()],
        ),
        (
            "run awk first xargs wc pipe",
            awk_first_xargs_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", awk_first_xargs_wc_pipe.as_str()],
        ),
        (
            "run awk first xargs wc sort pipe",
            awk_first_xargs_wc_sort_pipe.clone(),
            "/bin/bash",
            vec!["-c", awk_first_xargs_wc_sort_pipe.as_str()],
        ),
        (
            "run awk first grep pipe",
            awk_first_grep_pipe.clone(),
            "/bin/bash",
            vec!["-c", awk_first_grep_pipe.as_str()],
        ),
        (
            "run awk first grep wc pipe",
            awk_first_grep_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", awk_first_grep_wc_pipe.as_str()],
        ),
        (
            "run awk first grep sort uniq wc pipe",
            awk_first_grep_sort_uniq_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", awk_first_grep_sort_uniq_wc_pipe.as_str()],
        ),
        (
            "run awk first grep xargs wc pipe",
            awk_first_grep_xargs_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", awk_first_grep_xargs_wc_pipe.as_str()],
        ),
        (
            "run awk first grep xargs wc sort pipe",
            awk_first_grep_xargs_wc_sort_pipe.clone(),
            "/bin/bash",
            vec!["-c", awk_first_grep_xargs_wc_sort_pipe.as_str()],
        ),
        (
            "run awk wc pipe",
            awk_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", awk_wc_pipe.as_str()],
        ),
        (
            "run awk head pipe",
            awk_head_pipe.clone(),
            "/bin/bash",
            vec!["-c", awk_head_pipe.as_str()],
        ),
        (
            "run awk tail pipe",
            awk_tail_pipe.clone(),
            "/bin/bash",
            vec!["-c", awk_tail_pipe.as_str()],
        ),
        (
            "run awk sort pipe",
            awk_sort_pipe.clone(),
            "/bin/bash",
            vec!["-c", awk_sort_pipe.as_str()],
        ),
        (
            "run awk sort uniq pipe",
            awk_sort_uniq_pipe.clone(),
            "/bin/bash",
            vec!["-c", awk_sort_uniq_pipe.as_str()],
        ),
        (
            "run awk sort uniq wc pipe",
            awk_sort_uniq_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", awk_sort_uniq_wc_pipe.as_str()],
        ),
        (
            "run awk sort uniq head pipe",
            awk_sort_uniq_head_pipe.clone(),
            "/bin/bash",
            vec!["-c", awk_sort_uniq_head_pipe.as_str()],
        ),
        (
            "run awk sort uniq sort xargs pipe",
            awk_sort_uniq_sort_xargs_pipe.clone(),
            "/bin/bash",
            vec!["-c", awk_sort_uniq_sort_xargs_pipe.as_str()],
        ),
        (
            "run awk sort uniq xargs wc pipe",
            awk_sort_uniq_xargs_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", awk_sort_uniq_xargs_wc_pipe.as_str()],
        ),
        (
            "run awk sort uniq sort xargs wc pipe",
            awk_sort_uniq_sort_xargs_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", awk_sort_uniq_sort_xargs_wc_pipe.as_str()],
        ),
        (
            "run awk sort wc pipe",
            awk_sort_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", awk_sort_wc_pipe.as_str()],
        ),
        (
            "run awk sort head pipe",
            awk_sort_head_pipe.clone(),
            "/bin/bash",
            vec!["-c", awk_sort_head_pipe.as_str()],
        ),
        (
            "run awk sort tail pipe",
            awk_sort_tail_pipe.clone(),
            "/bin/bash",
            vec!["-c", awk_sort_tail_pipe.as_str()],
        ),
        (
            "run awk xargs pipe",
            awk_xargs_pipe.clone(),
            "/bin/bash",
            vec!["-c", awk_xargs_pipe.as_str()],
        ),
        (
            "run awk xargs wc pipe",
            awk_xargs_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", awk_xargs_wc_pipe.as_str()],
        ),
        (
            "run awk xargs wc sort pipe",
            awk_xargs_wc_sort_pipe.clone(),
            "/bin/bash",
            vec!["-c", awk_xargs_wc_sort_pipe.as_str()],
        ),
        (
            "run awk sort xargs pipe",
            awk_sort_xargs_pipe.clone(),
            "/bin/bash",
            vec!["-c", awk_sort_xargs_pipe.as_str()],
        ),
        (
            "run awk sort xargs wc pipe",
            awk_sort_xargs_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", awk_sort_xargs_wc_pipe.as_str()],
        ),
        (
            "run awk sort xargs wc sort tail pipe",
            awk_sort_xargs_wc_sort_tail_pipe.clone(),
            "/bin/bash",
            vec!["-c", awk_sort_xargs_wc_sort_tail_pipe.as_str()],
        ),
        (
            "run cat awk first pipe",
            cat_awk_first_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_awk_first_pipe.as_str()],
        ),
        (
            "run cat awk first compact pipe",
            cat_awk_first_compact_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_awk_first_compact_pipe.as_str()],
        ),
        (
            "run cat awk second wc pipe",
            cat_awk_second_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_awk_second_wc_pipe.as_str()],
        ),
        (
            "run cat awk first wc pipe",
            cat_awk_first_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_awk_first_wc_pipe.as_str()],
        ),
        (
            "run cat awk first xargs wc pipe",
            cat_awk_first_xargs_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_awk_first_xargs_wc_pipe.as_str()],
        ),
        (
            "run cat awk first xargs wc sort pipe",
            cat_awk_first_xargs_wc_sort_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_awk_first_xargs_wc_sort_pipe.as_str()],
        ),
        (
            "run cat awk first grep tail pipe",
            cat_awk_first_grep_tail_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_awk_first_grep_tail_pipe.as_str()],
        ),
        (
            "run cat awk first grep sort xargs wc pipe",
            cat_awk_first_grep_sort_xargs_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_awk_first_grep_sort_xargs_wc_pipe.as_str()],
        ),
        (
            "run cat awk first grep sort xargs wc sort tail pipe",
            cat_awk_first_grep_sort_xargs_wc_sort_tail_pipe.clone(),
            "/bin/bash",
            vec![
                "-c",
                cat_awk_first_grep_sort_xargs_wc_sort_tail_pipe.as_str(),
            ],
        ),
        (
            "run cat awk pipe",
            cat_awk_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_awk_pipe.as_str()],
        ),
        (
            "run cat awk wc pipe",
            cat_awk_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_awk_wc_pipe.as_str()],
        ),
        (
            "run cat awk head pipe",
            cat_awk_head_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_awk_head_pipe.as_str()],
        ),
        (
            "run cat awk tail pipe",
            cat_awk_tail_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_awk_tail_pipe.as_str()],
        ),
        (
            "run cat awk sort pipe",
            cat_awk_sort_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_awk_sort_pipe.as_str()],
        ),
        (
            "run cat awk sort uniq wc pipe",
            cat_awk_sort_uniq_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_awk_sort_uniq_wc_pipe.as_str()],
        ),
        (
            "run cat awk xargs pipe",
            cat_awk_xargs_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_awk_xargs_pipe.as_str()],
        ),
        (
            "run cat awk xargs wc pipe",
            cat_awk_xargs_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_awk_xargs_wc_pipe.as_str()],
        ),
        (
            "run cat awk xargs wc sort pipe",
            cat_awk_xargs_wc_sort_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_awk_xargs_wc_sort_pipe.as_str()],
        ),
        (
            "run cat awk sort xargs pipe",
            cat_awk_sort_xargs_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_awk_sort_xargs_pipe.as_str()],
        ),
        (
            "run cat awk sort xargs wc pipe",
            cat_awk_sort_xargs_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_awk_sort_xargs_wc_pipe.as_str()],
        ),
        (
            "run cat awk sort xargs wc sort tail pipe",
            cat_awk_sort_xargs_wc_sort_tail_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_awk_sort_xargs_wc_sort_tail_pipe.as_str()],
        ),
        (
            "run find all xargs wc pipe",
            find_all_xargs_pipe.clone(),
            "/bin/bash",
            vec!["-c", find_all_xargs_pipe.as_str()],
        ),
        (
            "run find all xargs wc wc pipe",
            find_all_xargs_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", find_all_xargs_wc_pipe.as_str()],
        ),
        (
            "run find all xargs wc sort pipe",
            find_all_xargs_sort_pipe.clone(),
            "/bin/bash",
            vec!["-c", find_all_xargs_sort_pipe.as_str()],
        ),
        (
            "run find all xargs wc sort tail pipe",
            find_all_xargs_sort_tail_pipe.clone(),
            "/bin/bash",
            vec!["-c", find_all_xargs_sort_tail_pipe.as_str()],
        ),
        (
            "run find all xargs echo pipe",
            find_all_xargs_echo_pipe.clone(),
            "/bin/bash",
            vec!["-c", find_all_xargs_echo_pipe.as_str()],
        ),
        (
            "run find all default xargs pipe",
            find_all_xargs_default_pipe.clone(),
            "/bin/bash",
            vec!["-c", find_all_xargs_default_pipe.as_str()],
        ),
        (
            "run find all wc pipe",
            find_all_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", find_all_wc_pipe.as_str()],
        ),
        (
            "run find all head pipe",
            find_all_head_pipe.clone(),
            "/bin/bash",
            vec!["-c", find_all_head_pipe.as_str()],
        ),
        (
            "run find all tail pipe",
            find_all_tail_pipe.clone(),
            "/bin/bash",
            vec!["-c", find_all_tail_pipe.as_str()],
        ),
        (
            "run find all sort pipe",
            find_all_sort_pipe.clone(),
            "/bin/bash",
            vec!["-c", find_all_sort_pipe.as_str()],
        ),
        (
            "run find all sort uniq pipe",
            find_all_sort_uniq_pipe.clone(),
            "/bin/bash",
            vec!["-c", find_all_sort_uniq_pipe.as_str()],
        ),
        (
            "run find all sort uniq wc pipe",
            find_all_sort_uniq_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", find_all_sort_uniq_wc_pipe.as_str()],
        ),
        (
            "run find all sort uniq head pipe",
            find_all_sort_uniq_head_pipe.clone(),
            "/bin/bash",
            vec!["-c", find_all_sort_uniq_head_pipe.as_str()],
        ),
        (
            "run find all sort uniq sort uniq wc pipe",
            find_all_sort_uniq_sort_uniq_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", find_all_sort_uniq_sort_uniq_wc_pipe.as_str()],
        ),
        (
            "run find all sort uniq xargs wc pipe",
            find_all_sort_uniq_xargs_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", find_all_sort_uniq_xargs_wc_pipe.as_str()],
        ),
        (
            "run find all sort uniq xargs wc sort wc pipe",
            find_all_sort_uniq_xargs_sort_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", find_all_sort_uniq_xargs_sort_wc_pipe.as_str()],
        ),
        (
            "run find all sort uniq grep pipe",
            find_all_sort_uniq_grep_pipe.clone(),
            "/bin/bash",
            vec!["-c", find_all_sort_uniq_grep_pipe.as_str()],
        ),
        (
            "run find all sort uniq grep sort xargs wc pipe",
            find_all_sort_uniq_grep_sort_xargs_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", find_all_sort_uniq_grep_sort_xargs_wc_pipe.as_str()],
        ),
        (
            "run find all sort wc pipe",
            find_all_sort_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", find_all_sort_wc_pipe.as_str()],
        ),
        (
            "run find all sort xargs echo pipe",
            find_all_sort_xargs_echo_pipe.clone(),
            "/bin/bash",
            vec!["-c", find_all_sort_xargs_echo_pipe.as_str()],
        ),
        (
            "run find all sort xargs wc pipe",
            find_all_sort_xargs_pipe.clone(),
            "/bin/bash",
            vec!["-c", find_all_sort_xargs_pipe.as_str()],
        ),
        (
            "run find all sort xargs wc sort tail pipe",
            find_all_sort_xargs_sort_tail_pipe.clone(),
            "/bin/bash",
            vec!["-c", find_all_sort_xargs_sort_tail_pipe.as_str()],
        ),
        (
            "run find all sort head pipe",
            find_all_sort_head_pipe.clone(),
            "/bin/bash",
            vec!["-c", find_all_sort_head_pipe.as_str()],
        ),
        (
            "run find all sort tail pipe",
            find_all_sort_tail_pipe.clone(),
            "/bin/bash",
            vec!["-c", find_all_sort_tail_pipe.as_str()],
        ),
        (
            "run find maxdepth wc pipe",
            find_maxdepth_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", find_maxdepth_wc_pipe.as_str()],
        ),
        (
            "run find maxdepth head pipe",
            find_maxdepth_head_pipe.clone(),
            "/bin/bash",
            vec!["-c", find_maxdepth_head_pipe.as_str()],
        ),
        (
            "run find maxdepth grep wc pipe",
            find_maxdepth_grep_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", find_maxdepth_grep_wc_pipe.as_str()],
        ),
        (
            "run find maxdepth xargs echo pipe",
            find_maxdepth_xargs_echo_pipe.clone(),
            "/bin/bash",
            vec!["-c", find_maxdepth_xargs_echo_pipe.as_str()],
        ),
        (
            "run find maxdepth two sort tail pipe",
            find_maxdepth_two_sort_tail_pipe.clone(),
            "/bin/bash",
            vec!["-c", find_maxdepth_two_sort_tail_pipe.as_str()],
        ),
        (
            "run find maxdepth two name grep wc pipe",
            find_maxdepth_two_name_grep_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", find_maxdepth_two_name_grep_wc_pipe.as_str()],
        ),
        (
            "run find xargs wc pipe",
            find_xargs_pipe.clone(),
            "/bin/bash",
            vec!["-c", find_xargs_pipe.as_str()],
        ),
        (
            "run find xargs wc sort pipe",
            find_xargs_sort_pipe.clone(),
            "/bin/bash",
            vec!["-c", find_xargs_sort_pipe.as_str()],
        ),
        (
            "run find xargs echo pipe",
            find_xargs_echo_pipe.clone(),
            "/bin/bash",
            vec!["-c", find_xargs_echo_pipe.as_str()],
        ),
        (
            "run find default xargs pipe",
            find_xargs_default_pipe.clone(),
            "/bin/bash",
            vec!["-c", find_xargs_default_pipe.as_str()],
        ),
        (
            "run find grep xargs echo pipe",
            find_grep_xargs_echo_pipe.clone(),
            "/bin/bash",
            vec!["-c", find_grep_xargs_echo_pipe.as_str()],
        ),
        (
            "run find grep xargs pipe",
            find_grep_xargs_pipe.clone(),
            "/bin/bash",
            vec!["-c", find_grep_xargs_pipe.as_str()],
        ),
        (
            "run find grep wc pipe",
            find_grep_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", find_grep_wc_pipe.as_str()],
        ),
        (
            "run find grep head pipe",
            find_grep_head_pipe.clone(),
            "/bin/bash",
            vec!["-c", find_grep_head_pipe.as_str()],
        ),
        (
            "run find grep tail pipe",
            find_grep_tail_pipe.clone(),
            "/bin/bash",
            vec!["-c", find_grep_tail_pipe.as_str()],
        ),
        (
            "run find grep sort pipe",
            find_grep_sort_pipe.clone(),
            "/bin/bash",
            vec!["-c", find_grep_sort_pipe.as_str()],
        ),
        (
            "run find grep sort uniq wc pipe",
            find_grep_sort_uniq_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", find_grep_sort_uniq_wc_pipe.as_str()],
        ),
        (
            "run find grep sort xargs echo pipe",
            find_grep_sort_xargs_echo_pipe.clone(),
            "/bin/bash",
            vec!["-c", find_grep_sort_xargs_echo_pipe.as_str()],
        ),
        (
            "run find grep sort xargs pipe",
            find_grep_sort_xargs_pipe.clone(),
            "/bin/bash",
            vec!["-c", find_grep_sort_xargs_pipe.as_str()],
        ),
        (
            "run find grep sort xargs wc sort pipe",
            find_grep_sort_xargs_sort_pipe.clone(),
            "/bin/bash",
            vec!["-c", find_grep_sort_xargs_sort_pipe.as_str()],
        ),
        (
            "run find grep sort uniq xargs wc sort tail pipe",
            find_grep_sort_uniq_xargs_sort_tail_pipe.clone(),
            "/bin/bash",
            vec!["-c", find_grep_sort_uniq_xargs_sort_tail_pipe.as_str()],
        ),
        (
            "run find wc pipe",
            find_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", find_wc_pipe.as_str()],
        ),
        (
            "run find head pipe",
            find_head_pipe.clone(),
            "/bin/bash",
            vec!["-c", find_head_pipe.as_str()],
        ),
        (
            "run find tail pipe",
            find_tail_pipe.clone(),
            "/bin/bash",
            vec!["-c", find_tail_pipe.as_str()],
        ),
        (
            "run find sort pipe",
            find_sort_pipe.clone(),
            "/bin/bash",
            vec!["-c", find_sort_pipe.as_str()],
        ),
        (
            "run find sort uniq pipe",
            find_sort_uniq_pipe.clone(),
            "/bin/bash",
            vec!["-c", find_sort_uniq_pipe.as_str()],
        ),
        (
            "run find sort uniq wc pipe",
            find_sort_uniq_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", find_sort_uniq_wc_pipe.as_str()],
        ),
        (
            "run find sort uniq xargs wc pipe",
            find_sort_uniq_xargs_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", find_sort_uniq_xargs_wc_pipe.as_str()],
        ),
        (
            "run find sort uniq xargs wc sort wc pipe",
            find_sort_uniq_xargs_sort_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", find_sort_uniq_xargs_sort_wc_pipe.as_str()],
        ),
        (
            "run find sort uniq grep sort xargs wc pipe",
            find_sort_uniq_grep_sort_xargs_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", find_sort_uniq_grep_sort_xargs_wc_pipe.as_str()],
        ),
        (
            "run find sort wc pipe",
            find_sort_wc_pipe.clone(),
            "/bin/bash",
            vec!["-c", find_sort_wc_pipe.as_str()],
        ),
        (
            "run find sort xargs echo pipe",
            find_sort_xargs_echo_pipe.clone(),
            "/bin/bash",
            vec!["-c", find_sort_xargs_echo_pipe.as_str()],
        ),
        (
            "run find sort xargs wc pipe",
            find_sort_xargs_pipe.clone(),
            "/bin/bash",
            vec!["-c", find_sort_xargs_pipe.as_str()],
        ),
        (
            "run find sort xargs wc sort tail pipe",
            find_sort_xargs_sort_tail_pipe.clone(),
            "/bin/bash",
            vec!["-c", find_sort_xargs_sort_tail_pipe.as_str()],
        ),
        (
            "run find sort head pipe",
            find_sort_head_pipe.clone(),
            "/bin/bash",
            vec!["-c", find_sort_head_pipe.as_str()],
        ),
        (
            "run find sort tail pipe",
            find_sort_tail_pipe.clone(),
            "/bin/bash",
            vec!["-c", find_sort_tail_pipe.as_str()],
        ),
        (
            "run wc",
            format!("wc -l {}", fixture.wc_files().join(" ")),
            "/usr/bin/wc",
            wc_case.original_args.clone(),
        ),
        (
            "run wc bytes",
            format!("wc -c {}", fixture.wc_files().join(" ")),
            "/usr/bin/wc",
            wc_bytes_case.original_args.clone(),
        ),
        (
            "run wc words",
            format!("wc -w {}", fixture.wc_files().join(" ")),
            "/usr/bin/wc",
            wc_words_case.original_args.clone(),
        ),
        (
            "run echo wc bytes pipe",
            "echo alpha beta | wc -c".to_string(),
            "/bin/bash",
            vec!["-c", "echo alpha beta | wc -c"],
        ),
        (
            "run echo wc words pipe",
            "echo alpha beta | wc -w".to_string(),
            "/bin/bash",
            vec!["-c", "echo alpha beta | wc -w"],
        ),
        (
            "run printf wc bytes pipe",
            "printf '%s\\n' alpha beta gamma | wc -c".to_string(),
            "/bin/bash",
            vec!["-c", "printf '%s\\n' alpha beta gamma | wc -c"],
        ),
        (
            "run printf grep sort uniq wc words pipe",
            "printf '%s\\n' NEEDLE2 NEEDLE1 NEEDLE1 alpha | grep NEEDLE | sort | uniq | wc -w"
                .to_string(),
            "/bin/bash",
            vec![
                "-c",
                "printf '%s\\n' NEEDLE2 NEEDLE1 NEEDLE1 alpha | grep NEEDLE | sort | uniq | wc -w",
            ],
        ),
        (
            "run cat wc bytes pipe",
            cat_wc_bytes_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_wc_bytes_pipe.as_str()],
        ),
        (
            "run cat sort wc words pipe",
            cat_sort_wc_words_pipe.clone(),
            "/bin/bash",
            vec!["-c", cat_sort_wc_words_pipe.as_str()],
        ),
        (
            "run sort uniq wc bytes pipe",
            sort_uniq_wc_bytes_pipe.clone(),
            "/bin/bash",
            vec!["-c", sort_uniq_wc_bytes_pipe.as_str()],
        ),
        (
            "run head grep wc bytes pipe",
            head_grep_wc_bytes_pipe.clone(),
            "/bin/bash",
            vec!["-c", head_grep_wc_bytes_pipe.as_str()],
        ),
    ];

    for (name, command, original_program, original_args) in run_success_cases {
        assert_run_string_success_parity(&cap, name, &command, original_program, &original_args)?;
    }
    assert_run_string_stdin_success_parity(
        &cap,
        "run cut stdin wc pipe",
        &cut_stdin_wc_pipe,
        "/bin/bash",
        &["-c", &cut_stdin_wc_pipe],
        b"alpha,beta\nplain\ngamma,delta\n",
    )?;
    assert_run_string_stdin_success_parity(
        &cap,
        "run wc stdin wc pipe",
        &wc_stdin_wc_pipe,
        "/bin/bash",
        &["-c", &wc_stdin_wc_pipe],
        b"one two\nthree\n",
    )?;
    assert_run_string_stdin_success_parity(
        &cap,
        "run wc stdin head pipe",
        &wc_stdin_head_pipe,
        "/bin/bash",
        &["-c", &wc_stdin_head_pipe],
        b"one two\nthree\n",
    )?;
    assert_run_string_stdin_success_parity(
        &cap,
        "run wc stdin grep wc pipe",
        &wc_stdin_grep_wc_pipe,
        "/bin/bash",
        &["-c", &wc_stdin_grep_wc_pipe],
        b"one two\nthree\n",
    )?;
    assert_run_string_stdin_success_parity(
        &cap,
        "run wc stdin sort xargs pipe",
        &wc_stdin_sort_xargs_pipe,
        "/bin/bash",
        &["-c", &wc_stdin_sort_xargs_pipe],
        b"one two\nthree\n",
    )?;
    assert_run_string_stdin_success_parity(
        &cap,
        "run head stdin wc pipe",
        &head_stdin_wc_pipe,
        "/bin/bash",
        &["-c", &head_stdin_wc_pipe],
        b"one\ntwo\nthree\n",
    )?;
    assert_run_string_stdin_success_parity(
        &cap,
        "run tail stdin wc pipe",
        &tail_stdin_wc_pipe,
        "/bin/bash",
        &["-c", &tail_stdin_wc_pipe],
        b"one\ntwo\nthree\n",
    )?;
    assert_run_string_stdin_success_parity(
        &cap,
        "run xargs echo stdin wc pipe",
        &xargs_stdin_wc_pipe,
        "/bin/bash",
        &["-c", &xargs_stdin_wc_pipe],
        b"one two\nthree\n",
    )?;
    assert_run_string_stdin_success_parity(
        &cap,
        "run xargs default empty stdin wc pipe",
        &xargs_default_stdin_wc_pipe,
        "/bin/bash",
        &["-c", &xargs_default_stdin_wc_pipe],
        b"",
    )?;
    assert_run_string_stdin_success_parity(
        &cap,
        "run xargs n1 stdin wc pipe",
        &xargs_n1_stdin_wc_pipe,
        "/bin/bash",
        &["-c", &xargs_n1_stdin_wc_pipe],
        b"one two\nthree\n",
    )?;
    assert_run_string_stdin_success_parity(
        &cap,
        "run xargs n2 stdin wc pipe",
        &xargs_n2_stdin_wc_pipe,
        "/bin/bash",
        &["-c", &xargs_n2_stdin_wc_pipe],
        b"one two\nthree four\nfive\n",
    )?;
    assert_run_string_stdin_success_parity(
        &cap,
        "run xargs n1 stdin grep head pipe",
        &xargs_n1_grep_head_pipe,
        "/bin/bash",
        &["-c", &xargs_n1_grep_head_pipe],
        b"alpha NEEDLE\nbeta NEEDLE2\n",
    )?;
    assert_run_string_stdin_success_parity(
        &cap,
        "run xargs n2 stdin grep head pipe",
        &xargs_n2_grep_head_pipe,
        "/bin/bash",
        &["-c", &xargs_n2_grep_head_pipe],
        b"alpha NEEDLE\nbeta NEEDLE2\n",
    )?;
    assert_run_string_stdin_success_parity(
        &cap,
        "run xargs echo stdin grep pipe",
        &xargs_grep_pipe,
        "/bin/bash",
        &["-c", &xargs_grep_pipe],
        b"alpha NEEDLE\nbeta\n",
    )?;
    assert_run_string_stdin_success_parity(
        &cap,
        "run xargs echo stdin grep wc pipe",
        &xargs_grep_wc_pipe,
        "/bin/bash",
        &["-c", &xargs_grep_wc_pipe],
        b"alpha NEEDLE\nbeta\n",
    )?;
    assert_run_string_stdin_success_parity(
        &cap,
        "run xargs echo stdin grep head pipe",
        &xargs_grep_head_pipe,
        "/bin/bash",
        &["-c", &xargs_grep_head_pipe],
        b"alpha NEEDLE\nbeta\n",
    )?;
    assert_run_string_success_parity(
        &cap,
        "run command v missing",
        "command -v __cap_missing_command__",
        "/bin/bash",
        &["-c", "command -v __cap_missing_command__"],
    )?;

    let missing = temp.path().join("missing-target").display().to_string();
    for (name, command) in [
        ("run missing cat wc pipe", format!("cat {missing} | wc -l")),
        (
            "run missing cat head pipe",
            format!("cat {missing} | head -n 3"),
        ),
        (
            "run missing cat tail pipe",
            format!("cat {missing} | tail -n 3"),
        ),
        (
            "run missing cat grep pipe",
            format!("cat {missing} | grep NEEDLE"),
        ),
        (
            "run missing cat cut pipe",
            format!("cat {missing} | cut -d, -f1"),
        ),
        (
            "run missing cat tr pipe",
            format!("cat {missing} | tr a-z A-Z"),
        ),
        (
            "run missing grep wc pipe",
            format!("grep -R NEEDLE {missing} | wc -l"),
        ),
        (
            "run missing grep tail pipe",
            format!("grep -R NEEDLE {missing} | tail -n 3"),
        ),
        (
            "run missing grep sort wc pipe",
            format!("grep -R NEEDLE {missing} | sort | wc -l"),
        ),
        (
            "run missing find wc pipe",
            format!("find {missing} -type f -name '*.txt' | wc -l"),
        ),
        (
            "run missing find head pipe",
            format!("find {missing} -type f -name '*.txt' | head -n 3"),
        ),
    ] {
        assert_run_string_success_parity(&cap, name, &command, "/bin/bash", &["-c", &command])?;
    }

    let error_cases = [
        Case::new(
            "ls",
            vec!["ls", missing.as_str()],
            "/bin/ls",
            vec![missing.as_str()],
        ),
        Case::new(
            "cat",
            vec!["cat", missing.as_str()],
            "/bin/cat",
            vec![missing.as_str()],
        ),
        Case::new(
            "uniq",
            vec!["uniq", missing.as_str()],
            "/usr/bin/uniq",
            vec![missing.as_str()],
        ),
        Case::new(
            "find",
            vec!["find", missing.as_str(), "-type", "f", "-name", "*.txt"],
            "/usr/bin/find",
            vec![missing.as_str(), "-type", "f", "-name", "*.txt"],
        ),
        Case::new(
            "du",
            vec!["du", "-sk", missing.as_str()],
            "/usr/bin/du",
            vec!["-sk", missing.as_str()],
        ),
        Case::new(
            "sort",
            vec!["sort", missing.as_str()],
            "/usr/bin/sort",
            vec![missing.as_str()],
        ),
        Case::new(
            "cut",
            vec!["cut", "-d,", "-f1", missing.as_str()],
            "/usr/bin/cut",
            vec!["-d,", "-f1", missing.as_str()],
        ),
        Case::new(
            "sed",
            vec!["sed", "-n", "1,2p", missing.as_str()],
            "/usr/bin/sed",
            vec!["-n", "1,2p", missing.as_str()],
        ),
        Case::new(
            "grep",
            vec!["grep", "-R", "NEEDLE", missing.as_str()],
            "/usr/bin/grep",
            vec!["-R", "NEEDLE", missing.as_str()],
        ),
        Case::new(
            "wc",
            vec!["wc", "-l", missing.as_str()],
            "/usr/bin/wc",
            vec!["-l", missing.as_str()],
        ),
    ];

    for case in error_cases {
        assert_error_parity(&cap, &case, &missing)?;
    }

    let run_error_cases = [
        (
            "run ls",
            format!("ls {}", missing),
            "/bin/ls",
            vec![missing.as_str()],
        ),
        (
            "run cat",
            format!("cat {}", missing),
            "/bin/cat",
            vec![missing.as_str()],
        ),
        (
            "run uniq",
            format!("uniq {}", missing),
            "/usr/bin/uniq",
            vec![missing.as_str()],
        ),
        (
            "run find",
            format!("find {} -type f -name '*.txt'", missing),
            "/usr/bin/find",
            vec![missing.as_str(), "-type", "f", "-name", "*.txt"],
        ),
        (
            "run du",
            format!("du -sk {}", missing),
            "/usr/bin/du",
            vec!["-sk", missing.as_str()],
        ),
        (
            "run sort",
            format!("sort {}", missing),
            "/usr/bin/sort",
            vec![missing.as_str()],
        ),
        (
            "run cut",
            format!("cut -d, -f1 {}", missing),
            "/usr/bin/cut",
            vec!["-d,", "-f1", missing.as_str()],
        ),
        (
            "run sed",
            format!("sed -n 1,2p {}", missing),
            "/usr/bin/sed",
            vec!["-n", "1,2p", missing.as_str()],
        ),
        (
            "run grep",
            format!("grep -R NEEDLE {}", missing),
            "/usr/bin/grep",
            vec!["-R", "NEEDLE", missing.as_str()],
        ),
        (
            "run wc",
            format!("wc -l {}", missing),
            "/usr/bin/wc",
            vec!["-l", missing.as_str()],
        ),
    ];

    for (name, command, original_program, original_args) in run_error_cases {
        assert_run_string_error_parity(
            &cap,
            name,
            &command,
            original_program,
            &original_args,
            &missing,
        )?;
    }

    let no_match = Case::new(
        "grep",
        vec!["grep", "-R", "ABSENT", fixture.grep_root()],
        "/usr/bin/grep",
        vec!["-R", "ABSENT", fixture.grep_root()],
    );
    assert_quiet_nonzero_parity(&cap, &no_match)?;
    let seq_grep_no_match = "seq 1 3 | grep 9";
    let seq_grep_no_match = Case::new(
        "run-seq-grep-no-match",
        vec!["run", seq_grep_no_match],
        "/bin/bash",
        vec!["-c", seq_grep_no_match],
    );
    assert_quiet_nonzero_parity(&cap, &seq_grep_no_match)?;
    let printf_grep_no_match = "printf '%s\\n' alpha beta | grep NEEDLE";
    let printf_grep_no_match = Case::new(
        "run-printf-grep-no-match",
        vec!["run", printf_grep_no_match],
        "/bin/bash",
        vec!["-c", printf_grep_no_match],
    );
    assert_quiet_nonzero_parity(&cap, &printf_grep_no_match)?;
    let false_grep_no_match = Case::new(
        "run-false-grep-no-match",
        vec!["run", false_grep_pipe.as_str()],
        "/bin/bash",
        vec!["-c", false_grep_pipe.as_str()],
    );
    assert_quiet_nonzero_parity(&cap, &false_grep_no_match)?;
    let test_grep_no_match = Case::new(
        "run-test-grep-no-match",
        vec!["run", test_grep_pipe.as_str()],
        "/bin/bash",
        vec!["-c", test_grep_pipe.as_str()],
    );
    assert_quiet_nonzero_parity(&cap, &test_grep_no_match)?;
    let wc_grep_no_match = Case::new(
        "run-wc-grep-no-match",
        vec!["run", wc_grep_pipe.as_str()],
        "/bin/bash",
        vec!["-c", wc_grep_pipe.as_str()],
    );
    assert_quiet_nonzero_parity(&cap, &wc_grep_no_match)?;
    let pwd_grep_no_match = "pwd | grep CAP_PATTERN_THAT_SHOULD_NOT_MATCH";
    let pwd_grep_no_match = Case::new(
        "run-pwd-grep-no-match",
        vec!["run", pwd_grep_no_match],
        "/bin/bash",
        vec!["-c", pwd_grep_no_match],
    );
    assert_quiet_nonzero_parity(&cap, &pwd_grep_no_match)?;
    let sed_grep_no_match = format!(
        "sed -n 1,12p {} | grep CAP_PATTERN_THAT_SHOULD_NOT_MATCH",
        fixture.sed_file()
    );
    let sed_grep_no_match = Case::new(
        "run-sed-grep-no-match",
        vec!["run", sed_grep_no_match.as_str()],
        "/bin/bash",
        vec!["-c", sed_grep_no_match.as_str()],
    );
    assert_quiet_nonzero_parity(&cap, &sed_grep_no_match)?;
    let cut_grep_no_match = format!(
        "cut -d, -f1 {} | grep CAP_PATTERN_THAT_SHOULD_NOT_MATCH",
        fixture.cut_file()
    );
    let cut_grep_no_match = Case::new(
        "run-cut-grep-no-match",
        vec!["run", cut_grep_no_match.as_str()],
        "/bin/bash",
        vec!["-c", cut_grep_no_match.as_str()],
    );
    assert_quiet_nonzero_parity(&cap, &cut_grep_no_match)?;
    let cat_cut_grep_no_match = format!(
        "cat {} | cut -d, -f1 | grep CAP_PATTERN_THAT_SHOULD_NOT_MATCH",
        fixture.cut_file()
    );
    let cat_cut_grep_no_match = Case::new(
        "run-cat-cut-grep-no-match",
        vec!["run", cat_cut_grep_no_match.as_str()],
        "/bin/bash",
        vec!["-c", cat_cut_grep_no_match.as_str()],
    );
    assert_quiet_nonzero_parity(&cap, &cat_cut_grep_no_match)?;
    let cat_tr_grep_no_match = format!(
        "cat {} | tr a-z A-Z | grep CAP_PATTERN_THAT_SHOULD_NOT_MATCH",
        fixture.window_file()
    );
    let cat_tr_grep_no_match = Case::new(
        "run-cat-tr-grep-no-match",
        vec!["run", cat_tr_grep_no_match.as_str()],
        "/bin/bash",
        vec!["-c", cat_tr_grep_no_match.as_str()],
    );
    assert_quiet_nonzero_parity(&cap, &cat_tr_grep_no_match)?;
    let cat_uniq_grep_no_match = format!(
        "cat {} | uniq | grep CAP_PATTERN_THAT_SHOULD_NOT_MATCH",
        fixture.uniq_file()
    );
    let cat_uniq_grep_no_match = Case::new(
        "run-cat-uniq-grep-no-match",
        vec!["run", cat_uniq_grep_no_match.as_str()],
        "/bin/bash",
        vec!["-c", cat_uniq_grep_no_match.as_str()],
    );
    assert_quiet_nonzero_parity(&cap, &cat_uniq_grep_no_match)?;
    let uniq_grep_no_match = format!(
        "uniq {} | grep CAP_PATTERN_THAT_SHOULD_NOT_MATCH",
        fixture.uniq_file()
    );
    let uniq_grep_no_match = Case::new(
        "run-uniq-grep-no-match",
        vec!["run", uniq_grep_no_match.as_str()],
        "/bin/bash",
        vec!["-c", uniq_grep_no_match.as_str()],
    );
    assert_quiet_nonzero_parity(&cap, &uniq_grep_no_match)?;
    let sort_uniq_grep_no_match = format!(
        "sort {} | uniq | grep CAP_PATTERN_THAT_SHOULD_NOT_MATCH",
        fixture.uniq_file()
    );
    let sort_uniq_grep_no_match = Case::new(
        "run-sort-uniq-grep-no-match",
        vec!["run", sort_uniq_grep_no_match.as_str()],
        "/bin/bash",
        vec!["-c", sort_uniq_grep_no_match.as_str()],
    );
    assert_quiet_nonzero_parity(&cap, &sort_uniq_grep_no_match)?;
    let cat_sort_uniq_grep_no_match = format!(
        "cat {} | sort | uniq | grep CAP_PATTERN_THAT_SHOULD_NOT_MATCH",
        fixture.sort_file()
    );
    let cat_sort_uniq_grep_no_match = Case::new(
        "run-cat-sort-uniq-grep-no-match",
        vec!["run", cat_sort_uniq_grep_no_match.as_str()],
        "/bin/bash",
        vec!["-c", cat_sort_uniq_grep_no_match.as_str()],
    );
    assert_quiet_nonzero_parity(&cap, &cat_sort_uniq_grep_no_match)?;
    let ls_sort_uniq_grep_no_match = format!(
        "ls -1 {} | sort | uniq | grep CAP_PATTERN_THAT_SHOULD_NOT_MATCH",
        fixture.list_dir()
    );
    let ls_sort_uniq_grep_no_match = Case::new(
        "run-ls-sort-uniq-grep-no-match",
        vec!["run", ls_sort_uniq_grep_no_match.as_str()],
        "/bin/bash",
        vec!["-c", ls_sort_uniq_grep_no_match.as_str()],
    );
    assert_quiet_nonzero_parity(&cap, &ls_sort_uniq_grep_no_match)?;
    let find_grep_no_match = format!(
        "find {} -type f | grep CAP_PATTERN_THAT_SHOULD_NOT_MATCH",
        fixture.find_root()
    );
    let find_grep_no_match = Case::new(
        "run-find-grep-no-match",
        vec!["run", find_grep_no_match.as_str()],
        "/bin/bash",
        vec!["-c", find_grep_no_match.as_str()],
    );
    assert_quiet_nonzero_parity(&cap, &find_grep_no_match)?;
    let find_sort_uniq_grep_no_match = format!(
        "find {} -type f | sort | uniq | grep CAP_PATTERN_THAT_SHOULD_NOT_MATCH",
        fixture.find_root()
    );
    let find_sort_uniq_grep_no_match = Case::new(
        "run-find-sort-uniq-grep-no-match",
        vec!["run", find_sort_uniq_grep_no_match.as_str()],
        "/bin/bash",
        vec!["-c", find_sort_uniq_grep_no_match.as_str()],
    );
    assert_quiet_nonzero_parity(&cap, &find_sort_uniq_grep_no_match)?;
    for case in [
        Case::new(
            "test-negated-existing",
            vec!["test", "!", "-e", fixture.cat_file()],
            "/bin/test",
            vec!["!", "-e", fixture.cat_file()],
        ),
        Case::new(
            "test-int-false",
            vec!["test", "1", "-gt", "3"],
            "/bin/test",
            vec!["1", "-gt", "3"],
        ),
        Case::new(
            "bracket-file-is-dir-false",
            vec!["[", "-d", fixture.cat_file(), "]"],
            "/bin/[",
            vec!["-d", fixture.cat_file(), "]"],
        ),
        Case::new(
            "which-missing",
            vec!["which", "__cap_missing_command__"],
            "/usr/bin/which",
            vec!["__cap_missing_command__"],
        ),
        Case::new(
            "which-all-missing",
            vec!["which", "-a", "__cap_missing_command__"],
            "/usr/bin/which",
            vec!["-a", "__cap_missing_command__"],
        ),
        Case::new(
            "printenv-missing",
            vec!["printenv", "__CAP_MISSING_ENV__"],
            "/usr/bin/printenv",
            vec!["__CAP_MISSING_ENV__"],
        ),
    ] {
        assert_quiet_nonzero_parity(&cap, &case)?;
    }

    Ok(())
}

#[test]
fn installed_frontend_exposes_standard_agent_commands() -> Result<()> {
    let temp = tempfile::tempdir().context("create frontend tempdir")?;
    let bin_dir = temp.path().join("bin");
    fs::create_dir(&bin_dir)?;
    let cap = build_cap_frontend(&bin_dir)?;

    let help = run(&cap, &["--help"])?;
    assert!(
        help.status.success(),
        "cap --help failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&help.stdout),
        String::from_utf8_lossy(&help.stderr)
    );
    let help_stdout = String::from_utf8_lossy(&help.stdout);
    assert!(help_stdout.contains("Usage: cap "), "{help_stdout}");
    assert!(!help_stdout.contains("Usage: cap-full "), "{help_stdout}");
    for verb in ["llm", "upgrade", "issue", "report-issue"] {
        assert!(
            help_stdout.contains(verb),
            "installed cap help missing {verb}:\n{help_stdout}"
        );
    }

    let llm = run(&cap, &["llm", "--topic", "outline", "--format", "json"])?;
    assert!(
        llm.status.success(),
        "cap llm failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&llm.stdout),
        String::from_utf8_lossy(&llm.stderr)
    );
    let llm_stdout = String::from_utf8_lossy(&llm.stdout);
    assert!(llm_stdout.contains("\"project\": \"cap\""), "{llm_stdout}");
    assert!(llm_stdout.contains("\"id\": \"workflow\""), "{llm_stdout}");

    for args in [
        vec![
            "issue",
            "create",
            "--title",
            "cap: smoke",
            "--dry-run",
            "smoke",
        ],
        vec![
            "report-issue",
            "--title",
            "cap: smoke",
            "--dry-run",
            "smoke",
        ],
    ] {
        let out = run(&cap, &args)?;
        assert!(
            out.status.success(),
            "cap {} failed:\nstdout:\n{}\nstderr:\n{}",
            args.join(" "),
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(stdout.contains("labels: app:cap"), "{stdout}");
        assert!(stdout.contains("## Diagnostics"), "{stdout}");
    }

    let passthrough = run(&cap, &["sh", "-c", "printf cap-path-ok"])?;
    assert!(
        passthrough.status.success(),
        "cap passthrough failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&passthrough.stdout),
        String::from_utf8_lossy(&passthrough.stderr)
    );
    assert_eq!(passthrough.stdout, b"cap-path-ok");

    Ok(())
}

struct Case<'a> {
    name: &'a str,
    cap_args: Vec<&'a str>,
    original_program: &'a str,
    original_args: Vec<&'a str>,
}

impl<'a> Case<'a> {
    fn new(
        name: &'a str,
        cap_args: Vec<&'a str>,
        original_program: &'a str,
        original_args: Vec<&'a str>,
    ) -> Self {
        Self {
            name,
            cap_args,
            original_program,
            original_args,
        }
    }
}

fn assert_success_parity(cap: &Path, case: &Case<'_>) -> Result<()> {
    let cap_out = run(cap, &case.cap_args)?;
    let original_out = run(Path::new(case.original_program), &case.original_args)?;
    assert_eq!(
        exit_code(&cap_out),
        exit_code(&original_out),
        "{} exit",
        case.name
    );
    assert_eq!(cap_out.stdout, original_out.stdout, "{} stdout", case.name);
    assert_eq!(cap_out.stderr, original_out.stderr, "{} stderr", case.name);
    Ok(())
}

fn assert_error_parity(cap: &Path, case: &Case<'_>, missing: &str) -> Result<()> {
    let cap_out = run(cap, &case.cap_args)?;
    let original_out = run(Path::new(case.original_program), &case.original_args)?;
    assert_ne!(
        exit_code(&original_out),
        Some(0),
        "{} original must fail",
        case.name
    );
    assert_eq!(
        exit_code(&cap_out),
        exit_code(&original_out),
        "{} exit",
        case.name
    );
    assert_eq!(cap_out.stdout, original_out.stdout, "{} stdout", case.name);
    assert!(
        !cap_out.stderr.is_empty(),
        "{} cap stderr should explain the failure",
        case.name
    );
    let cap_stderr = String::from_utf8_lossy(&cap_out.stderr);
    assert!(
        cap_stderr.contains(case.name),
        "{} stderr should name the command: {cap_stderr}",
        case.name
    );
    if case.name != "sort" {
        assert!(
            cap_stderr.contains(missing),
            "{} stderr should name the failed path: {cap_stderr}",
            case.name
        );
    }
    Ok(())
}

fn assert_quiet_nonzero_parity(cap: &Path, case: &Case<'_>) -> Result<()> {
    let cap_out = run(cap, &case.cap_args)?;
    let original_out = run(Path::new(case.original_program), &case.original_args)?;
    assert_eq!(
        exit_code(&cap_out),
        exit_code(&original_out),
        "{} exit",
        case.name
    );
    assert_eq!(cap_out.stdout, original_out.stdout, "{} stdout", case.name);
    assert_eq!(cap_out.stderr, original_out.stderr, "{} stderr", case.name);
    Ok(())
}

fn assert_run_string_success_parity(
    cap: &Path,
    name: &str,
    command: &str,
    original_program: &str,
    original_args: &[&str],
) -> Result<()> {
    let cap_out = run(cap, &["run", command])?;
    let original_out = run(Path::new(original_program), original_args)?;
    assert_eq!(exit_code(&cap_out), exit_code(&original_out), "{name} exit");
    assert_eq!(cap_out.stdout, original_out.stdout, "{name} stdout");
    assert_eq!(cap_out.stderr, original_out.stderr, "{name} stderr");
    Ok(())
}

fn assert_stdin_success_parity(
    cap: &Path,
    name: &str,
    cap_args: &[&str],
    original_program: &str,
    original_args: &[&str],
    input: &[u8],
) -> Result<()> {
    let cap_out = run_with_stdin(cap, cap_args, input)?;
    let original_out = run_with_stdin(Path::new(original_program), original_args, input)?;
    assert_eq!(exit_code(&cap_out), exit_code(&original_out), "{name} exit");
    assert_eq!(cap_out.stdout, original_out.stdout, "{name} stdout");
    assert_eq!(cap_out.stderr, original_out.stderr, "{name} stderr");
    Ok(())
}

fn assert_run_string_stdin_success_parity(
    cap: &Path,
    name: &str,
    command: &str,
    original_program: &str,
    original_args: &[&str],
    input: &[u8],
) -> Result<()> {
    let cap_out = run_with_stdin(cap, &["run", command], input)?;
    let original_out = run_with_stdin(Path::new(original_program), original_args, input)?;
    assert_eq!(exit_code(&cap_out), exit_code(&original_out), "{name} exit");
    assert_eq!(cap_out.stdout, original_out.stdout, "{name} stdout");
    assert_eq!(cap_out.stderr, original_out.stderr, "{name} stderr");
    Ok(())
}

fn assert_run_string_error_parity(
    cap: &Path,
    name: &str,
    command: &str,
    original_program: &str,
    original_args: &[&str],
    missing: &str,
) -> Result<()> {
    let cap_out = run(cap, &["run", command])?;
    let original_out = run(Path::new(original_program), original_args)?;
    assert_ne!(
        exit_code(&original_out),
        Some(0),
        "{name} original must fail"
    );
    assert_eq!(exit_code(&cap_out), exit_code(&original_out), "{name} exit");
    assert_eq!(cap_out.stdout, original_out.stdout, "{name} stdout");
    assert!(
        !cap_out.stderr.is_empty(),
        "{name} cap stderr should explain the failure"
    );
    let cap_stderr = String::from_utf8_lossy(&cap_out.stderr);
    let command_name = name.strip_prefix("run ").unwrap_or(name);
    assert!(
        cap_stderr.contains(command_name),
        "{name} stderr should name the command: {cap_stderr}",
    );
    if command_name != "sort" {
        assert!(
            cap_stderr.contains(missing),
            "{name} stderr should name the failed path: {cap_stderr}",
        );
    }
    Ok(())
}

fn run(program: &Path, args: &[&str]) -> Result<Output> {
    Command::new(program)
        .args(args)
        .output()
        .with_context(|| format!("run {} {}", program.display(), args.join(" ")))
}

fn run_with_stdin(program: &Path, args: &[&str], input: &[u8]) -> Result<Output> {
    let mut child = Command::new(program)
        .args(args)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .with_context(|| format!("spawn {} {}", program.display(), args.join(" ")))?;
    child
        .stdin
        .as_mut()
        .context("child stdin missing")?
        .write_all(input)?;
    child
        .wait_with_output()
        .with_context(|| format!("wait {} {}", program.display(), args.join(" ")))
}

fn exit_code(output: &Output) -> Option<i32> {
    output.status.code()
}

fn build_cap_frontend(bin_dir: &Path) -> Result<PathBuf> {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let cap = bin_dir.join(format!("cap{}", std::env::consts::EXE_SUFFIX));
    let fast = bin_dir.join(format!("cap-fast{}", std::env::consts::EXE_SUFFIX));
    let full = bin_dir.join(format!("cap-full{}", std::env::consts::EXE_SUFFIX));
    fs::copy(cap_full_binary()?, &full).context("copy cap-full sibling")?;

    let strip_flag = if cfg!(target_os = "macos") {
        "-Wl,-dead_strip"
    } else {
        "-Wl,--gc-sections"
    };
    let c_flags = [
        "-Oz",
        "-ffunction-sections",
        "-fdata-sections",
        "-fno-stack-protector",
        "-fno-unwind-tables",
        "-fno-asynchronous-unwind-tables",
        strip_flag,
    ];
    compile_c(&manifest.join("src/cap_fast_frontend.c"), &fast, &c_flags)?;

    let mut frontend_flags = c_flags.to_vec();
    if cfg!(target_os = "macos") && cfg!(target_arch = "aarch64") {
        frontend_flags.extend([
            "-ffreestanding",
            "-fno-builtin",
            "-nostartfiles",
            "-Wl,-e,_start",
        ]);
    }
    compile_c(&manifest.join("src/cap_frontend.c"), &cap, &frontend_flags)?;

    if cfg!(target_os = "macos") {
        let _ = Command::new("codesign")
            .args(["-s", "-", "-f", "--options", "runtime"])
            .arg(&cap)
            .status();
        let _ = Command::new("codesign")
            .args(["-s", "-", "-f", "--options", "runtime"])
            .arg(&fast)
            .status();
        let _ = Command::new("codesign")
            .args(["-s", "-", "-f"])
            .arg(&full)
            .status();
    }

    Ok(cap)
}

fn compile_c(source: &Path, out: &Path, flags: &[&str]) -> Result<()> {
    let cc = std::env::var("CC").unwrap_or_else(|_| "/usr/bin/cc".to_string());
    let status = Command::new(cc)
        .args(flags)
        .arg(source)
        .arg("-o")
        .arg(out)
        .status()
        .with_context(|| format!("compile {}", source.display()))?;
    if !status.success() {
        bail!("compile {} failed with {status}", source.display());
    }
    Ok(())
}

fn cap_full_binary() -> Result<PathBuf> {
    if let Some(path) = option_env!("CARGO_BIN_EXE_cap-full") {
        let path = PathBuf::from(path);
        if path.is_file() {
            return Ok(path);
        }
    }

    let current = std::env::current_exe().context("resolve test executable")?;
    let deps = current
        .parent()
        .context("test executable has no deps directory")?;
    let profile = deps
        .parent()
        .context("test executable has no profile directory")?;
    let candidate = profile.join(format!("cap-full{}", std::env::consts::EXE_SUFFIX));
    if candidate.is_file() {
        return Ok(candidate);
    }
    bail!("could not locate cap-full binary next to test profile")
}

struct Fixture {
    list_dir: String,
    basename_path: String,
    cat_file: String,
    window_file: String,
    no_newline_file: String,
    mkdir_existing: String,
    touch_file: String,
    touch_dir: String,
    uniq_file: String,
    find_root: String,
    du_root: String,
    sort_file: String,
    cut_file: String,
    sed_file: String,
    grep_root: String,
    grep_file: String,
    wc_files: Vec<String>,
    xargs_wc_file: String,
    awk_xargs_wc_file: String,
}

/// @spec apps/cap/tech-design/semantic/source/projects-cap-tests-behavior-cap-command-replacement-parity-rs.md#source
impl Fixture {
    fn create(root: &Path) -> Result<Self> {
        let data = root.join("data");
        fs::create_dir(&data)?;

        let list_dir = data.join("list");
        fs::create_dir(&list_dir)?;
        for idx in 0..1024 {
            fs::write(list_dir.join(format!("item-{idx:04}.txt")), b"x\n")?;
        }
        fs::write(list_dir.join(".hidden-alpha"), b"x\n")?;
        fs::write(list_dir.join(".hidden-beta"), b"x\n")?;

        let cat_file = data.join("cat.txt");
        fs::write(&cat_file, b"alpha\nbeta\n")?;
        let basename_path = data.join("nested/example.txt");
        fs::create_dir_all(basename_path.parent().unwrap())?;
        fs::write(&basename_path, b"basename\n")?;
        let window_file = data.join("window.txt");
        fs::write(&window_file, b"one\ntwo\nthree\nfour\nfive\n")?;
        let no_newline_file = data.join("no-newline.txt");
        fs::write(&no_newline_file, b"alpha\nNEEDLE")?;
        let mkdir_existing = data.join("mkdir/existing/deep");
        fs::create_dir_all(&mkdir_existing)?;
        let touch_file = data.join("touch-existing.txt");
        fs::write(&touch_file, b"touch\n")?;
        let touch_dir = data.join("touch-dir");
        fs::create_dir(&touch_dir)?;

        let uniq_file = data.join("uniq.txt");
        fs::write(&uniq_file, b"same\nsame\nnext\nnext\nsame\n")?;

        let find_root = data.join("find");
        fs::create_dir(&find_root)?;
        for idx in 0..512 {
            fs::write(find_root.join(format!("only-{idx:04}.txt")), b"found\n")?;
            fs::write(find_root.join(format!("source-{idx:04}.rs")), b"found\n")?;
        }
        let nested_find = find_root.join("nested");
        fs::create_dir(&nested_find)?;
        fs::write(nested_find.join("nested-only.txt"), b"found\n")?;
        fs::write(nested_find.join("nested-source-00.rs"), b"found\n")?;

        let du_root = data.join("du");
        fs::create_dir(&du_root)?;
        fs::write(du_root.join("payload.bin"), vec![b'x'; 16 * 1024])?;

        let sort_file = data.join("sort.txt");
        let mut sort = fs::File::create(&sort_file)?;
        for idx in (0..120_000).rev() {
            writeln!(sort, "line-{idx:06}")?;
        }

        let cut_file = data.join("cut.csv");
        let mut cut = fs::File::create(&cut_file)?;
        for idx in 0..4096 {
            if idx % 97 == 0 {
                writeln!(cut, "plain-{idx:04}")?;
            } else {
                writeln!(cut, "field-{idx:04},value-{idx:04},tail-{idx:04}")?;
            }
        }

        let sed_file = data.join("sed.txt");
        let mut sed = fs::File::create(&sed_file)?;
        for idx in 0..1100 {
            if idx == 17 {
                writeln!(sed)?;
            }
            writeln!(sed, "line {idx:04}")?;
        }

        let grep_root = data.join("grep");
        fs::create_dir(&grep_root)?;
        for idx in 0..64 {
            fs::write(
                grep_root.join(format!("match-{idx:04}.txt")),
                b"plain\nNEEDLE here\n",
            )?;
        }
        let grep_file = data.join("grep-file.txt");
        fs::write(
            &grep_file,
            b"plain\nNEEDLE beta\nother\nNEEDLE alpha\nNEEDLE beta\n",
        )?;
        let wc_root = data.join("wc");
        fs::create_dir(&wc_root)?;
        let mut wc_files = Vec::new();
        for idx in 0..64 {
            let file = wc_root.join(format!("count-{idx:04}.txt"));
            fs::write(&file, b"one\ntwo\n")?;
            wc_files.push(path_string(&file));
        }
        let xargs_wc_file = data.join("xargs-wc-paths.txt");
        fs::write(
            &xargs_wc_file,
            format!("{}\n{}\n", wc_files[1], wc_files[0]),
        )?;
        let awk_xargs_wc_file = data.join("awk-xargs-wc-paths.txt");
        fs::write(
            &awk_xargs_wc_file,
            format!("{} NEEDLE\n{} NEEDLE\n", wc_files[1], wc_files[0]),
        )?;

        Ok(Self {
            list_dir: path_string(&list_dir),
            basename_path: path_string(&basename_path),
            cat_file: path_string(&cat_file),
            window_file: path_string(&window_file),
            no_newline_file: path_string(&no_newline_file),
            mkdir_existing: path_string(&mkdir_existing),
            touch_file: path_string(&touch_file),
            touch_dir: path_string(&touch_dir),
            uniq_file: path_string(&uniq_file),
            find_root: path_string(&find_root),
            du_root: path_string(&du_root),
            sort_file: path_string(&sort_file),
            cut_file: path_string(&cut_file),
            sed_file: path_string(&sed_file),
            grep_root: path_string(&grep_root),
            grep_file: path_string(&grep_file),
            wc_files,
            xargs_wc_file: path_string(&xargs_wc_file),
            awk_xargs_wc_file: path_string(&awk_xargs_wc_file),
        })
    }

    fn list_dir(&self) -> &str {
        &self.list_dir
    }
    fn basename_path(&self) -> &str {
        &self.basename_path
    }
    fn cat_file(&self) -> &str {
        &self.cat_file
    }
    fn window_file(&self) -> &str {
        &self.window_file
    }
    fn no_newline_file(&self) -> &str {
        &self.no_newline_file
    }
    fn mkdir_existing(&self) -> &str {
        &self.mkdir_existing
    }
    fn touch_file(&self) -> &str {
        &self.touch_file
    }
    fn touch_dir(&self) -> &str {
        &self.touch_dir
    }
    fn uniq_file(&self) -> &str {
        &self.uniq_file
    }
    fn find_root(&self) -> &str {
        &self.find_root
    }
    fn du_root(&self) -> &str {
        &self.du_root
    }
    fn sort_file(&self) -> &str {
        &self.sort_file
    }
    fn cut_file(&self) -> &str {
        &self.cut_file
    }
    fn sed_file(&self) -> &str {
        &self.sed_file
    }
    fn grep_root(&self) -> &str {
        &self.grep_root
    }
    fn grep_file(&self) -> &str {
        &self.grep_file
    }
    fn wc_files(&self) -> &[String] {
        &self.wc_files
    }
    fn xargs_wc_file(&self) -> &str {
        &self.xargs_wc_file
    }
    fn awk_xargs_wc_file(&self) -> &str {
        &self.awk_xargs_wc_file
    }
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().to_string()
}
// CODEGEN-END
