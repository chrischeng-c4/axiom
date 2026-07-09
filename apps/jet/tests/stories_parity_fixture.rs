//! In-repo AC2/AC3 evidence for epic #1001 (WI #1343): a decorators + `play()`
//! interaction + `argTypes` override + MDX docs page fixture, verified through
//! the real [`jet::stories::build_stories_static`] static-export pipeline —
//! without depending on an external, manually-run Storybook install.
//!
//! Fixture: `tests/stories/fixtures/parity/` (`Widget.tsx` +
//! `Widget.stories.tsx` + `Widget.mdx`).

use std::fs;
use std::path::PathBuf;

use jet::stories::build_stories_static;
use tempfile::TempDir;

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/stories/fixtures/parity")
}

/// `build_stories_static` compiles the parity fixture with zero diagnostics,
/// the emitted static module preserves the decorators/argTypes/play() source
/// text verbatim, and the MDX docs page wires to the Interactive story.
#[test]
fn parity_fixture_compiles_with_decorators_play_argtypes_and_mdx() {
    let root = fixture_root();
    let out = TempDir::new().expect("temp out dir");
    let result = build_stories_static(&root, out.path()).expect("build parity fixture");

    // Zero diagnostics: the fixture is a clean, valid CSF + MDX suite.
    assert!(
        result.diagnostics.is_empty(),
        "parity fixture must build with zero diagnostics: {:?}",
        result.diagnostics
    );

    // One story discovered (Interactive), one docs page (Widget.mdx).
    assert_eq!(result.story_count, 1, "single Interactive story discovered");

    // The emitted static module preserves the decorator (meta-level) and the
    // argTypes/play() source text (story-level) verbatim.
    let module = fs::read_to_string(out.path().join("modules/Widget.stories.js"))
        .expect("read emitted Widget.stories module");
    assert!(
        module.contains("jet-widget-frame"),
        "decorator's wrapping markup preserved in emitted module: {module}"
    );
    assert!(
        module.contains("decorators"),
        "decorators field preserved in emitted module: {module}"
    );
    assert!(
        module.contains("argTypes") && module.contains("control") && module.contains("number"),
        "argTypes override preserved verbatim in emitted module: {module}"
    );
    assert!(
        module.contains("play")
            && module.contains("canvasElement")
            && module.contains("querySelector"),
        "play() interaction preserved verbatim in emitted module: {module}"
    );

    // The MDX docs page wires to the Interactive story: the manager embeds a
    // docs page whose Source doc block reproduces the raw story source
    // (captured by the CSF parser, independent of JS transpilation), and the
    // Canvas doc block links a preview for the resolved Interactive story.
    let manager = fs::read_to_string(out.path().join("index.html")).expect("read manager");
    assert!(
        manager.contains("class=\"jet-docs-link\""),
        "Widget.mdx registered as a docs page"
    );
    assert!(
        manager.contains("export const Interactive"),
        "Source doc block reproduces the raw Interactive story source: {manager}"
    );
    assert!(
        manager.contains("play: async"),
        "Source doc block preserves the play() interaction verbatim: {manager}"
    );
    // The Canvas doc block resolved `WidgetStories.Interactive` to the real
    // story id and links its emitted preview.
    let index_json = fs::read_to_string(out.path().join("index.json")).expect("read index.json");
    assert!(
        index_json.contains("\"id\":\"components-widget--interactive\""),
        "Interactive story id present in the static index: {index_json}"
    );
    assert!(
        manager.contains("preview/components-widget--interactive.html"),
        "MDX Canvas doc block links the resolved Interactive story preview: {manager}"
    );
    let preview = out
        .path()
        .join("preview/components-widget--interactive.html");
    assert!(preview.is_file(), "linked preview file exists: {preview:?}");
}
