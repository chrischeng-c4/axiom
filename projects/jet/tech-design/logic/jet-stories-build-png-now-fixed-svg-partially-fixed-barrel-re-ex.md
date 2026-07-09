---
id: jet-stories-build-bare-specifier-style-assets
summary: "jet stories build: a bare-specifier `.css`/`.scss`/`.sass` import (e.g. `import 'pkg/style.css'`, a package.json `exports` subpath resolving straight to a stylesheet) is not recognized as a style asset — it falls through to the JS-dependency path, producing a dangling `deps/pkg/style.css.js` reference to a file that is never written, with no compiled CSS and no `<link>` tag. This closes the still-reproducible slice of the CSS-orphaning family filed as WI #938 and re-tested as WI #1237; the relative-import CSS scenario and the SVGR barrel re-export scenario both cited in #1237 are already fixed on current `app/jet` (verified empirically and via the existing `build_compiles_scss_side_effect_imports_to_static_css` and `build_rewrites_svg_reactcomponent_barrel_reexports` tests, both passing)."
capability_refs:
  - id: "component-workbench"
    role: primary
    gap: "stories-static-export"
    claim: "stories-static-export"
    coverage: partial
    rationale: "Pins WI #1237/#938 regression coverage for the bare-specifier style-import variant of the stories-build asset-orphaning family inside the Stories Static Export work root (jet stories build's compiled-CSS/SVG/PNG asset wiring)."
fill_sections: [logic, unit-test, changes]
---

# jet stories build: bare-specifier CSS/SCSS/Sass style imports are not recognized as style assets

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: jet-stories-build-bare-specifier-style-assets
entry: import_specifier
nodes:
  import_specifier:      { kind: start,    label: "extract_all_import_specifiers(code)\nspecifier found while walking a\nmodule/dep in emit_module_graph" }
  is_relative_spec:      { kind: decision, label: "spec starts with '.'\n(relative import)?" }
  relative_style_check:  { kind: decision, label: "is_style_path(target_file)?\n(css/scss/sass extension,\nchecked BEFORE module-vs-dep split)" }
  relative_style_ok:     { kind: terminal, label: "ALREADY CORRECT: styles.push(\nstyle_asset_for_file(root, target_file));\nstyle_specs records spec for later\nremove_static_import_for_spec;\nCssPipeline compiles via emit_style_asset,\nwriting modules-or-deps/*.css;\ninject_static_stylesheet_links links it\ninto every preview/<id>.html" }
  relative_non_style:    { kind: process,  label: "existing relative asset/module\nhandling unchanged (is_raw_asset_path,\nEmitItem::Module/Dep)" }
  bare_spec:              { kind: process,  label: "resolve_bare_specifier(root,\nimporter_file, spec) via oxc resolver\n(#923's package.json exports fallback\nincluded) -> dep_file" }
  bare_asset_check:       { kind: decision, label: "is_raw_asset_path(dep_file)?\n(svg/png/jpg/jpeg/gif/webp/avif only\n- build.rs local list, NOT css/scss/sass)" }
  bare_style_check_gap:   { kind: decision, label: "MISSING: no is_style_path(dep_file)\ncheck exists on this branch at all -\nthe relative branch's style detection\nis never mirrored here" }
  bare_style_ok:          { kind: terminal, label: "FIX: styles.push(\nstyle_asset_for_file(root, dep_file));\nstyle_specs records spec; same\nremove_static_import_for_spec +\nemit_style_asset + link-injection\npath the relative branch already uses" }
  bare_fallthrough_dep:   { kind: terminal, label: "BUG (current): dep_file falls through\nto EmitItem::Dep(dep_file); emitted_path\nalways appends .js (to_js_path) ->\ndeps/<pkg>/style.css.js referenced in\nrewritten JS, but nothing ever writes\nthat file (emit_item only compiles JS\nsource text, not CSS) - the import\nsurvives as a dangling reference to a\nnonexistent module, no diagnostic raised,\nno CSS compiled, no <link> tag added" }
edges:
  - { from: import_specifier,      to: is_relative_spec }
  - { from: is_relative_spec,      to: relative_style_check, label: "yes" }
  - { from: is_relative_spec,      to: bare_spec,             label: "no" }
  - { from: relative_style_check,  to: relative_style_ok,     label: "yes" }
  - { from: relative_style_check,  to: relative_non_style,    label: "no" }
  - { from: bare_spec,             to: bare_asset_check }
  - { from: bare_asset_check,      to: bare_style_check_gap,  label: "no (not svg/png/etc)" }
  - { from: bare_asset_check,      to: relative_non_style,    label: "yes (raw asset,\nalready handled)" }
  - { from: bare_style_check_gap,  to: bare_fallthrough_dep,  label: "current: falls through\n(no style check exists)" }
  - { from: bare_style_check_gap,  to: bare_style_ok,         label: "fixed: add\nis_style_path(dep_file) check" }
---
flowchart TD
    import_specifier(["extract_all_import_specifiers(code)\nspecifier found while walking a\nmodule/dep in emit_module_graph"]) --> is_relative_spec{"spec starts with '.'\n(relative import)?"}
    is_relative_spec -->|yes| relative_style_check{"is_style_path(target_file)?\n(css/scss/sass extension,\nchecked BEFORE module-vs-dep split)"}
    is_relative_spec -->|no| bare_spec["resolve_bare_specifier(root,\nimporter_file, spec) via oxc resolver\n(#923's package.json exports fallback\nincluded) -> dep_file"]
    relative_style_check -->|yes| relative_style_ok(["ALREADY CORRECT: styles.push(\nstyle_asset_for_file(root, target_file));\nstyle_specs records spec for later\nremove_static_import_for_spec;\nCssPipeline compiles via emit_style_asset,\nwriting modules-or-deps/*.css;\ninject_static_stylesheet_links links it\ninto every preview/<id>.html"])
    relative_style_check -->|no| relative_non_style["existing relative asset/module\nhandling unchanged (is_raw_asset_path,\nEmitItem::Module/Dep)"]
    bare_spec --> bare_asset_check{"is_raw_asset_path(dep_file)?\n(svg/png/jpg/jpeg/gif/webp/avif only\n- build.rs local list, NOT css/scss/sass)"}
    bare_asset_check -->|no not svg/png/etc| bare_style_check_gap{"MISSING: no is_style_path(dep_file)\ncheck exists on this branch at all -\nthe relative branch's style detection\nis never mirrored here"}
    bare_asset_check -->|yes raw asset already handled| relative_non_style
    bare_style_check_gap -->|current falls through no style check exists| bare_fallthrough_dep(["BUG (current): dep_file falls through\nto EmitItem::Dep(dep_file); emitted_path\nalways appends .js (to_js_path) ->\ndeps/<pkg>/style.css.js referenced in\nrewritten JS, but nothing ever writes\nthat file (emit_item only compiles JS\nsource text, not CSS) - the import\nsurvives as a dangling reference to a\nnonexistent module, no diagnostic raised,\nno CSS compiled, no <link> tag added"])
    bare_style_check_gap -->|fixed add is_style_path dep_file check| bare_style_ok(["FIX: styles.push(\nstyle_asset_for_file(root, dep_file));\nstyle_specs records spec; same\nremove_static_import_for_spec +\nemit_style_asset + link-injection\npath the relative branch already uses"])
```

Scope for WI #1237 (`projects/jet/src/stories/build.rs`, `rewrite_imports`): the relative-import branch already calls `is_style_path(&target_file)` before deciding module-vs-dep, so a relative `.scss`/`.css`/`.sass` import — the literal scenario #1237 describes (`sp-box.tsx` -> `import './sp-box.scss'`) — is compiled through `CssPipeline`, its static import statement is stripped, and the compiled CSS is linked into every static preview via `inject_static_stylesheet_links`. This is confirmed working on current `app/jet` HEAD by the pre-existing `build_compiles_scss_side_effect_imports_to_static_css` test (passing) and by a from-scratch minimal repro built for this investigation (a project-local `.scss` side-effect import, a full-package-subpath `.svg` import, an SVGR barrel `export { ReactComponent as X } from './icon.svg'` re-export, and a bare `.png` import all resolved and linked/compiled correctly). The SVGR barrel re-export handling (`is_svg_component_reexport`/`rewrite_svg_component_reexport_for_spec`) also landed after #1237 was filed (in the jet@0.4.16 release) and is confirmed working by the pre-existing `build_rewrites_svg_reactcomponent_barrel_reexports` test.

The bare-specifier branch, however, only checks `is_raw_asset_path(&dep_file)` (svg/png/jpg/jpeg/gif/webp/avif) before falling through to `EmitItem::Dep(dep_file)` — there is no equivalent `is_style_path` check on this branch at all. A bare specifier that resolves (via `super::deps::resolve_bare_specifier`, including the `#923` package.json `exports`-subpath fallback) straight to a `.css`/`.scss`/`.sass` file is therefore treated as a JS dependency: `EmitItem::Dep::emitted_path` unconditionally appends `.js` (`to_js_path`), so the rewritten import becomes a reference to `deps/<pkg>/<name>.css.js` — a file `emit_item` never writes (it only transforms JS source text). The result: the import specifier survives as a dangling reference in the emitted JS, no CSS is compiled, no `<link>` tag is added (since `emitted_styles` is only ever populated from `emit.styles`, which this path never populates), and no diagnostic is raised — the failure is silent. This was reproduced from scratch for this investigation (`import '@tw-tech/ds2/style.css'` resolving via a package.json `exports` subpath) and confirmed against current `app/jet` HEAD: the emitted `deps/@tw-tech/ds2` directory exists but is empty, and the importing module's rewritten JS references `../../../deps/@tw-tech/ds2/style.css.js`, a file that is never written.

The fix mirrors the relative branch exactly: add an `is_style_path(&dep_file)` check to the bare-specifier branch of `rewrite_imports`, ahead of (or alongside) the existing `is_raw_asset_path` check, that pushes a `StyleAsset` via the existing `style_asset_for_file` helper (which already branches on `path_has_node_modules` to compute the correct `deps/...css` emitted path) and records the spec in `style_specs` for `remove_static_import_for_spec`, then `continue`s the same way the relative branch does. No new CSS-compilation, link-injection, or path-computation logic is needed — `emit_style_asset`, `inject_static_stylesheet_links`, and `style_asset_for_file` already handle both module- and dep-rooted style assets correctly; only the missing detection branch needs wiring.

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: jet-stories-build-bare-specifier-style-assets-verification
requirements:
  bare_specifier_css_import_compiles_and_links:
    id: R1
    text: "A bare-specifier `.css` import that resolves via a package's `package.json` exports subpath (e.g. `import '@scope/pkg/style.css'`) emits a real compiled CSS asset at `deps/<pkg>/<name>.css`, strips the import statement from the emitted JS module (no dangling `import` remains), and links the compiled CSS into every static preview via `<link rel=\"stylesheet\">` - matching the existing relative-import style-asset behavior instead of falling through to a broken `deps/<pkg>/<name>.css.js` JS-dependency reference."
    kind: functional
    risk: high
    verify: cargo test -p jet --test stories_build build_compiles_bare_specifier_css_import_to_static_css
  bare_specifier_raw_asset_import_still_resolves_as_dep:
    id: R3
    text: "Negative control: a bare-specifier import resolving to a non-style, non-raw-asset file (a plain `.js`/`.mjs` dependency module) is unaffected by the new `is_style_path` check on the bare-specifier branch and still resolves as `EmitItem::Dep` exactly as before, proving the fix is scoped to style extensions and does not change bare-specifier JS dependency resolution."
    kind: regression
    risk: medium
    verify: cargo test -p jet --test stories_build build_emits_svg_and_png_assets_as_url_strings
  bare_specifier_scss_import_compiles_via_css_pipeline:
    id: R2
    text: "A bare-specifier `.scss` import resolved through the same package.json exports subpath path is compiled through the real `CssPipeline` (nesting and variables flattened into valid CSS), not copied verbatim or left as raw Sass, confirming the fix reuses the identical style-asset detection and compilation path the relative-import branch already uses rather than adding a second, divergent implementation."
    kind: functional
    risk: medium
    verify: cargo test -p jet --test stories_build build_compiles_bare_specifier_scss_import_via_css_pipeline
---
flowchart TD
    r1[R1 bare specifier css import compiles and links] --> cargo_test_p_jet_test_stories_build_build_compiles_bare_specifier_css_import_to_static_css[cargo test -p jet --test stories_build build_compiles_bare_specifier_css_import_to_static_css]
    r2[R2 bare specifier scss import compiles via css pipeline] --> cargo_test_p_jet_test_stories_build_build_compiles_bare_specifier_scss_import_via_css_pipeline[cargo test -p jet --test stories_build build_compiles_bare_specifier_scss_import_via_css_pipeline]
    r3[R3 bare specifier raw asset import still resolves as dep] --> cargo_test_p_jet_test_stories_build_build_emits_svg_and_png_assets_as_url_strings[cargo test -p jet --test stories_build build_emits_svg_and_png_assets_as_url_strings]
```
