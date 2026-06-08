// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-css-plugins.md#schema
// CODEGEN-BEGIN
//! Native Rust emitter for `tailwindcss-animate`.
//!
//! Implements keyframe animations and animation utility classes equivalent to
//! the `tailwindcss-animate` npm package, without JS execution.

use std::collections::HashSet;

/// Emit CSS for all `tailwindcss-animate` keyframes and animation utilities
/// that are referenced in `used_classes`.
///
/// Returns the CSS string to append to the utilities layer.
/// @spec .aw/tech-design/projects/jet/semantic/jet-css-plugins.md#schema
pub fn emit(used_classes: &HashSet<String>) -> String {
    let mut keyframes = String::new();
    let mut utilities = String::new();

    for animation in ANIMATIONS {
        let class_name = animation.class;
        // Emit if the animation class is used, or if any variant of it is used
        let is_used = used_classes
            .iter()
            .any(|c| c == class_name || c.ends_with(&format!(":{}", class_name)));

        if !is_used {
            continue;
        }

        // Emit @keyframes (deduplicate by keyframe name)
        if !keyframes.contains(&format!("@keyframes {}", animation.keyframe_name)) {
            keyframes.push_str(animation.keyframes_css);
            keyframes.push('\n');
        }

        // Emit utility class
        utilities.push_str(&format!(
            ".{} {{ {} }}\n",
            class_name, animation.utility_css
        ));
    }

    let mut out = String::new();
    if !keyframes.is_empty() || !utilities.is_empty() {
        out.push_str("/* tailwindcss-animate */\n");
        out.push_str(&keyframes);
        out.push_str(&utilities);
    }
    out
}

// ─── tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn cls(names: &[&str]) -> HashSet<String> {
        names.iter().map(|s| s.to_string()).collect()
    }

    /// T13: tailwindcss-animate Keyframes (R8)
    ///
    /// Verifies that animate-spin emits the correct @keyframes and utility class.
    #[test]
    fn t13_tailwindcss_animate_spin_keyframes() {
        let used = cls(&["animate-spin"]);
        let css = emit(&used);

        assert!(!css.is_empty(), "Should emit CSS for animate-spin");
        assert!(
            css.contains("@keyframes spin"),
            "Should emit @keyframes spin, got: {}",
            css
        );
        assert!(
            css.contains("rotate(360deg)"),
            "Should emit rotate(360deg) in spin keyframes, got: {}",
            css
        );
        assert!(
            css.contains(".animate-spin"),
            "Should emit .animate-spin utility class, got: {}",
            css
        );
        assert!(
            css.contains("animation: spin 1s linear infinite"),
            "Should emit spin animation declaration, got: {}",
            css
        );
    }

    /// Unit test: animate-ping emits its own keyframes.
    #[test]
    fn animate_ping_keyframes() {
        let used = cls(&["animate-ping"]);
        let css = emit(&used);
        assert!(
            css.contains("@keyframes ping"),
            "Should emit @keyframes ping: {}",
            css
        );
        assert!(
            css.contains(".animate-ping"),
            "Should emit .animate-ping: {}",
            css
        );
    }

    /// Unit test: unused animation classes are not emitted.
    #[test]
    fn unused_animations_not_emitted() {
        let used = cls(&["flex"]); // No animate-* classes
        let css = emit(&used);
        assert!(
            css.is_empty(),
            "Should emit nothing when no animate-* classes are used: {}",
            css
        );
    }

    /// Unit test: multiple animation classes can be emitted together.
    #[test]
    fn multiple_animations_emitted() {
        let used = cls(&["animate-spin", "animate-pulse"]);
        let css = emit(&used);
        assert!(
            css.contains("@keyframes spin"),
            "Should emit spin keyframes: {}",
            css
        );
        assert!(
            css.contains("@keyframes pulse"),
            "Should emit pulse keyframes: {}",
            css
        );
        assert!(
            css.contains(".animate-spin"),
            "Should emit .animate-spin: {}",
            css
        );
        assert!(
            css.contains(".animate-pulse"),
            "Should emit .animate-pulse: {}",
            css
        );
    }

    /// Unit test: animate-bounce emits complex multi-stop keyframes.
    #[test]
    fn animate_bounce_keyframes() {
        let used = cls(&["animate-bounce"]);
        let css = emit(&used);
        assert!(
            css.contains("@keyframes bounce"),
            "Should emit @keyframes bounce: {}",
            css
        );
        assert!(
            css.contains("translateY"),
            "Should emit translateY in bounce: {}",
            css
        );
    }
}

struct AnimationDef {
    class: &'static str,
    keyframe_name: &'static str,
    keyframes_css: &'static str,
    utility_css: &'static str,
}

static ANIMATIONS: &[AnimationDef] = &[
    // animate-spin
    AnimationDef {
        class: "animate-spin",
        keyframe_name: "spin",
        keyframes_css: "@keyframes spin {\n  to { transform: rotate(360deg); }\n}",
        utility_css: "animation: spin 1s linear infinite;",
    },
    // animate-ping
    AnimationDef {
        class: "animate-ping",
        keyframe_name: "ping",
        keyframes_css: "@keyframes ping {\n  75%, 100% { transform: scale(2); opacity: 0; }\n}",
        utility_css: "animation: ping 1s cubic-bezier(0, 0, 0.2, 1) infinite;",
    },
    // animate-pulse
    AnimationDef {
        class: "animate-pulse",
        keyframe_name: "pulse",
        keyframes_css: "@keyframes pulse {\n  50% { opacity: 0.5; }\n}",
        utility_css: "animation: pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;",
    },
    // animate-bounce
    AnimationDef {
        class: "animate-bounce",
        keyframe_name: "bounce",
        keyframes_css: concat!(
            "@keyframes bounce {\n",
            "  0%, 100% {\n",
            "    transform: translateY(-25%);\n",
            "    animation-timing-function: cubic-bezier(0.8, 0, 1, 1);\n",
            "  }\n",
            "  50% {\n",
            "    transform: none;\n",
            "    animation-timing-function: cubic-bezier(0, 0, 0.2, 1);\n",
            "  }\n",
            "}",
        ),
        utility_css: "animation: bounce 1s infinite;",
    },
    // animate-none
    AnimationDef {
        class: "animate-none",
        keyframe_name: "__none__",
        keyframes_css: "",
        utility_css: "animation: none;",
    },
    // animate-enter (fade/slide in — tailwindcss-animate specific)
    AnimationDef {
        class: "animate-in",
        keyframe_name: "enter",
        keyframes_css: concat!(
            "@keyframes enter {\n",
            "  from {\n",
            "    opacity: var(--tw-enter-opacity, 1);\n",
            "    transform: translate3d(var(--tw-enter-translate-x, 0), var(--tw-enter-translate-y, 0), 0)\n",
            "      scale3d(var(--tw-enter-scale, 1), var(--tw-enter-scale, 1), var(--tw-enter-scale, 1))\n",
            "      rotate(var(--tw-enter-rotate, 0));\n",
            "  }\n",
            "}",
        ),
        utility_css: "animation-duration: 150ms; animation-fill-mode: both; --tw-animate-duration: 150ms; animation-name: enter;",
    },
    // animate-out
    AnimationDef {
        class: "animate-out",
        keyframe_name: "exit",
        keyframes_css: concat!(
            "@keyframes exit {\n",
            "  to {\n",
            "    opacity: var(--tw-exit-opacity, 1);\n",
            "    transform: translate3d(var(--tw-exit-translate-x, 0), var(--tw-exit-translate-y, 0), 0)\n",
            "      scale3d(var(--tw-exit-scale, 1), var(--tw-exit-scale, 1), var(--tw-exit-scale, 1))\n",
            "      rotate(var(--tw-exit-rotate, 0));\n",
            "  }\n",
            "}",
        ),
        utility_css: "animation-duration: 150ms; animation-fill-mode: both; --tw-animate-duration: 150ms; animation-name: exit;",
    },
    // fade-in
    AnimationDef {
        class: "fade-in",
        keyframe_name: "fade-in",
        keyframes_css: "@keyframes fade-in {\n  from { opacity: 0; }\n}",
        utility_css: "--tw-enter-opacity: 0;",
    },
    // fade-out
    AnimationDef {
        class: "fade-out",
        keyframe_name: "fade-out",
        keyframes_css: "@keyframes fade-out {\n  to { opacity: 0; }\n}",
        utility_css: "--tw-exit-opacity: 0;",
    },
    // zoom-in
    AnimationDef {
        class: "zoom-in",
        keyframe_name: "zoom-in",
        keyframes_css: "@keyframes zoom-in {\n  from { transform: scale(0.95); }\n}",
        utility_css: "--tw-enter-scale: 0.95;",
    },
    // zoom-out
    AnimationDef {
        class: "zoom-out",
        keyframe_name: "zoom-out",
        keyframes_css: "@keyframes zoom-out {\n  to { transform: scale(0.95); }\n}",
        utility_css: "--tw-exit-scale: 0.95;",
    },
    // slide-in-from-top
    AnimationDef {
        class: "slide-in-from-top",
        keyframe_name: "slide-in-from-top",
        keyframes_css: "@keyframes slide-in-from-top {\n  from { transform: translateY(-100%); }\n}",
        utility_css: "--tw-enter-translate-y: -100%;",
    },
    // slide-in-from-bottom
    AnimationDef {
        class: "slide-in-from-bottom",
        keyframe_name: "slide-in-from-bottom",
        keyframes_css: "@keyframes slide-in-from-bottom {\n  from { transform: translateY(100%); }\n}",
        utility_css: "--tw-enter-translate-y: 100%;",
    },
    // slide-in-from-left
    AnimationDef {
        class: "slide-in-from-left",
        keyframe_name: "slide-in-from-left",
        keyframes_css: "@keyframes slide-in-from-left {\n  from { transform: translateX(-100%); }\n}",
        utility_css: "--tw-enter-translate-x: -100%;",
    },
    // slide-in-from-right
    AnimationDef {
        class: "slide-in-from-right",
        keyframe_name: "slide-in-from-right",
        keyframes_css: "@keyframes slide-in-from-right {\n  from { transform: translateX(100%); }\n}",
        utility_css: "--tw-enter-translate-x: 100%;",
    },
];
// CODEGEN-END
