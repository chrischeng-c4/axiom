// HANDWRITE-BEGIN gap="missing-generator:unit-test:6522e798" tracker="pending-tracker" reason="Tests: importing an .svg yields a component rendering <svg> with forwarded props; named vs default export; plain asset-url import still works."
//! SVGR (import `.svg` as a React component) build tests.
//!
//! Covers the transform (`transform_svg_to_component`) and the bundler routing
//! decision (`should_route_svg_as_component`) that picks the component path vs.
//! the existing asset-URL path. The full end-to-end "build a fixture component
//! that imports an .svg-as-component" is deferred — see the
//! `// TODO(#203 follow-up)` at the end of this file.

use jet::asset::svgr::{transform_svg_to_component, SvgrExportType};
use jet::bundler::imports::{is_svg_specifier, should_route_svg_as_component, ImportKind};

// ─── (a) transform → React component module ──────────────────────────────────

#[test]
fn transform_small_svg_emits_named_react_component() {
    let svg = r#"<svg viewBox="0 0 24 24"><path d="M0 0h24v24H0z"/></svg>"#;
    let module = transform_svg_to_component(svg, SvgrExportType::Named).unwrap();

    // Imports React.
    assert!(
        module.contains("import * as React from \"react\""),
        "module must import React:\n{module}"
    );
    // Renders the root <svg> and its <path> child as JSX.
    assert!(module.contains("<svg"), "module:\n{module}");
    assert!(module.contains("<path"), "module:\n{module}");
    assert!(
        module.contains("viewBox=\"0 0 24 24\""),
        "viewBox attr should survive:\n{module}"
    );
    // Spreads incoming props and forwards ref onto the root <svg>.
    assert!(module.contains("{...props}"), "module:\n{module}");
    assert!(module.contains("ref={ref}"), "module:\n{module}");
    assert!(module.contains("React.forwardRef"), "module:\n{module}");
    // Named export (matches fe-shared `{ exportType: 'named' }`).
    assert!(
        module.contains("export { ReactComponent };"),
        "module:\n{module}"
    );
    assert!(
        !module.contains("export default"),
        "named-only should not emit a default export:\n{module}"
    );
}

#[test]
fn transform_default_export_emits_default_component() {
    let svg = r#"<svg><path d="M0 0"/></svg>"#;
    let module = transform_svg_to_component(svg, SvgrExportType::Default).unwrap();
    assert!(
        module.contains("export default ReactComponent;"),
        "module:\n{module}"
    );
    assert!(
        !module.contains("export { ReactComponent }"),
        "default-only should not emit the named export:\n{module}"
    );
}

#[test]
fn transform_both_export_emits_named_and_default() {
    let svg = r#"<svg><path d="M0 0"/></svg>"#;
    let module = transform_svg_to_component(svg, SvgrExportType::Both).unwrap();
    assert!(module.contains("export { ReactComponent };"), "module:\n{module}");
    assert!(
        module.contains("export default ReactComponent;"),
        "module:\n{module}"
    );
}

#[test]
fn default_export_type_matches_fe_shared() {
    // fe-shared uses `vite-plugin-svgr` with `{ exportType: 'named' }`.
    assert_eq!(SvgrExportType::default(), SvgrExportType::Named);
}

// ─── (b) attribute conversion ────────────────────────────────────────────────

#[test]
fn attribute_conversion_class_to_classname() {
    let svg = r#"<svg class="icon"><path class="shape" d="M0 0"/></svg>"#;
    let module = transform_svg_to_component(svg, SvgrExportType::Named).unwrap();
    assert!(module.contains("className=\"icon\""), "module:\n{module}");
    assert!(module.contains("className=\"shape\""), "module:\n{module}");
    assert!(
        !module.contains(" class="),
        "raw `class=` must not appear in JSX output:\n{module}"
    );
}

#[test]
fn attribute_conversion_kebab_to_camel() {
    let svg = r#"<svg><path stroke-width="2" clip-rule="evenodd" fill-opacity="0.5" d="M0 0"/></svg>"#;
    let module = transform_svg_to_component(svg, SvgrExportType::Named).unwrap();
    assert!(module.contains("strokeWidth=\"2\""), "module:\n{module}");
    assert!(module.contains("clipRule=\"evenodd\""), "module:\n{module}");
    assert!(module.contains("fillOpacity=\"0.5\""), "module:\n{module}");
}

#[test]
fn attribute_conversion_style_string_to_object() {
    let svg = r#"<svg style="fill:red;stroke-width:2px"><path d="M0 0"/></svg>"#;
    let module = transform_svg_to_component(svg, SvgrExportType::Named).unwrap();
    assert!(module.contains("style={{"), "style must be an object literal:\n{module}");
    assert!(module.contains("fill:\"red\""), "module:\n{module}");
    assert!(module.contains("strokeWidth:\"2px\""), "module:\n{module}");
}

#[test]
fn nested_elements_and_groups_are_rendered() {
    let svg =
        r#"<svg><g fill="none"><path d="M1 1"/><circle cx="1" cy="2" r="3"/></g></svg>"#;
    let module = transform_svg_to_component(svg, SvgrExportType::Named).unwrap();
    assert!(module.contains("<g"), "module:\n{module}");
    assert!(module.contains("</g>"), "module:\n{module}");
    assert!(module.contains("<circle"), "module:\n{module}");
    assert!(module.contains("<path"), "module:\n{module}");
}

#[test]
fn transform_rejects_non_svg_root() {
    let err = transform_svg_to_component("<div></div>", SvgrExportType::Named).unwrap_err();
    assert!(format!("{err}").contains("svg"), "err: {err}");
}

// ─── (c) bundler routing: .svg-as-component vs asset-URL ─────────────────────

#[test]
fn imports_routes_named_reactcomponent_svg_through_svgr() {
    // `import { ReactComponent as Icon } from './icon.svg'` (fe-shared shape)
    // routes through SVGR; the resulting module is a real React component.
    let specifier = "./icon.svg";
    assert!(is_svg_specifier(specifier));
    assert!(
        should_route_svg_as_component(
            specifier,
            &ImportKind::Named,
            /* svgr_enabled */ true,
            SvgrExportType::Named,
        ),
        "named ReactComponent .svg import must route through SVGR"
    );

    // And the transform that this routing selects produces a usable component.
    let svg = r#"<svg><path d="M0 0"/></svg>"#;
    let module = transform_svg_to_component(svg, SvgrExportType::Named).unwrap();
    assert!(module.contains("export { ReactComponent };"));
}

#[test]
fn imports_keeps_asset_url_for_default_import_under_named_config() {
    // Under `{ exportType: 'named' }`, a bare default import of an .svg stays
    // the existing asset-URL behavior — the SVGR component path is NOT taken.
    assert!(!should_route_svg_as_component(
        "./logo.svg",
        &ImportKind::Default,
        true,
        SvgrExportType::Named,
    ));
}

#[test]
fn imports_url_query_forces_asset_url() {
    // `import url from './logo.svg?url'` keeps the asset-URL path even when a
    // named import would otherwise route through SVGR.
    assert!(!should_route_svg_as_component(
        "./logo.svg?url",
        &ImportKind::Named,
        true,
        SvgrExportType::Named,
    ));
}

#[test]
fn imports_non_svg_assets_never_route_through_svgr() {
    // Plain image asset-URL imports are untouched by SVGR routing.
    for spec in ["./photo.png", "./icon.jpg", "./bg.webp", "./font.woff2"] {
        assert!(
            !should_route_svg_as_component(spec, &ImportKind::Named, true, SvgrExportType::Named),
            "{spec} must never route through SVGR"
        );
    }
}

// TODO(#203 follow-up): full end-to-end build of a fixture component that
// `import { ReactComponent as Icon } from './icon.svg'` and asserting the
// bundled output contains the inlined React component (plus a sibling test that
// `import url from './icon.svg?url'` still emits the hashed asset file). Wiring
// the SVGR-emitted virtual module into the bundler's module-graph loader is a
// deeper change than this slice; the routing decision + transform are unit
// covered above, and the bundler's existing `.svg` asset-URL path is unchanged.
// HANDWRITE-END
