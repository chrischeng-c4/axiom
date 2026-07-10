---
id: jet-build-fails-to-resolve-tailwind-css-import-tailwindcss
summary: "placeholder"
fill_sections: [logic, unit-test, changes]
---

# jet build fails to resolve Tailwind CSS @import 'tailwindcss'

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: jet-css-bare-specifier-package-entry-resolution-flow
entry: encounter_bare_specifier_import
nodes:
  encounter_bare_specifier_import:
    kind: start
    label: "CSS import resolver (import_resolver.rs) hits a\nbare-specifier @import, e.g. @import \"tailwindcss\";"
  walk_node_modules:
    kind: process
    label: "resolve_import_path walks base_dir upward,\nreturning the first existing\nnode_modules/<pkg> candidate path"
  is_dir_check:
    kind: decision
    label: "Does the resolved node_modules/<pkg>\ncandidate path exist as a directory\n(not a file)?"
  read_file_directly:
    kind: process
    label: "Existing behavior: fs::read_to_string(candidate)\ninlines the file's CSS content unchanged"
  open_package_json:
    kind: process
    label: "NEW: build candidate.join(\"package.json\");\nif it does not exist, fall through to the\nexisting unresolved-path error path\n(no silent success)"
  try_exports_field:
    kind: process
    label: "NEW: call the existing resolver::package::\nresolve_exports(package_json, Some(\".\"),\n[\"style\", \"default\"]) helper (already used by\nresolver/mod.rs for JS resolution) to look up\nthe root export's CSS entry via a style-first\ncondition list"
  exports_found:
    kind: decision
    label: "Did resolve_exports return Some(path)?"
  use_exports_path:
    kind: process
    label: "candidate = package_dir.join(exported_path)\n(strip a leading \"./\")"
  try_style_field:
    kind: process
    label: "NEW: no exports match \u2014 read the package.json\ntop-level \"style\" field (the long-standing\nnpm CSS-entry convention used by e.g. bootstrap)\nvia a new PackageJson.style: Option<String> field"
  style_found:
    kind: decision
    label: "Is package.json's top-level \"style\" field present?"
  use_style_path:
    kind: process
    label: "candidate = package_dir.join(style_path)"
  try_main_field:
    kind: process
    label: "NEW: fall back to resolver::package::get_package_main\n(module || main || \"index.js\"), reusing the same\nhelper resolver/mod.rs already calls for JS packages"
  use_main_path:
    kind: process
    label: "candidate = package_dir.join(main_path)"
  final_candidate_exists:
    kind: decision
    label: "Does the resulting candidate file exist\non disk?"
  inline_resolved_entry:
    kind: process
    label: "resolve_file(candidate, visited) reads and\nrecursively inlines the package's real CSS\nentry, exactly like any other resolved import"
  bail_with_clear_error:
    kind: process
    label: "Bail with an error naming the package.json\nfallback path attempted (never re-attempt\nfs::read_to_string on a directory \u2014 the\noriginal \"Is a directory (os error 21)\" defect\nmust not resurface here)"
  done_inlined:
    kind: terminal
    label: "@import \"tailwindcss\"; (or any bare specifier\nwhose node_modules/<pkg> path is a directory)\ninlines the package's real CSS entry file\ninstead of crashing"
  done_error:
    kind: terminal
    label: "Directory-only bare specifier with no resolvable\npackage.json entry surfaces a typed, descriptive\nerror \u2014 never an unhandled 'Is a directory' I/O error"
edges:
  - { from: encounter_bare_specifier_import, to: walk_node_modules }
  - { from: walk_node_modules, to: is_dir_check }
  - { from: is_dir_check, to: read_file_directly, label: "file (existing packages\nwith a flat entry, e.g.\nnormalize.css)" }
  - { from: is_dir_check, to: open_package_json, label: "directory (e.g.\nnode_modules/tailwindcss)" }
  - { from: read_file_directly, to: done_inlined }
  - { from: open_package_json, to: try_exports_field, label: "package.json exists" }
  - { from: open_package_json, to: bail_with_clear_error, label: "no package.json" }
  - { from: try_exports_field, to: exports_found }
  - { from: exports_found, to: use_exports_path, label: "yes" }
  - { from: exports_found, to: try_style_field, label: "no" }
  - { from: use_exports_path, to: final_candidate_exists }
  - { from: try_style_field, to: style_found }
  - { from: style_found, to: use_style_path, label: "yes" }
  - { from: style_found, to: try_main_field, label: "no" }
  - { from: use_style_path, to: final_candidate_exists }
  - { from: try_main_field, to: use_main_path }
  - { from: use_main_path, to: final_candidate_exists }
  - { from: final_candidate_exists, to: inline_resolved_entry, label: "yes" }
  - { from: final_candidate_exists, to: bail_with_clear_error, label: "no" }
  - { from: inline_resolved_entry, to: done_inlined }
  - { from: bail_with_clear_error, to: done_error }
---
flowchart TD
    encounter_bare_specifier_import([CSS import resolver hits a bare-specifier @import, e.g. @import "tailwindcss";]) --> walk_node_modules[resolve_import_path walks base_dir upward for the first existing node_modules/pkg candidate]
    walk_node_modules --> is_dir_check{Does the resolved node_modules/pkg candidate exist as a directory, not a file?}
    is_dir_check -->|file| read_file_directly[Existing behavior: read the file directly and inline it]
    is_dir_check -->|directory| open_package_json[NEW: build candidate/package.json; missing file falls through to the existing unresolved-path error path]
    read_file_directly --> done_inlined([Bare specifier inlines correctly])
    open_package_json -->|exists| try_exports_field[NEW: call existing resolve_exports with subpath '.' and conditions style,default]
    open_package_json -->|missing| bail_with_clear_error[Bail with a clear error naming the attempted package.json fallback, never re-read the directory as a file]
    try_exports_field --> exports_found{Did resolve_exports return Some path?}
    exports_found -->|yes| use_exports_path[candidate = package_dir join exported path]
    exports_found -->|no| try_style_field[NEW: read package.json top-level style field via new PackageJson.style]
    use_exports_path --> final_candidate_exists
    try_style_field --> style_found{Is top-level style field present?}
    style_found -->|yes| use_style_path[candidate = package_dir join style path]
    style_found -->|no| try_main_field[NEW: fall back to existing get_package_main module main or index.js]
    use_style_path --> final_candidate_exists{Does the resulting candidate file exist on disk?}
    try_main_field --> use_main_path[candidate = package_dir join main path]
    use_main_path --> final_candidate_exists
    final_candidate_exists -->|yes| inline_resolved_entry[resolve_file recursively inlines the package real CSS entry]
    final_candidate_exists -->|no| bail_with_clear_error
    inline_resolved_entry --> done_inlined
    bail_with_clear_error --> done_error([Directory-only bare specifier with no resolvable entry surfaces a typed descriptive error, never an unhandled I/O 'Is a directory' error])
```

Root cause: `projects/jet/src/css/import_resolver.rs::resolve_import_path` builds the `node_modules/<pkg>` candidate for a bare-specifier `@import` and returns it as soon as `candidate.exists()` is true, without ever checking whether the candidate is a file or a directory. `resolve_file` then unconditionally calls `std::fs::read_to_string(path)` on that candidate. For a package like `tailwindcss` whose npm entry point is not a single top-level CSS file, `node_modules/tailwindcss` is a directory, so `read_to_string` fails with `Is a directory (os error 21)` -- there is no fallback to the package's own `package.json` `exports`/`style`/`main` map, unlike `resolver/mod.rs::resolve_package_dir`, which already performs this fallback for JS bare specifiers via `resolver::package::resolve_exports` and `get_package_main`.

Fix (R1): add a directory-fallback branch to the CSS import resolver, reusing `resolver::package`'s existing helpers rather than re-implementing package.json exports resolution (WI Scope): when the `node_modules/<pkg>` candidate is a directory, read `candidate/package.json` and resolve the real CSS entry in priority order -- `resolve_exports(package_json, Some("."), ["style", "default"])` (mirrors the JS resolver's own call shape), then a new top-level `PackageJson.style` field (added to `resolver/package.rs`, the long-standing npm CSS-entry convention), then `get_package_main`. The first entry that resolves to an existing file on disk is passed to the existing `resolve_file` recursion unchanged; if none resolve, the resolver bails with a descriptive error naming the package.json fallback attempted, so the original `Is a directory` defect can never resurface as either a silent crash or a silent no-op.
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: jet-css-bare-specifier-package-entry-resolution-verification
requirements:
  bare_specifier_directory_falls_back_to_main_field_when_style_absent:
    id: R3
    text: "WI #1375 R1 priority order: when a directory-only bare specifier's package.json has neither an 'exports' match nor a top-level 'style' field, the resolver falls back to the package.json 'main' field to find the real CSS entry file."
    kind: functional
    risk: medium
    verify: cargo test -p jet --lib css::import_resolver::tests::bare_specifier_directory_falls_back_to_main_field_when_style_absent
  bare_specifier_directory_falls_back_to_style_field_when_exports_absent:
    id: R2
    text: "WI #1375 R1 priority order: when a directory-only bare specifier's package.json has no 'exports' match, the resolver falls back to the package.json top-level 'style' field to find the real CSS entry file."
    kind: functional
    risk: medium
    verify: cargo test -p jet --lib css::import_resolver::tests::bare_specifier_directory_falls_back_to_style_field_when_exports_absent
  bare_specifier_directory_resolves_via_package_json_exports:
    id: R1
    text: "WI #1375 AC1/AC2: a bare-specifier @import (e.g. @import \"tailwindcss\";) whose node_modules/<pkg> path is a directory, with a package.json 'exports' map whose '.' entry resolves (via the existing resolve_exports helper with a style-first condition list) to a real CSS file, is inlined by resolve_imports/resolve_source instead of raising \"Is a directory (os error 21)\". Uses a minimal fixture package under a tempdir's node_modules with a package.json pointing at a real CSS file; does not require an actual tailwindcss install."
    kind: functional
    risk: high
    verify: cargo test -p jet --lib css::import_resolver::tests::bare_specifier_directory_resolves_via_package_json_exports
  bare_specifier_directory_without_resolvable_entry_surfaces_clear_error:
    id: R4
    text: "Negative path: a directory-only bare specifier whose package.json has no exports/style/main match (or no package.json at all) surfaces a typed, descriptive error naming the attempted package.json fallback -- never the raw 'Is a directory (os error 21)' I/O error and never a silent false success."
    kind: regression
    risk: medium
    verify: cargo test -p jet --lib css::import_resolver::tests::bare_specifier_directory_without_resolvable_entry_surfaces_clear_error
  existing_css_import_resolver_suite_stays_green:
    id: R5
    text: "WI #1375 AC3: the existing css::import_resolver test suite (relative resolution, circular detection, remote-URL passthrough, three-level import chain, canonicalize-error surfacing) remains green after the package.json fallback change."
    kind: regression
    risk: low
    verify: cargo test -p jet --lib css::
---
flowchart TD
    r1[R1 bare specifier directory resolves via package json exports] --> cargo_test_p_jet_lib_css_import_resolver_tests_bare_specifier_directory_resolves_via_package_json_exports[cargo test -p jet --lib css::import_resolver::tests::bare_specifier_directory_resolves_via_package_json_exports]
    r2[R2 bare specifier directory falls back to style field when exports absent] --> cargo_test_p_jet_lib_css_import_resolver_tests_bare_specifier_directory_falls_back_to_style_field_when_exports_absent[cargo test -p jet --lib css::import_resolver::tests::bare_specifier_directory_falls_back_to_style_field_when_exports_absent]
    r3[R3 bare specifier directory falls back to main field when style absent] --> cargo_test_p_jet_lib_css_import_resolver_tests_bare_specifier_directory_falls_back_to_main_field_when_style_absent[cargo test -p jet --lib css::import_resolver::tests::bare_specifier_directory_falls_back_to_main_field_when_style_absent]
    r4[R4 bare specifier directory without resolvable entry surfaces clear error] --> cargo_test_p_jet_lib_css_import_resolver_tests_bare_specifier_directory_without_resolvable_entry_surfaces_clear_error[cargo test -p jet --lib css::import_resolver::tests::bare_specifier_directory_without_resolvable_entry_surfaces_clear_error]
    r5[R5 existing css import resolver suite stays green] --> cargo_test_p_jet_lib_css[cargo test -p jet --lib css::]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/jet/src/resolver/package.rs
    action: update
    section: logic
    impl_mode: hand-written
    reason: "Add a `style: Option<String>` field to `PackageJson` (alongside the existing `main`/`module`/`exports` fields) so the CSS import resolver can reuse this same struct/`read_package_json` to read the long-standing npm top-level `style` CSS-entry convention, instead of re-implementing package.json parsing in css/import_resolver.rs."
  - path: projects/jet/src/css/import_resolver.rs
    action: update
    section: logic
    impl_mode: hand-written
    reason: "Fix WI #1375 R1: when a resolved node_modules/<pkg> bare-specifier candidate path (built by resolve_import_path) is a directory instead of a file, add a new fallback helper (e.g. resolve_package_css_entry) that: (1) builds candidate.join(\"package.json\"); if absent, keeps the existing unresolved-path behavior; (2) calls the existing resolver::package::resolve_exports(package_json, Some(\".\"), [\"style\", \"default\"]) helper (already used by resolver/mod.rs for JS resolution) to look up a CSS entry via the root export; (3) if no exports match, falls back to the new PackageJson.style top-level field; (4) if style is absent, falls back to resolver::package::get_package_main (module || main || \"index.js\"). The final resolved file path is joined with package_dir and returned for resolve_file to read/inline exactly as any other resolved import; if none of the three resolve to an existing file, bail with a descriptive error naming the package.json fallback attempted -- never re-attempt fs::read_to_string on the directory itself (the original \"Is a directory (os error 21)\" defect must not resurface). Reuses resolver::package's existing helpers rather than re-implementing package.json exports resolution, per WI Scope."
  - path: projects/jet/src/css/import_resolver.rs
    action: update
    section: unit-test
    impl_mode: hand-written
    reason: "Add the R1-R5 regression tests specified in the unit-test section to the existing mod tests block, each building a minimal fixture package under a tempdir's node_modules/<pkg>/ (a package.json plus a real target .css file) to exercise the directory-fallback path without requiring an actual tailwindcss install: R1 exercises an 'exports' map with a '.' entry; R2 exercises a top-level 'style' field with no matching 'exports' entry; R3 exercises a 'main' field with neither 'exports' nor 'style' present; R4 exercises a directory-only bare specifier with no resolvable package.json entry, asserting a descriptive typed error (not the raw 'Is a directory' OS error and not a silent success); R5 re-runs the pre-existing css::import_resolver test suite (t3/t4/resolve_source_inlines_import/remote_imports_preserved/resolve_file_surfaces_canonicalize_error_for_missing_path/three_level_import_chain_merged) unchanged, proving no regression."
  - path: projects/jet/README.md
    action: update
    section: config
    impl_mode: hand-written
    reason: "Add a new 'Fix CSS Bare-Specifier Import Resolution For Package Directories' work-root row (kind: change, WI #1375, impl: planned, verification: none, maturity: smoke) to the Bundler And Production Build capability's work-root table, citing the future `cargo test -p jet --lib css::import_resolver` regression coverage and this TD's path, closing the capability gap described in WI #1375's Capability Alignment section (the CSS import resolver could not resolve a bare package-name @import when node_modules/<pkg> is a directory)."
```
