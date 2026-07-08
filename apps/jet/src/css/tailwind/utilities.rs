// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-css-tailwind.md#schema
// CODEGEN-BEGIN
//! Tailwind utility class → CSS rule mapping.
//!
//! Implements a two-level lookup:
//! 1. Exact match against a static table for fixed utilities.
//! 2. Generative patterns for spacing, sizing, color, and typography utilities.

use std::collections::HashMap;

// ─── public API ──────────────────────────────────────────────────────────────

/// Resolve a Tailwind utility class name (without variant prefix) to its CSS
/// declaration block.
///
/// Returns `None` if the class is unknown or not supported.
/// @spec .aw/tech-design/projects/jet/semantic/jet-css-tailwind.md#schema
pub fn class_to_css(class: &str) -> Option<String> {
    // 1. Exact table
    if let Some(css) = EXACT.get(class) {
        return Some(css.to_string());
    }
    // 2. Generative patterns
    generate_utility(class)
}

/// Resolve multiple `@apply` class names to their CSS declarations.
///
/// Returns the concatenated declarations (no selector wrapper).
/// @spec .aw/tech-design/projects/jet/semantic/jet-css-tailwind.md#schema
pub fn apply_to_declarations(classes: &[&str]) -> String {
    let mut out = String::new();
    for cls in classes {
        if let Some(css) = class_to_css(cls) {
            // css is "property: value;" — strip surrounding "{ ... }" if present
            let decl = css
                .trim()
                .trim_start_matches('{')
                .trim_end_matches('}')
                .trim();
            out.push_str(decl);
            out.push(' ');
        }
    }
    out.trim().to_string()
}

// ─── exact utility table ──────────────────────────────────────────────────────

// Build the static exact-match table with common Tailwind utilities.
//
// Format: class name → CSS declaration(s) to embed inside `{ ... }`.
// These match the most frequently used Tailwind utilities.
static EXACT: std::sync::LazyLock<HashMap<&'static str, &'static str>> =
    std::sync::LazyLock::new(build_exact_table);

fn build_exact_table() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();

    // ── display ──────────────────────────────────────────────────────────────
    m.insert("block", "display: block;");
    m.insert("inline-block", "display: inline-block;");
    m.insert("inline", "display: inline;");
    m.insert("flex", "display: flex;");
    m.insert("inline-flex", "display: inline-flex;");
    m.insert("grid", "display: grid;");
    m.insert("inline-grid", "display: inline-grid;");
    m.insert("table", "display: table;");
    m.insert("table-row", "display: table-row;");
    m.insert("table-cell", "display: table-cell;");
    m.insert("contents", "display: contents;");
    m.insert("hidden", "display: none;");
    m.insert("list-item", "display: list-item;");

    // ── flexbox ───────────────────────────────────────────────────────────────
    m.insert("flex-row", "flex-direction: row;");
    m.insert("flex-row-reverse", "flex-direction: row-reverse;");
    m.insert("flex-col", "flex-direction: column;");
    m.insert("flex-col-reverse", "flex-direction: column-reverse;");
    m.insert("flex-wrap", "flex-wrap: wrap;");
    m.insert("flex-wrap-reverse", "flex-wrap: wrap-reverse;");
    m.insert("flex-nowrap", "flex-wrap: nowrap;");
    m.insert("flex-1", "flex: 1 1 0%;");
    m.insert("flex-auto", "flex: 1 1 auto;");
    m.insert("flex-initial", "flex: 0 1 auto;");
    m.insert("flex-none", "flex: none;");
    m.insert("flex-grow", "flex-grow: 1;");
    m.insert("flex-grow-0", "flex-grow: 0;");
    m.insert("flex-shrink", "flex-shrink: 1;");
    m.insert("flex-shrink-0", "flex-shrink: 0;");
    m.insert("grow", "flex-grow: 1;");
    m.insert("grow-0", "flex-grow: 0;");
    m.insert("shrink", "flex-shrink: 1;");
    m.insert("shrink-0", "flex-shrink: 0;");

    // ── align / justify ──────────────────────────────────────────────────────
    m.insert("items-start", "align-items: flex-start;");
    m.insert("items-end", "align-items: flex-end;");
    m.insert("items-center", "align-items: center;");
    m.insert("items-baseline", "align-items: baseline;");
    m.insert("items-stretch", "align-items: stretch;");
    m.insert("justify-start", "justify-content: flex-start;");
    m.insert("justify-end", "justify-content: flex-end;");
    m.insert("justify-center", "justify-content: center;");
    m.insert("justify-between", "justify-content: space-between;");
    m.insert("justify-around", "justify-content: space-around;");
    m.insert("justify-evenly", "justify-content: space-evenly;");
    m.insert("self-auto", "align-self: auto;");
    m.insert("self-start", "align-self: flex-start;");
    m.insert("self-end", "align-self: flex-end;");
    m.insert("self-center", "align-self: center;");
    m.insert("self-stretch", "align-self: stretch;");
    m.insert("self-baseline", "align-self: baseline;");
    m.insert("content-start", "align-content: flex-start;");
    m.insert("content-end", "align-content: flex-end;");
    m.insert("content-center", "align-content: center;");
    m.insert("content-between", "align-content: space-between;");
    m.insert("content-around", "align-content: space-around;");
    m.insert("content-evenly", "align-content: space-evenly;");

    // ── grid ─────────────────────────────────────────────────────────────────
    m.insert("col-auto", "grid-column: auto;");
    m.insert("col-span-full", "grid-column: 1 / -1;");
    m.insert("row-auto", "grid-row: auto;");
    m.insert("row-span-full", "grid-row: 1 / -1;");

    // ── position ─────────────────────────────────────────────────────────────
    m.insert("static", "position: static;");
    m.insert("fixed", "position: fixed;");
    m.insert("absolute", "position: absolute;");
    m.insert("relative", "position: relative;");
    m.insert("sticky", "position: sticky;");
    m.insert("inset-0", "inset: 0px;");
    m.insert("inset-auto", "inset: auto;");
    m.insert("inset-x-0", "left: 0px; right: 0px;");
    m.insert("inset-y-0", "top: 0px; bottom: 0px;");
    m.insert("top-0", "top: 0px;");
    m.insert("right-0", "right: 0px;");
    m.insert("bottom-0", "bottom: 0px;");
    m.insert("left-0", "left: 0px;");
    m.insert("top-auto", "top: auto;");
    m.insert("right-auto", "right: auto;");
    m.insert("bottom-auto", "bottom: auto;");
    m.insert("left-auto", "left: auto;");

    // ── overflow ──────────────────────────────────────────────────────────────
    m.insert("overflow-auto", "overflow: auto;");
    m.insert("overflow-hidden", "overflow: hidden;");
    m.insert("overflow-visible", "overflow: visible;");
    m.insert("overflow-scroll", "overflow: scroll;");
    m.insert("overflow-x-auto", "overflow-x: auto;");
    m.insert("overflow-x-hidden", "overflow-x: hidden;");
    m.insert("overflow-x-scroll", "overflow-x: scroll;");
    m.insert("overflow-y-auto", "overflow-y: auto;");
    m.insert("overflow-y-hidden", "overflow-y: hidden;");
    m.insert("overflow-y-scroll", "overflow-y: scroll;");

    // ── visibility ────────────────────────────────────────────────────────────
    m.insert("visible", "visibility: visible;");
    m.insert("invisible", "visibility: hidden;");
    m.insert("collapse", "visibility: collapse;");

    // ── z-index ───────────────────────────────────────────────────────────────
    m.insert("z-0", "z-index: 0;");
    m.insert("z-10", "z-index: 10;");
    m.insert("z-20", "z-index: 20;");
    m.insert("z-30", "z-index: 30;");
    m.insert("z-40", "z-index: 40;");
    m.insert("z-50", "z-index: 50;");
    m.insert("z-auto", "z-index: auto;");

    // ── sizing ────────────────────────────────────────────────────────────────
    m.insert("w-auto", "width: auto;");
    m.insert("w-full", "width: 100%;");
    m.insert("w-screen", "width: 100vw;");
    m.insert("w-min", "width: min-content;");
    m.insert("w-max", "width: max-content;");
    m.insert("w-fit", "width: fit-content;");
    m.insert("h-auto", "height: auto;");
    m.insert("h-full", "height: 100%;");
    m.insert("h-screen", "height: 100vh;");
    m.insert("h-min", "height: min-content;");
    m.insert("h-max", "height: max-content;");
    m.insert("h-fit", "height: fit-content;");
    m.insert("min-w-0", "min-width: 0px;");
    m.insert("min-w-full", "min-width: 100%;");
    m.insert("min-h-0", "min-height: 0px;");
    m.insert("min-h-full", "min-height: 100%;");
    m.insert("min-h-screen", "min-height: 100vh;");
    m.insert("max-w-none", "max-width: none;");
    m.insert("max-w-full", "max-width: 100%;");
    m.insert("max-w-xs", "max-width: 20rem;");
    m.insert("max-w-sm", "max-width: 24rem;");
    m.insert("max-w-md", "max-width: 28rem;");
    m.insert("max-w-lg", "max-width: 32rem;");
    m.insert("max-w-xl", "max-width: 36rem;");
    m.insert("max-w-2xl", "max-width: 42rem;");
    m.insert("max-w-3xl", "max-width: 48rem;");
    m.insert("max-w-4xl", "max-width: 56rem;");
    m.insert("max-w-5xl", "max-width: 64rem;");
    m.insert("max-w-6xl", "max-width: 72rem;");
    m.insert("max-w-7xl", "max-width: 80rem;");
    m.insert("max-w-screen-sm", "max-width: 640px;");
    m.insert("max-w-screen-md", "max-width: 768px;");
    m.insert("max-w-screen-lg", "max-width: 1024px;");
    m.insert("max-w-screen-xl", "max-width: 1280px;");
    m.insert("max-h-none", "max-height: none;");
    m.insert("max-h-full", "max-height: 100%;");
    m.insert("max-h-screen", "max-height: 100vh;");

    // ── typography ────────────────────────────────────────────────────────────
    m.insert("text-xs", "font-size: 0.75rem; line-height: 1rem;");
    m.insert("text-sm", "font-size: 0.875rem; line-height: 1.25rem;");
    m.insert("text-base", "font-size: 1rem; line-height: 1.5rem;");
    m.insert("text-lg", "font-size: 1.125rem; line-height: 1.75rem;");
    m.insert("text-xl", "font-size: 1.25rem; line-height: 1.75rem;");
    m.insert("text-2xl", "font-size: 1.5rem; line-height: 2rem;");
    m.insert("text-3xl", "font-size: 1.875rem; line-height: 2.25rem;");
    m.insert("text-4xl", "font-size: 2.25rem; line-height: 2.5rem;");
    m.insert("text-5xl", "font-size: 3rem; line-height: 1;");
    m.insert("text-6xl", "font-size: 3.75rem; line-height: 1;");
    m.insert("text-7xl", "font-size: 4.5rem; line-height: 1;");
    m.insert("text-8xl", "font-size: 6rem; line-height: 1;");
    m.insert("text-9xl", "font-size: 8rem; line-height: 1;");
    m.insert("font-thin", "font-weight: 100;");
    m.insert("font-extralight", "font-weight: 200;");
    m.insert("font-light", "font-weight: 300;");
    m.insert("font-normal", "font-weight: 400;");
    m.insert("font-medium", "font-weight: 500;");
    m.insert("font-semibold", "font-weight: 600;");
    m.insert("font-bold", "font-weight: 700;");
    m.insert("font-extrabold", "font-weight: 800;");
    m.insert("font-black", "font-weight: 900;");
    m.insert("italic", "font-style: italic;");
    m.insert("not-italic", "font-style: normal;");
    m.insert("leading-3", "line-height: 0.75rem;");
    m.insert("leading-4", "line-height: 1rem;");
    m.insert("leading-5", "line-height: 1.25rem;");
    m.insert("leading-6", "line-height: 1.5rem;");
    m.insert("leading-7", "line-height: 1.75rem;");
    m.insert("leading-8", "line-height: 2rem;");
    m.insert("leading-9", "line-height: 2.25rem;");
    m.insert("leading-10", "line-height: 2.5rem;");
    m.insert("leading-none", "line-height: 1;");
    m.insert("leading-tight", "line-height: 1.25;");
    m.insert("leading-snug", "line-height: 1.375;");
    m.insert("leading-normal", "line-height: 1.5;");
    m.insert("leading-relaxed", "line-height: 1.625;");
    m.insert("leading-loose", "line-height: 2;");
    m.insert("tracking-tighter", "letter-spacing: -0.05em;");
    m.insert("tracking-tight", "letter-spacing: -0.025em;");
    m.insert("tracking-normal", "letter-spacing: 0em;");
    m.insert("tracking-wide", "letter-spacing: 0.025em;");
    m.insert("tracking-wider", "letter-spacing: 0.05em;");
    m.insert("tracking-widest", "letter-spacing: 0.1em;");
    m.insert("text-left", "text-align: left;");
    m.insert("text-center", "text-align: center;");
    m.insert("text-right", "text-align: right;");
    m.insert("text-justify", "text-align: justify;");
    m.insert("text-start", "text-align: start;");
    m.insert("text-end", "text-align: end;");
    m.insert("underline", "text-decoration-line: underline;");
    m.insert("overline", "text-decoration-line: overline;");
    m.insert("line-through", "text-decoration-line: line-through;");
    m.insert("no-underline", "text-decoration-line: none;");
    m.insert("uppercase", "text-transform: uppercase;");
    m.insert("lowercase", "text-transform: lowercase;");
    m.insert("capitalize", "text-transform: capitalize;");
    m.insert("normal-case", "text-transform: none;");
    m.insert(
        "truncate",
        "overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
    );
    m.insert("text-ellipsis", "text-overflow: ellipsis;");
    m.insert("text-clip", "text-overflow: clip;");
    m.insert("whitespace-normal", "white-space: normal;");
    m.insert("whitespace-nowrap", "white-space: nowrap;");
    m.insert("whitespace-pre", "white-space: pre;");
    m.insert("whitespace-pre-line", "white-space: pre-line;");
    m.insert("whitespace-pre-wrap", "white-space: pre-wrap;");
    m.insert("whitespace-break-spaces", "white-space: break-spaces;");
    m.insert("break-normal", "overflow-wrap: normal; word-break: normal;");
    m.insert("break-words", "overflow-wrap: break-word;");
    m.insert("break-all", "word-break: break-all;");
    m.insert("break-keep", "word-break: keep-all;");

    // ── borders ───────────────────────────────────────────────────────────────
    m.insert("border", "border-width: 1px;");
    m.insert("border-0", "border-width: 0px;");
    m.insert("border-2", "border-width: 2px;");
    m.insert("border-4", "border-width: 4px;");
    m.insert("border-8", "border-width: 8px;");
    m.insert("border-t", "border-top-width: 1px;");
    m.insert("border-r", "border-right-width: 1px;");
    m.insert("border-b", "border-bottom-width: 1px;");
    m.insert("border-l", "border-left-width: 1px;");
    m.insert("border-t-0", "border-top-width: 0px;");
    m.insert("border-r-0", "border-right-width: 0px;");
    m.insert("border-b-0", "border-bottom-width: 0px;");
    m.insert("border-l-0", "border-left-width: 0px;");
    m.insert("rounded-none", "border-radius: 0px;");
    m.insert("rounded-sm", "border-radius: 0.125rem;");
    m.insert("rounded", "border-radius: 0.25rem;");
    m.insert("rounded-md", "border-radius: 0.375rem;");
    m.insert("rounded-lg", "border-radius: 0.5rem;");
    m.insert("rounded-xl", "border-radius: 0.75rem;");
    m.insert("rounded-2xl", "border-radius: 1rem;");
    m.insert("rounded-3xl", "border-radius: 1.5rem;");
    m.insert("rounded-full", "border-radius: 9999px;");
    m.insert("border-solid", "border-style: solid;");
    m.insert("border-dashed", "border-style: dashed;");
    m.insert("border-dotted", "border-style: dotted;");
    m.insert("border-double", "border-style: double;");
    m.insert("border-hidden", "border-style: hidden;");
    m.insert("border-none", "border-style: none;");

    // ── backgrounds ───────────────────────────────────────────────────────────
    m.insert("bg-transparent", "background-color: transparent;");
    m.insert("bg-current", "background-color: currentColor;");
    m.insert("bg-white", "background-color: rgb(255 255 255);");
    m.insert("bg-black", "background-color: rgb(0 0 0);");
    m.insert("bg-none", "background-image: none;");
    m.insert("bg-fixed", "background-attachment: fixed;");
    m.insert("bg-local", "background-attachment: local;");
    m.insert("bg-scroll", "background-attachment: scroll;");
    m.insert("bg-cover", "background-size: cover;");
    m.insert("bg-contain", "background-size: contain;");
    m.insert("bg-auto", "background-size: auto;");
    m.insert("bg-center", "background-position: center;");
    m.insert("bg-top", "background-position: top;");
    m.insert("bg-bottom", "background-position: bottom;");
    m.insert("bg-left", "background-position: left;");
    m.insert("bg-right", "background-position: right;");
    m.insert("bg-no-repeat", "background-repeat: no-repeat;");
    m.insert("bg-repeat", "background-repeat: repeat;");
    m.insert("bg-repeat-x", "background-repeat: repeat-x;");
    m.insert("bg-repeat-y", "background-repeat: repeat-y;");

    // ── text colors ───────────────────────────────────────────────────────────
    m.insert("text-transparent", "color: transparent;");
    m.insert("text-current", "color: currentColor;");
    m.insert("text-white", "color: rgb(255 255 255);");
    m.insert("text-black", "color: rgb(0 0 0);");

    // ── shadows ───────────────────────────────────────────────────────────────
    m.insert("shadow-sm", "box-shadow: 0 1px 2px 0 rgb(0 0 0 / 0.05);");
    m.insert(
        "shadow",
        "box-shadow: 0 1px 3px 0 rgb(0 0 0 / 0.1), 0 1px 2px -1px rgb(0 0 0 / 0.1);",
    );
    m.insert(
        "shadow-md",
        "box-shadow: 0 4px 6px -1px rgb(0 0 0 / 0.1), 0 2px 4px -2px rgb(0 0 0 / 0.1);",
    );
    m.insert(
        "shadow-lg",
        "box-shadow: 0 10px 15px -3px rgb(0 0 0 / 0.1), 0 4px 6px -4px rgb(0 0 0 / 0.1);",
    );
    m.insert(
        "shadow-xl",
        "box-shadow: 0 20px 25px -5px rgb(0 0 0 / 0.1), 0 8px 10px -6px rgb(0 0 0 / 0.1);",
    );
    m.insert(
        "shadow-2xl",
        "box-shadow: 0 25px 50px -12px rgb(0 0 0 / 0.25);",
    );
    m.insert(
        "shadow-inner",
        "box-shadow: inset 0 2px 4px 0 rgb(0 0 0 / 0.05);",
    );
    m.insert("shadow-none", "box-shadow: 0 0 #0000;");

    // ── opacity ───────────────────────────────────────────────────────────────
    m.insert("opacity-0", "opacity: 0;");
    m.insert("opacity-5", "opacity: 0.05;");
    m.insert("opacity-10", "opacity: 0.1;");
    m.insert("opacity-20", "opacity: 0.2;");
    m.insert("opacity-25", "opacity: 0.25;");
    m.insert("opacity-30", "opacity: 0.3;");
    m.insert("opacity-40", "opacity: 0.4;");
    m.insert("opacity-50", "opacity: 0.5;");
    m.insert("opacity-60", "opacity: 0.6;");
    m.insert("opacity-70", "opacity: 0.7;");
    m.insert("opacity-75", "opacity: 0.75;");
    m.insert("opacity-80", "opacity: 0.8;");
    m.insert("opacity-90", "opacity: 0.9;");
    m.insert("opacity-95", "opacity: 0.95;");
    m.insert("opacity-100", "opacity: 1;");

    // ── transitions ───────────────────────────────────────────────────────────
    m.insert("transition-none", "transition-property: none;");
    m.insert("transition-all", "transition-property: all; transition-timing-function: cubic-bezier(0.4, 0, 0.2, 1); transition-duration: 150ms;");
    m.insert("transition", "transition-property: color, background-color, border-color, text-decoration-color, fill, stroke, opacity, box-shadow, transform, filter, backdrop-filter; transition-timing-function: cubic-bezier(0.4, 0, 0.2, 1); transition-duration: 150ms;");
    m.insert("transition-colors", "transition-property: color, background-color, border-color, text-decoration-color, fill, stroke; transition-timing-function: cubic-bezier(0.4, 0, 0.2, 1); transition-duration: 150ms;");
    m.insert("transition-opacity", "transition-property: opacity; transition-timing-function: cubic-bezier(0.4, 0, 0.2, 1); transition-duration: 150ms;");
    m.insert("transition-shadow", "transition-property: box-shadow; transition-timing-function: cubic-bezier(0.4, 0, 0.2, 1); transition-duration: 150ms;");
    m.insert("transition-transform", "transition-property: transform; transition-timing-function: cubic-bezier(0.4, 0, 0.2, 1); transition-duration: 150ms;");
    m.insert("duration-75", "transition-duration: 75ms;");
    m.insert("duration-100", "transition-duration: 100ms;");
    m.insert("duration-150", "transition-duration: 150ms;");
    m.insert("duration-200", "transition-duration: 200ms;");
    m.insert("duration-300", "transition-duration: 300ms;");
    m.insert("duration-500", "transition-duration: 500ms;");
    m.insert("duration-700", "transition-duration: 700ms;");
    m.insert("duration-1000", "transition-duration: 1000ms;");
    m.insert("ease-linear", "transition-timing-function: linear;");
    m.insert(
        "ease-in",
        "transition-timing-function: cubic-bezier(0.4, 0, 1, 1);",
    );
    m.insert(
        "ease-out",
        "transition-timing-function: cubic-bezier(0, 0, 0.2, 1);",
    );
    m.insert(
        "ease-in-out",
        "transition-timing-function: cubic-bezier(0.4, 0, 0.2, 1);",
    );

    // ── cursor ────────────────────────────────────────────────────────────────
    m.insert("cursor-auto", "cursor: auto;");
    m.insert("cursor-default", "cursor: default;");
    m.insert("cursor-pointer", "cursor: pointer;");
    m.insert("cursor-wait", "cursor: wait;");
    m.insert("cursor-text", "cursor: text;");
    m.insert("cursor-move", "cursor: move;");
    m.insert("cursor-not-allowed", "cursor: not-allowed;");
    m.insert("cursor-none", "cursor: none;");
    m.insert("cursor-grab", "cursor: grab;");
    m.insert("cursor-grabbing", "cursor: grabbing;");

    // ── pointer events ────────────────────────────────────────────────────────
    m.insert("pointer-events-none", "pointer-events: none;");
    m.insert("pointer-events-auto", "pointer-events: auto;");

    // ── select ────────────────────────────────────────────────────────────────
    m.insert("select-none", "user-select: none;");
    m.insert("select-text", "user-select: text;");
    m.insert("select-all", "user-select: all;");
    m.insert("select-auto", "user-select: auto;");

    // ── outline ───────────────────────────────────────────────────────────────
    m.insert(
        "outline-none",
        "outline: 2px solid transparent; outline-offset: 2px;",
    );
    m.insert("outline", "outline-style: solid;");
    m.insert("outline-dashed", "outline-style: dashed;");
    m.insert("outline-dotted", "outline-style: dotted;");
    m.insert("outline-double", "outline-style: double;");

    // ── appearance ────────────────────────────────────────────────────────────
    m.insert("appearance-none", "appearance: none;");
    m.insert("appearance-auto", "appearance: auto;");

    // ── object-fit ────────────────────────────────────────────────────────────
    m.insert("object-contain", "object-fit: contain;");
    m.insert("object-cover", "object-fit: cover;");
    m.insert("object-fill", "object-fit: fill;");
    m.insert("object-none", "object-fit: none;");
    m.insert("object-scale-down", "object-fit: scale-down;");

    // ── spaces (space-x, space-y) ─────────────────────────────────────────────
    m.insert(
        "space-x-0 > :not([hidden]) ~ :not([hidden])",
        "margin-left: 0px;",
    );
    m.insert(
        "space-y-0 > :not([hidden]) ~ :not([hidden])",
        "margin-top: 0px;",
    );
    m.insert(
        "space-x-reverse > :not([hidden]) ~ :not([hidden])",
        "--tw-space-x-reverse: 1;",
    );
    m.insert(
        "space-y-reverse > :not([hidden]) ~ :not([hidden])",
        "--tw-space-y-reverse: 1;",
    );

    // ── ring ─────────────────────────────────────────────────────────────────
    m.insert("ring-0", "box-shadow: var(--tw-ring-inset) 0 0 0 calc(0px + var(--tw-ring-offset-width)) var(--tw-ring-color);");
    m.insert("ring-1", "box-shadow: var(--tw-ring-inset) 0 0 0 calc(1px + var(--tw-ring-offset-width)) var(--tw-ring-color);");
    m.insert("ring-2", "box-shadow: var(--tw-ring-inset) 0 0 0 calc(2px + var(--tw-ring-offset-width)) var(--tw-ring-color);");
    m.insert("ring", "box-shadow: var(--tw-ring-inset) 0 0 0 calc(3px + var(--tw-ring-offset-width)) var(--tw-ring-color);");
    m.insert("ring-4", "box-shadow: var(--tw-ring-inset) 0 0 0 calc(4px + var(--tw-ring-offset-width)) var(--tw-ring-color);");
    m.insert("ring-8", "box-shadow: var(--tw-ring-inset) 0 0 0 calc(8px + var(--tw-ring-offset-width)) var(--tw-ring-color);");
    m.insert("ring-inset", "--tw-ring-inset: inset;");

    m
}

// ─── generative utilities ─────────────────────────────────────────────────────

fn generate_utility(class: &str) -> Option<String> {
    // Spacing: p-{n}, m-{n}, px-{n}, py-{n}, pt-{n}, pr-{n}, pb-{n}, pl-{n},
    //          mt-{n}, mr-{n}, mb-{n}, ml-{n}, mx-{n}, my-{n}
    if let Some(css) = generate_spacing(class) {
        return Some(css);
    }
    // Sizing: w-{n}, h-{n}
    if let Some(css) = generate_sizing(class) {
        return Some(css);
    }
    // Colors: text-{color}-{shade}, bg-{color}-{shade}, border-{color}-{shade}
    if let Some(css) = generate_color_utility(class) {
        return Some(css);
    }
    // Arbitrary value: class-[value]
    if let Some(css) = generate_arbitrary(class) {
        return Some(css);
    }
    // Grid columns/rows
    if let Some(css) = generate_grid(class) {
        return Some(css);
    }
    None
}

fn generate_spacing(class: &str) -> Option<String> {
    // Prefix → (CSS property list)
    let prefixes: &[(&str, &[&str])] = &[
        ("p-", &["padding"]),
        ("px-", &["padding-left", "padding-right"]),
        ("py-", &["padding-top", "padding-bottom"]),
        ("pt-", &["padding-top"]),
        ("pr-", &["padding-right"]),
        ("pb-", &["padding-bottom"]),
        ("pl-", &["padding-left"]),
        ("m-", &["margin"]),
        ("mx-", &["margin-left", "margin-right"]),
        ("my-", &["margin-top", "margin-bottom"]),
        ("mt-", &["margin-top"]),
        ("mr-", &["margin-right"]),
        ("mb-", &["margin-bottom"]),
        ("ml-", &["margin-left"]),
        ("gap-", &["gap"]),
        ("gap-x-", &["column-gap"]),
        ("gap-y-", &["row-gap"]),
        ("space-x-", &["--tw-space-x: {}; margin-left: calc({} * var(--tw-space-x-reverse)); margin-right: calc({} * calc(1 - var(--tw-space-x-reverse)));"]),
        ("space-y-", &["--tw-space-y: {}; margin-top: calc({} * var(--tw-space-y-reverse)); margin-bottom: calc({} * calc(1 - var(--tw-space-y-reverse)));"]),
    ];

    for (prefix, props) in prefixes {
        if let Some(rest) = class.strip_prefix(prefix) {
            let value = parse_spacing_value(rest)?;
            if props.len() == 1 && props[0].contains("{}") {
                // Special format for space-x/space-y
                return Some(props[0].replace("{}", &value));
            }
            let declarations: String = props
                .iter()
                .map(|p| format!("{}: {};", p, value))
                .collect::<Vec<_>>()
                .join(" ");
            return Some(declarations);
        }
    }
    None
}

/// Convert a Tailwind spacing step (e.g. "4", "0.5", "px", "auto") to CSS value.
fn parse_spacing_value(s: &str) -> Option<String> {
    match s {
        "px" => return Some("1px".to_string()),
        "auto" => return Some("auto".to_string()),
        "full" => return Some("100%".to_string()),
        _ => {}
    }
    // Arbitrary value wrapped in brackets
    if let Some(arb) = s.strip_prefix('[').and_then(|v| v.strip_suffix(']')) {
        return Some(arb.to_string());
    }
    // Numeric: map to rem (1 step = 0.25rem)
    let n: f64 = s.parse().ok()?;
    if n == 0.0 {
        return Some("0px".to_string());
    }
    Some(
        format!("{:.4}", n * 0.25)
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
            + "rem",
    )
}

fn generate_sizing(class: &str) -> Option<String> {
    let (prop, rest) = if let Some(r) = class.strip_prefix("w-") {
        ("width", r)
    } else if let Some(r) = class.strip_prefix("h-") {
        ("height", r)
    } else if let Some(r) = class.strip_prefix("min-w-") {
        ("min-width", r)
    } else if let Some(r) = class.strip_prefix("min-h-") {
        ("min-height", r)
    } else if let Some(r) = class.strip_prefix("max-w-") {
        ("max-width", r)
    } else if let Some(r) = class.strip_prefix("max-h-") {
        ("max-height", r)
    } else {
        return None;
    };

    // Fractions: 1/2, 1/3, 2/3, etc.
    if rest.contains('/') {
        let parts: Vec<&str> = rest.splitn(2, '/').collect();
        if parts.len() == 2 {
            let num: f64 = parts[0].parse().ok()?;
            let den: f64 = parts[1].parse().ok()?;
            if den != 0.0 {
                return Some(format!("{}: {:.6}%;", prop, num / den * 100.0));
            }
        }
    }

    let value = parse_spacing_value(rest)?;
    Some(format!("{}: {};", prop, value))
}

fn generate_grid(class: &str) -> Option<String> {
    // grid-cols-{n}
    if let Some(rest) = class.strip_prefix("grid-cols-") {
        match rest {
            "none" => return Some("grid-template-columns: none;".to_string()),
            _ => {
                let n: u32 = rest.parse().ok()?;
                return Some(format!(
                    "grid-template-columns: repeat({}, minmax(0, 1fr));",
                    n
                ));
            }
        }
    }
    // grid-rows-{n}
    if let Some(rest) = class.strip_prefix("grid-rows-") {
        match rest {
            "none" => return Some("grid-template-rows: none;".to_string()),
            _ => {
                let n: u32 = rest.parse().ok()?;
                return Some(format!(
                    "grid-template-rows: repeat({}, minmax(0, 1fr));",
                    n
                ));
            }
        }
    }
    // col-span-{n}
    if let Some(rest) = class.strip_prefix("col-span-") {
        let n: u32 = rest.parse().ok()?;
        return Some(format!("grid-column: span {} / span {};", n, n));
    }
    // row-span-{n}
    if let Some(rest) = class.strip_prefix("row-span-") {
        let n: u32 = rest.parse().ok()?;
        return Some(format!("grid-row: span {} / span {};", n, n));
    }
    None
}

// ─── color utilities ──────────────────────────────────────────────────────────

fn generate_color_utility(class: &str) -> Option<String> {
    let (prop_prefix, rest) = if let Some(r) = class.strip_prefix("text-") {
        ("color", r)
    } else if let Some(r) = class.strip_prefix("bg-") {
        ("background-color", r)
    } else if let Some(r) = class.strip_prefix("border-") {
        ("border-color", r)
    } else if let Some(r) = class.strip_prefix("ring-") {
        ("--tw-ring-color", r)
    } else if let Some(r) = class.strip_prefix("fill-") {
        ("fill", r)
    } else if let Some(r) = class.strip_prefix("stroke-") {
        ("stroke", r)
    } else if let Some(r) = class.strip_prefix("placeholder-") {
        ("color", r) // handled with ::placeholder pseudo
    } else {
        return None;
    };

    // Attempt color resolution
    let color_value = resolve_color(rest)?;
    Some(format!("{}: {};", prop_prefix, color_value))
}

/// Resolve a Tailwind color name (e.g. `blue-500`, `gray-900`) to a CSS value.
/// @spec .aw/tech-design/projects/jet/semantic/jet-css-tailwind.md#schema
pub fn resolve_color(name: &str) -> Option<String> {
    // Arbitrary color: [#fff] or [rgb(...)]
    if let Some(arb) = name.strip_prefix('[').and_then(|v| v.strip_suffix(']')) {
        return Some(arb.to_string());
    }

    // Split into color-shade: "blue-500" → ("blue", 500)
    let dash = name.rfind('-')?;
    let color_name = &name[..dash];
    let shade_str = &name[dash + 1..];
    let shade: u32 = shade_str.parse().ok()?;

    let rgb = tailwind_color_rgb(color_name, shade)?;
    Some(format!("rgb({} {} {})", rgb.0, rgb.1, rgb.2))
}

/// Return the (r, g, b) tuple for a Tailwind color at a given shade.
fn tailwind_color_rgb(color: &str, shade: u32) -> Option<(u8, u8, u8)> {
    // Tailwind v3 color palette (key shades only)
    let table: &[(&str, &[(u32, (u8, u8, u8))])] = &[
        (
            "slate",
            &[
                (50, (248, 250, 252)),
                (100, (241, 245, 249)),
                (200, (226, 232, 240)),
                (300, (203, 213, 225)),
                (400, (148, 163, 184)),
                (500, (100, 116, 139)),
                (600, (71, 85, 105)),
                (700, (51, 65, 85)),
                (800, (30, 41, 59)),
                (900, (15, 23, 42)),
                (950, (2, 6, 23)),
            ],
        ),
        (
            "gray",
            &[
                (50, (249, 250, 251)),
                (100, (243, 244, 246)),
                (200, (229, 231, 235)),
                (300, (209, 213, 219)),
                (400, (156, 163, 175)),
                (500, (107, 114, 128)),
                (600, (75, 85, 99)),
                (700, (55, 65, 81)),
                (800, (31, 41, 55)),
                (900, (17, 24, 39)),
                (950, (3, 7, 18)),
            ],
        ),
        (
            "zinc",
            &[
                (50, (250, 250, 250)),
                (100, (244, 244, 245)),
                (200, (228, 228, 231)),
                (300, (212, 212, 216)),
                (400, (161, 161, 170)),
                (500, (113, 113, 122)),
                (600, (82, 82, 91)),
                (700, (63, 63, 70)),
                (800, (39, 39, 42)),
                (900, (24, 24, 27)),
                (950, (9, 9, 11)),
            ],
        ),
        (
            "red",
            &[
                (50, (254, 242, 242)),
                (100, (254, 226, 226)),
                (200, (254, 202, 202)),
                (300, (252, 165, 165)),
                (400, (248, 113, 113)),
                (500, (239, 68, 68)),
                (600, (220, 38, 38)),
                (700, (185, 28, 28)),
                (800, (153, 27, 27)),
                (900, (127, 29, 29)),
                (950, (69, 10, 10)),
            ],
        ),
        (
            "orange",
            &[
                (50, (255, 247, 237)),
                (100, (255, 237, 213)),
                (200, (254, 215, 170)),
                (300, (253, 186, 116)),
                (400, (251, 146, 60)),
                (500, (249, 115, 22)),
                (600, (234, 88, 12)),
                (700, (194, 65, 12)),
                (800, (154, 52, 18)),
                (900, (124, 45, 18)),
                (950, (67, 20, 7)),
            ],
        ),
        (
            "amber",
            &[
                (50, (255, 251, 235)),
                (100, (254, 243, 199)),
                (200, (253, 230, 138)),
                (300, (252, 211, 77)),
                (400, (251, 191, 36)),
                (500, (245, 158, 11)),
                (600, (217, 119, 6)),
                (700, (180, 83, 9)),
                (800, (146, 64, 14)),
                (900, (120, 53, 15)),
                (950, (69, 26, 3)),
            ],
        ),
        (
            "yellow",
            &[
                (50, (254, 252, 232)),
                (100, (254, 249, 195)),
                (200, (254, 240, 138)),
                (300, (253, 224, 71)),
                (400, (250, 204, 21)),
                (500, (234, 179, 8)),
                (600, (202, 138, 4)),
                (700, (161, 98, 7)),
                (800, (133, 77, 14)),
                (900, (113, 63, 18)),
                (950, (66, 32, 6)),
            ],
        ),
        (
            "green",
            &[
                (50, (240, 253, 244)),
                (100, (220, 252, 231)),
                (200, (187, 247, 208)),
                (300, (134, 239, 172)),
                (400, (74, 222, 128)),
                (500, (34, 197, 94)),
                (600, (22, 163, 74)),
                (700, (21, 128, 61)),
                (800, (22, 101, 52)),
                (900, (20, 83, 45)),
                (950, (5, 46, 22)),
            ],
        ),
        (
            "teal",
            &[
                (50, (240, 253, 250)),
                (100, (204, 251, 241)),
                (200, (153, 246, 228)),
                (300, (94, 234, 212)),
                (400, (45, 212, 191)),
                (500, (20, 184, 166)),
                (600, (13, 148, 136)),
                (700, (15, 118, 110)),
                (800, (17, 94, 89)),
                (900, (19, 78, 74)),
                (950, (4, 47, 46)),
            ],
        ),
        (
            "blue",
            &[
                (50, (239, 246, 255)),
                (100, (219, 234, 254)),
                (200, (191, 219, 254)),
                (300, (147, 197, 253)),
                (400, (96, 165, 250)),
                (500, (59, 130, 246)),
                (600, (37, 99, 235)),
                (700, (29, 78, 216)),
                (800, (30, 64, 175)),
                (900, (30, 58, 138)),
                (950, (23, 37, 84)),
            ],
        ),
        (
            "indigo",
            &[
                (50, (238, 242, 255)),
                (100, (224, 231, 255)),
                (200, (199, 210, 254)),
                (300, (165, 180, 252)),
                (400, (129, 140, 248)),
                (500, (99, 102, 241)),
                (600, (79, 70, 229)),
                (700, (67, 56, 202)),
                (800, (55, 48, 163)),
                (900, (49, 46, 129)),
                (950, (30, 27, 75)),
            ],
        ),
        (
            "violet",
            &[
                (50, (245, 243, 255)),
                (100, (237, 233, 254)),
                (200, (221, 214, 254)),
                (300, (196, 181, 253)),
                (400, (167, 139, 250)),
                (500, (139, 92, 246)),
                (600, (124, 58, 237)),
                (700, (109, 40, 217)),
                (800, (91, 33, 182)),
                (900, (76, 29, 149)),
                (950, (46, 16, 101)),
            ],
        ),
        (
            "pink",
            &[
                (50, (253, 242, 248)),
                (100, (252, 231, 243)),
                (200, (251, 207, 232)),
                (300, (249, 168, 212)),
                (400, (244, 114, 182)),
                (500, (236, 72, 153)),
                (600, (219, 39, 119)),
                (700, (190, 24, 93)),
                (800, (157, 23, 77)),
                (900, (131, 24, 67)),
                (950, (80, 7, 36)),
            ],
        ),
        (
            "rose",
            &[
                (50, (255, 241, 242)),
                (100, (255, 228, 230)),
                (200, (254, 205, 211)),
                (300, (253, 164, 175)),
                (400, (251, 113, 133)),
                (500, (244, 63, 94)),
                (600, (225, 29, 72)),
                (700, (190, 18, 60)),
                (800, (159, 18, 57)),
                (900, (136, 19, 55)),
                (950, (76, 5, 25)),
            ],
        ),
    ];

    for (name, shades) in table {
        if *name == color {
            for (s, rgb) in *shades {
                if *s == shade {
                    return Some(*rgb);
                }
            }
        }
    }
    None
}

// ─── arbitrary value utilities ────────────────────────────────────────────────

fn generate_arbitrary(class: &str) -> Option<String> {
    // Format: prefix-[value], e.g. w-[300px], text-[#ff0000], mt-[1.5rem]
    let bracket_pos = class.find("[")?;
    let prefix = &class[..bracket_pos];
    let value = class
        .get(bracket_pos..)?
        .strip_prefix('[')?
        .strip_suffix(']')?;

    match prefix.trim_end_matches('-') {
        "w" => Some(format!("width: {};", value)),
        "h" => Some(format!("height: {};", value)),
        "min-w" => Some(format!("min-width: {};", value)),
        "min-h" => Some(format!("min-height: {};", value)),
        "max-w" => Some(format!("max-width: {};", value)),
        "max-h" => Some(format!("max-height: {};", value)),
        "text" => Some(format!("color: {};", value)),
        "bg" => Some(format!("background-color: {};", value)),
        "border" => Some(format!("border-color: {};", value)),
        "p" => Some(format!("padding: {};", value)),
        "px" => Some(format!("padding-left: {0}; padding-right: {0};", value)),
        "py" => Some(format!("padding-top: {0}; padding-bottom: {0};", value)),
        "pt" => Some(format!("padding-top: {};", value)),
        "pr" => Some(format!("padding-right: {};", value)),
        "pb" => Some(format!("padding-bottom: {};", value)),
        "pl" => Some(format!("padding-left: {};", value)),
        "m" => Some(format!("margin: {};", value)),
        "mx" => Some(format!("margin-left: {0}; margin-right: {0};", value)),
        "my" => Some(format!("margin-top: {0}; margin-bottom: {0};", value)),
        "mt" => Some(format!("margin-top: {};", value)),
        "mr" => Some(format!("margin-right: {};", value)),
        "mb" => Some(format!("margin-bottom: {};", value)),
        "ml" => Some(format!("margin-left: {};", value)),
        "top" => Some(format!("top: {};", value)),
        "right" => Some(format!("right: {};", value)),
        "bottom" => Some(format!("bottom: {};", value)),
        "left" => Some(format!("left: {};", value)),
        "gap" => Some(format!("gap: {};", value)),
        "rotate" => Some(format!(
            "--tw-rotate: {}; transform: rotate(var(--tw-rotate));",
            value
        )),
        "translate-x" => Some(format!(
            "--tw-translate-x: {}; transform: translateX(var(--tw-translate-x));",
            value
        )),
        "translate-y" => Some(format!(
            "--tw-translate-y: {}; transform: translateY(var(--tw-translate-y));",
            value
        )),
        "scale" => Some(format!(
            "--tw-scale-x: {0}; --tw-scale-y: {0}; transform: scale({0});",
            value
        )),
        "duration" => Some(format!("transition-duration: {};", value)),
        "z" => Some(format!("z-index: {};", value)),
        "opacity" => Some(format!("opacity: {};", value)),
        _ => None,
    }
}

// ─── tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── spacing exact values ──────────────────────────────────────────────────

    /// Verify p-4 produces exactly 'padding: 1rem;' (no double-rem regression).
    #[test]
    fn spacing_p4_exact_value() {
        let css = class_to_css("p-4").expect("p-4 should resolve");
        assert_eq!(
            css, "padding: 1rem;",
            "p-4 must produce padding: 1rem;, got: {}",
            css
        );
    }

    /// Verify m-2 produces exactly 'margin: 0.5rem;'.
    #[test]
    fn spacing_m2_exact_value() {
        let css = class_to_css("m-2").expect("m-2 should resolve");
        assert_eq!(
            css, "margin: 0.5rem;",
            "m-2 must produce margin: 0.5rem;, got: {}",
            css
        );
    }

    /// Verify gap-3 produces exactly 'gap: 0.75rem;'.
    #[test]
    fn spacing_gap3_exact_value() {
        let css = class_to_css("gap-3").expect("gap-3 should resolve");
        assert_eq!(
            css, "gap: 0.75rem;",
            "gap-3 must produce gap: 0.75rem;, got: {}",
            css
        );
    }

    /// Verify px-4 produces left+right padding each 1rem (no double-rem).
    #[test]
    fn spacing_px4_exact_value() {
        let css = class_to_css("px-4").expect("px-4 should resolve");
        assert!(
            css.contains("padding-left: 1rem") && css.contains("padding-right: 1rem"),
            "px-4 must produce padding-left: 1rem and padding-right: 1rem, got: {}",
            css
        );
    }

    /// Verify p-0 produces 'padding: 0px;' (zero special-case).
    #[test]
    fn spacing_p0_zero_case() {
        let css = class_to_css("p-0").expect("p-0 should resolve");
        assert_eq!(
            css, "padding: 0px;",
            "p-0 must produce padding: 0px;, got: {}",
            css
        );
    }

    /// Verify p-0.5 produces 'padding: 0.125rem;' (fractional step).
    #[test]
    fn spacing_p05_fractional() {
        let css = class_to_css("p-0.5").expect("p-0.5 should resolve");
        assert_eq!(
            css, "padding: 0.125rem;",
            "p-0.5 must produce padding: 0.125rem;, got: {}",
            css
        );
    }

    // ── sizing exact values ───────────────────────────────────────────────────

    /// Verify w-4 produces exactly 'width: 1rem;'.
    #[test]
    fn sizing_w4_exact_value() {
        let css = class_to_css("w-4").expect("w-4 should resolve");
        assert_eq!(
            css, "width: 1rem;",
            "w-4 must produce width: 1rem;, got: {}",
            css
        );
    }

    /// Verify h-8 produces exactly 'height: 2rem;'.
    #[test]
    fn sizing_h8_exact_value() {
        let css = class_to_css("h-8").expect("h-8 should resolve");
        assert_eq!(
            css, "height: 2rem;",
            "h-8 must produce height: 2rem;, got: {}",
            css
        );
    }

    // ── arbitrary value exact output ──────────────────────────────────────────

    /// Verify w-[300px] passes the arbitrary value through unchanged.
    #[test]
    fn arbitrary_w_300px_passthrough() {
        let css = class_to_css("w-[300px]").expect("w-[300px] should resolve");
        assert_eq!(
            css, "width: 300px;",
            "w-[300px] must produce width: 300px;, got: {}",
            css
        );
    }

    // ── color utility exact output ────────────────────────────────────────────

    /// Verify text-[#ff0000] produces 'color: #ff0000;'.
    #[test]
    fn arbitrary_text_color_passthrough() {
        let css = class_to_css("text-[#ff0000]").expect("text-[#ff0000] should resolve");
        assert_eq!(
            css, "color: #ff0000;",
            "text-[#ff0000] must produce color: #ff0000;, got: {}",
            css
        );
    }

    // ── color utility generative pattern (TR1) ──────────────────────────────

    /// S1: bg-blue-500 → background-color with Tailwind v3 blue-500 RGB.
    #[test]
    fn color_bg_blue_500() {
        let css = class_to_css("bg-blue-500").expect("bg-blue-500 should resolve");
        assert_eq!(
            css, "background-color: rgb(59 130 246);",
            "bg-blue-500 must produce background-color: rgb(59 130 246);, got: {}",
            css
        );
    }

    /// S2: text-red-600 → color with Tailwind v3 red-600 RGB.
    #[test]
    fn color_text_red_600() {
        let css = class_to_css("text-red-600").expect("text-red-600 should resolve");
        assert_eq!(
            css, "color: rgb(220 38 38);",
            "text-red-600 must produce color: rgb(220 38 38);, got: {}",
            css
        );
    }

    /// S3: border-green-300 → border-color with Tailwind v3 green-300 RGB.
    #[test]
    fn color_border_green_300() {
        let css = class_to_css("border-green-300").expect("border-green-300 should resolve");
        assert_eq!(
            css, "border-color: rgb(134 239 172);",
            "border-green-300 must produce border-color: rgb(134 239 172);, got: {}",
            css
        );
    }

    // ── typography utility exact table (TR2) ─────────────────────────────────

    /// S4: text-lg → font-size and line-height from exact table.
    #[test]
    fn typography_text_lg() {
        let css = class_to_css("text-lg").expect("text-lg should resolve");
        assert_eq!(
            css, "font-size: 1.125rem; line-height: 1.75rem;",
            "text-lg must produce font-size: 1.125rem; line-height: 1.75rem;, got: {}",
            css
        );
    }

    /// S5: font-bold → font-weight: 700.
    #[test]
    fn typography_font_bold() {
        let css = class_to_css("font-bold").expect("font-bold should resolve");
        assert_eq!(
            css, "font-weight: 700;",
            "font-bold must produce font-weight: 700;, got: {}",
            css
        );
    }

    /// S6: text-center → text-align: center.
    #[test]
    fn typography_text_center() {
        let css = class_to_css("text-center").expect("text-center should resolve");
        assert_eq!(
            css, "text-align: center;",
            "text-center must produce text-align: center;, got: {}",
            css
        );
    }

    // ── display and layout utilities (TR3) ───────────────────────────────────

    /// S7: grid, block, hidden, inline-flex → correct display values.
    #[test]
    fn display_grid_block_hidden_inline_flex() {
        let cases = [
            ("grid", "display: grid;"),
            ("block", "display: block;"),
            ("hidden", "display: none;"),
            ("inline-flex", "display: inline-flex;"),
        ];
        for (class, expected) in &cases {
            let css = class_to_css(class).unwrap_or_else(|| panic!("{} should resolve", class));
            assert_eq!(
                css, *expected,
                "{} must produce {}, got: {}",
                class, expected, css
            );
        }
    }

    // ── grid template generative pattern (TR4) ───────────────────────────────

    /// S8: grid-cols-3 → grid-template-columns: repeat(3, minmax(0, 1fr)).
    #[test]
    fn grid_cols_3_template() {
        let css = class_to_css("grid-cols-3").expect("grid-cols-3 should resolve");
        assert_eq!(
            css, "grid-template-columns: repeat(3, minmax(0, 1fr));",
            "grid-cols-3 must produce grid-template-columns: repeat(3, minmax(0, 1fr));, got: {}",
            css
        );
    }

    /// S9: col-span-2 → grid-column: span 2 / span 2.
    #[test]
    fn grid_col_span_2() {
        let css = class_to_css("col-span-2").expect("col-span-2 should resolve");
        assert_eq!(
            css, "grid-column: span 2 / span 2;",
            "col-span-2 must produce grid-column: span 2 / span 2;, got: {}",
            css
        );
    }
}
// CODEGEN-END
