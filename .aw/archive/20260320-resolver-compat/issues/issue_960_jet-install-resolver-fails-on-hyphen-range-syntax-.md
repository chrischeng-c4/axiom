---
number: 960
title: "jet-install: resolver fails on hyphen range syntax (2 - 4)"
state: open
labels: [bug, crate:jet]
group: "resolver-compat"
---

# #960 — jet-install: resolver fails on hyphen range syntax (2 - 4)

## Problem

\`cclab jet install\` fails on tech-platform with:
\`\`\`
Error: Failed to parse version range '2 - 4'
\`\`\`

## Root Cause

npm semver supports hyphen ranges: \`2 - 4\` means \`>=2.0.0 <5.0.0\`. Jet's resolver doesn't parse this syntax.

Common hyphen range patterns:
- \`2 - 4\` → \`>=2.0.0 <5.0.0\`
- \`1.0 - 2.0\` → \`>=1.0.0 <2.1.0\`
- \`1.0.0 - 2.0.0\` → \`>=1.0.0 <=2.0.0\`

## Fix

In the semver range parser, detect \`X - Y\` pattern before normal parsing.

## References
- \`crates/cclab-jet/src/pkg_manager/resolver.rs\`
- #883, #957
