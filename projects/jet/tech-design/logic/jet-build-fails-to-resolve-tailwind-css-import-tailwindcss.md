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
