// SPEC-MANAGED: projects/agentic-workflow/tech-design/surface/validate/tests/td_no_merge_test.md#source
// CODEGEN-BEGIN
//! Regression tests proving the removed TD merge command is no longer part of the CLI surface.

use agentic_workflow::cli::commands::Commands;
use clap::{CommandFactory, Parser};

#[derive(Parser)]
#[command(name = "aw")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[test]
fn test_td_merge_subcommand_is_removed() {
    let cmd = Cli::command();
    let td = cmd.find_subcommand("td").expect("td namespace");
    assert!(
        td.find_subcommand("merge").is_none(),
        "removed TD merge command must not be registered"
    );
}

#[test]
fn test_td_merge_parse_fails() {
    let err = match Cli::try_parse_from(["aw", "td", "merge", "4124"]) {
        Ok(_) => panic!("removed TD merge command unexpectedly parsed"),
        Err(err) => err,
    };
    assert!(
        err.to_string().contains("unrecognized subcommand 'merge'"),
        "unexpected parse error: {err}"
    );
}

#[test]
fn test_cb_code_check_is_terminal_lifecycle_trailer() {
    use agentic_workflow::issues::types::lifecycle_trailer;

    assert_eq!(lifecycle_trailer::CB_CODE_CHECK, "Cb-CodeCheck");
}

// CODEGEN-END
