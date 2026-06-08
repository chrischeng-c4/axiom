---
number: 906
title: "chore(jet): codesign release binary — macOS kills unsigned cp'd binary"
state: open
labels: [bug, crate:jet]
group: "jet-infra-codesign"
---

# #906 — chore(jet): codesign release binary — macOS kills unsigned cp'd binary

## Problem

After `cargo build --release` + `cp target/release/cclab ~/.cargo/bin/cclab`, macOS SIGKILL (exit 137) the binary immediately on launch. The `cp` invalidates the ad-hoc code signature.

## Workaround

```bash
codesign -s - -f ~/.cargo/bin/cclab
```

## Fix Options

1. **Build script**: Add `codesign -s -` step to `/cclab:build-debug` and release skills
2. **Use `cargo install`** instead of `cp` (cargo handles signing)
3. **Add post-build hook** in `.cargo/config.toml`

## References
- macOS code signing requirement for arm64 binaries
- Affects all users who `cp` the binary instead of `cargo install`
