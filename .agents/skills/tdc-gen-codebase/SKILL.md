---
name: tdc:gen-codebase
description: Run codegen on the active tdc tech-design — emits CODEGEN-BEGIN/END blocks for spec-driven regions and HANDWRITE-BEGIN/END markers wherever a generator gap exists.
user-invocable: true
---

# /tdc:gen-codebase

Runs the codegen pipeline against the active `.tdc/state.toml` slug + td. Reads the spec's 19 sections, dispatches per-section generators, and writes generated code into the target codebase with `CODEGEN-BEGIN`/`CODEGEN-END` blocks. Wherever a generator does not yet exist (or punts on a region), it emits a `HANDWRITE-BEGIN reason: ...` / `HANDWRITE-END` marker for the handwrite subagent to fill later.

## Usage

```
/tdc:gen-codebase
```

Reads slug + td from `.tdc/state.toml`. No arguments — operates on the active session.

## Flow

1. Run `tdc gen-codebase`.
2. CLI loads `tech_designs/<td>.md`, validates it has all required sections.
3. For each section that has a registered generator: emit code into the target paths in the working tree (paths come from the spec's `changes` section).
4. For each section without a generator OR for regions the generator can't fill: emit `HANDWRITE-BEGIN reason: <gap>` / `HANDWRITE-END` markers.
5. CLI prints a summary: how many CODEGEN blocks written, how many HANDWRITE markers left.
6. Stage the new/changed files, commit with `tdc(<slug>): gen-codebase <td>`.

## What you (mainthread) actually do

- Run `tdc gen-codebase`. Read its summary.
- If summary lists HANDWRITE markers > 0: tell user to run `/tdc:handwrite` next.
- If summary lists 0 HANDWRITE markers: codegen is complete; tell user.

## Failure modes

- Spec missing required sections → CLI errors, surface to user.
- Generator crashes on a section → CLI errors with section name, surface to user. Do not retry.
- Generated code conflicts with existing CODEGEN block → CLI errors; user must reconcile manually before retrying.
