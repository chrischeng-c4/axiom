---
number: 397
title: "feat(mamba): stdlib subprocess"
state: open
labels: [enhancement, crate:mamba, P3]
---

# #397 — feat(mamba): stdlib subprocess

## Summary
Implement `subprocess` module for running external commands.

## Required
- `subprocess.run(args, capture_output=False, text=False, check=False)` → CompletedProcess
- `subprocess.Popen(args, stdin, stdout, stderr)` — low-level process management
- CompletedProcess: `.returncode`, `.stdout`, `.stderr`
- Pipe constants: `subprocess.PIPE`, `subprocess.DEVNULL`
- `subprocess.check_output(args)`, `subprocess.check_call(args)`

## Implementation Notes
- Use Rust `std::process::Command` as backend
