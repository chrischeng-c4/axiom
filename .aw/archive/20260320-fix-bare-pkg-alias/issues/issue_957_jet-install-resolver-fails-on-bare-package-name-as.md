---
number: 957
title: "jet-install: resolver fails on bare package name as version spec (@storybook/expect: storybook-jest)"
state: open
labels: [bug, crate:jet]
group: "jet-resolver-bare-pkg-alias"
---

# #957 — jet-install: resolver fails on bare package name as version spec (@storybook/expect: storybook-jest)

## Problem

\`cclab jet install\` fails on tech-platform frontend with:
\`\`\`
Error: Failed to parse version range 'storybook-jest'
\`\`\`

## Root Cause

package-lock.json contains: \`"@storybook/expect": "storybook-jest"\`

This is a valid npm pattern — a bare package name as a version spec means "install this package instead" (implicit \`npm:\` alias). Jet's resolver only handles explicit \`npm:pkg@version\` aliases, not bare package names without a version range.

## Expected Behavior

When version spec doesn't parse as semver, check if it's a valid package name and treat as \`npm:{name}@latest\`.

## Reproduction

\`\`\`bash
cd ~/projects/tech-platform/main/frontend
cclab jet install  # Fails
\`\`\`

## References
- \`crates/cclab-jet/src/pkg_manager/resolver.rs\`
- #883 (resolver bugs)
