---
id: jet-cli-add-issue-comment-auto-reopen-follow-up
summary: >
  Jet's `jet issue` group gains a `comment` action that wires
  `jet issue comment <number> [message...]` to the shared
  `cli_std::issue::comment(CommentOptions)` API in `libs/cli-std`, so a
  follow-up note on a closed issue reopens it first and posts a
  diagnostics-rich comment body, without duplicating GitHub API logic in
  jet's own CLI. `search`/`view`/`create` behavior is preserved unchanged;
  `--dry-run` prints the target issue, resolved state (open), and the
  assembled diagnostics comment without any network mutation.
capability_refs:
  - id: jet-cli-standard-commands
    role: primary
    gap: jet-cli-add-issue-comment-auto-reopen-follow-up
    claim: jet-cli-add-issue-comment-auto-reopen-follow-up
    coverage: full
    rationale: "Defines the bounded jet issue comment auto-reopen follow-up requested by WI #928, pinned to the 'Jet CLI Add Issue Comment Auto-Reopen Follow-Up' work root under the jet-cli-standard-commands capability (jet's llm/upgrade/issue standard CLI command convention)."
fill_sections: [logic, config, unit-test, changes]
---

# jet CLI: add issue comment auto-reopen follow-up

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: jet-cli-issue-comment-auto-reopen-flow
entry: parse_args
nodes:
  parse_args:
    kind: start
    label: "jet issue comment <number> [message...] [--dry-run]\nparsed by standard_cli::issue_command()"
  build_options:
    kind: process
    label: "Build cli_std::issue::CommentOptions { number, message,\nrepo: None, dry_run, yes: true }\nfrom parsed clap ArgMatches"
  dispatch_shared:
    kind: process
    label: "standard_cli::run_issue() dispatches to\ncli_std::issue::comment(&TOOL, opts)\n(no GitHub API logic duplicated in jet)"
  fetch_state:
    kind: process
    label: "cli-std fetches the target issue's current state\nvia GitHub API (behind the online feature)"
  is_closed:
    kind: decision
    label: "Is the issue currently closed?"
  reopen:
    kind: process
    label: "cli-std issues a reopen request (state: open)\nbefore posting the comment"
  assemble_body:
    kind: process
    label: "cli-std assembles the follow-up comment body:\nfollowup_comment_body() = operator message (or the\nstandard verification-failed note) + render_diagnostics()\n(tool version/target/git sha/build time/os-arch)"
  dry_run_check:
    kind: decision
    label: "Is --dry-run set?"
  print_preview:
    kind: process
    label: "Print target issue number, resolved state (open),\nand the assembled diagnostics comment body;\nno GitHub API mutation is issued"
  post_comment:
    kind: process
    label: "POST the reopen (if needed) and the comment payload\nto the GitHub issues API via GITHUB_TOKEN"
  done_dry:
    kind: terminal
    label: "jet issue comment 123 --dry-run prints the target issue,\nstate open, and diagnostics comment; no network mutation"
  done_live:
    kind: terminal
    label: "Issue is open (reopened if it was closed) and carries\nthe new diagnostics-rich follow-up comment"
edges:
  - from: parse_args
    to: build_options
    label: "args parsed"
  - from: build_options
    to: dispatch_shared
    label: "options built"
  - from: dispatch_shared
    to: fetch_state
    label: "delegate to shared cli-std logic"
  - from: fetch_state
    to: is_closed
    label: "state fetched"
  - from: is_closed
    to: reopen
    label: "yes"
  - from: is_closed
    to: assemble_body
    label: "no"
  - from: reopen
    to: assemble_body
    label: "reopen queued"
  - from: assemble_body
    to: dry_run_check
    label: "comment body assembled"
  - from: dry_run_check
    to: print_preview
    label: "yes"
  - from: dry_run_check
    to: post_comment
    label: "no"
  - from: print_preview
    to: done_dry
    label: "preview printed, no mutation"
  - from: post_comment
    to: done_live
    label: "reopen + comment applied"
---
flowchart TD
    parse_args([jet issue comment number message --dry-run]) --> build_options[Build CommentOptions from ArgMatches]
    build_options --> dispatch_shared[run_issue dispatches to cli_std::issue::comment]
    dispatch_shared --> fetch_state[cli-std fetches current issue state]
    fetch_state --> is_closed{Is the issue currently closed?}
    is_closed -- yes --> reopen[Queue reopen request state: open]
    is_closed -- no --> assemble_body[Assemble diagnostics-rich follow-up comment body]
    reopen --> assemble_body
    assemble_body --> dry_run_check{Is --dry-run set?}
    dry_run_check -- yes --> print_preview[Print target issue, state open, diagnostics comment; no mutation]
    dry_run_check -- no --> post_comment[POST reopen if needed + comment payload via GITHUB_TOKEN]
    print_preview --> done_dry([dry-run prints issue+state+comment, no network mutation])
    post_comment --> done_live([Issue open and carries new follow-up comment])
```
