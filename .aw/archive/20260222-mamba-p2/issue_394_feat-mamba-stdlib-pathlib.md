---
number: 394
title: "feat(mamba): stdlib pathlib"
state: open
labels: [enhancement, P2, crate:mamba]
---

# #394 — feat(mamba): stdlib pathlib

## Summary
Implement `pathlib` standard library module for object-oriented filesystem paths.

## Required
- `Path(str)` constructor
- Properties: `.name`, `.stem`, `.suffix`, `.parent`, `.parts`
- Methods: `.exists()`, `.is_file()`, `.is_dir()`
- `.read_text()`, `.write_text()`, `.read_bytes()`, `.write_bytes()`
- `.mkdir(parents=False, exist_ok=False)`, `.rmdir()`
- `.rename(target)`, `.unlink()`
- `.glob(pattern)`, `.rglob(pattern)`
- `.iterdir()` — iterate directory contents
- `.resolve()`, `.absolute()`
- `.joinpath()` / `/` operator: `Path('a') / 'b'`

## Implementation Notes
- Use Rust `std::path::PathBuf` as backend
- Path objects as MbObject with ObjData::Path variant
