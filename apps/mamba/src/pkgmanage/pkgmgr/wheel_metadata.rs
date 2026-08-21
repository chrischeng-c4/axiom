// PEP 491 `.dist-info/WHEEL` reader + writer (Tick 65).
//
// Every wheel ships a `WHEEL` file alongside `METADATA` and `RECORD`.
// It's small — five fields max — but installers must read it to know
// whether to lay the contents into `site-packages/` (purelib) or
// `site-packages/<arch>/` (platlib), to confirm the Wheel-Version is
// supported, and to verify the Tag matches the installer's compat set.
//
// Format (RFC 822-style):
//   Wheel-Version: 1.0
//   Generator: bdist_wheel (0.40.0)
//   Root-Is-Purelib: true
//   Tag: py3-none-any
//   Tag: cp310-cp310-manylinux_2_17_x86_64
//   Build: 1
//
// Rules from PEP 427 / PEP 491:
//   * Wheel-Version is `<major>.<minor>` — installers MUST refuse to
//     install if `major` is higher than they support. We surface the
//     raw string and a parsed `(major, minor)` so callers can gate.
//   * Generator is opaque text (e.g. `bdist_wheel (0.40.0)` or
//     `flit 3.10.1`). We pass it through untouched.
//   * Root-Is-Purelib is `true` / `false` (case-insensitive in
//     practice — older bdist_wheel emitted `True`).
//   * Tag is a `py-abi-platform` triple. Wheels with multiple tags
//     (fat / universal wheels) repeat the Tag line. We collect them
//     into a `Vec<String>` preserving order.
//   * Build is optional, free-form (PEP 427 only requires it to start
//     with a digit when present).
//
// Pure-data: no filesystem, no version-gate enforcement. Callers
// decide whether to refuse a wheel with an unsupported version.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WheelMetadata {
    pub wheel_version: String,
    pub generator: String,
    pub root_is_purelib: bool,
    pub tags: Vec<String>,
    pub build: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WheelMetadataError {
    InvalidLine { lineno: usize, detail: String },
    MissingField(String),
    InvalidBool { field: String, value: String },
    InvalidVersion(String),
    NoTags,
}

impl std::fmt::Display for WheelMetadataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WheelMetadataError::InvalidLine { lineno, detail } => {
                write!(f, "WHEEL: line {lineno}: {detail}")
            }
            WheelMetadataError::MissingField(field) => {
                write!(f, "WHEEL: missing required field {field}")
            }
            WheelMetadataError::InvalidBool { field, value } => {
                write!(f, "WHEEL: field {field} expected true/false, got {value:?}")
            }
            WheelMetadataError::InvalidVersion(v) => {
                write!(f, "WHEEL: Wheel-Version {v:?} is not <major>.<minor>")
            }
            WheelMetadataError::NoTags => write!(f, "WHEEL: at least one Tag is required"),
        }
    }
}

impl std::error::Error for WheelMetadataError {}

impl WheelMetadata {
    /// Parse the `<major>.<minor>` components of `wheel_version`.
    /// Installers compare `major` against the highest version they
    /// support and refuse anything higher. We surface this once so
    /// callers don't need to re-parse.
    pub fn version_tuple(&self) -> Result<(u32, u32), WheelMetadataError> {
        let (maj, min) = self
            .wheel_version
            .split_once('.')
            .ok_or_else(|| WheelMetadataError::InvalidVersion(self.wheel_version.clone()))?;
        let major: u32 = maj
            .parse()
            .map_err(|_| WheelMetadataError::InvalidVersion(self.wheel_version.clone()))?;
        let minor: u32 = min
            .parse()
            .map_err(|_| WheelMetadataError::InvalidVersion(self.wheel_version.clone()))?;
        Ok((major, minor))
    }
}

/// Parse a `.dist-info/WHEEL` body. Lines are processed in order;
/// repeated `Tag:` lines accumulate. Trailing whitespace on each
/// value is trimmed. Blank lines are tolerated. Unknown keys are
/// silently ignored so future PEPs that add new optional fields
/// don't break older mamba builds.
pub fn parse_wheel_metadata(src: &str) -> Result<WheelMetadata, WheelMetadataError> {
    let mut wheel_version: Option<String> = None;
    let mut generator: Option<String> = None;
    let mut root_is_purelib: Option<bool> = None;
    let mut tags: Vec<String> = Vec::new();
    let mut build: Option<String> = None;

    for (lineno0, raw) in src.lines().enumerate() {
        let lineno = lineno0 + 1;
        let line = raw.trim_end();
        if line.trim().is_empty() {
            continue;
        }
        let (key, value) = line
            .split_once(':')
            .ok_or_else(|| WheelMetadataError::InvalidLine {
                lineno,
                detail: format!("missing ':' in {line:?}"),
            })?;
        let key = key.trim();
        let value = value.trim();
        if key.is_empty() {
            return Err(WheelMetadataError::InvalidLine {
                lineno,
                detail: "empty header name".into(),
            });
        }
        match key {
            "Wheel-Version" => wheel_version = Some(value.to_string()),
            "Generator" => generator = Some(value.to_string()),
            "Root-Is-Purelib" => match value.to_ascii_lowercase().as_str() {
                "true" => root_is_purelib = Some(true),
                "false" => root_is_purelib = Some(false),
                other => {
                    return Err(WheelMetadataError::InvalidBool {
                        field: "Root-Is-Purelib".into(),
                        value: other.to_string(),
                    });
                }
            },
            "Tag" => tags.push(value.to_string()),
            "Build" => build = Some(value.to_string()),
            _ => {} // forward-compat: unknown fields ignored
        }
    }

    let wheel_version =
        wheel_version.ok_or_else(|| WheelMetadataError::MissingField("Wheel-Version".into()))?;
    let generator =
        generator.ok_or_else(|| WheelMetadataError::MissingField("Generator".into()))?;
    let root_is_purelib = root_is_purelib
        .ok_or_else(|| WheelMetadataError::MissingField("Root-Is-Purelib".into()))?;
    if tags.is_empty() {
        return Err(WheelMetadataError::NoTags);
    }

    let out = WheelMetadata {
        wheel_version,
        generator,
        root_is_purelib,
        tags,
        build,
    };
    // Validate version parses; reject malformed versions at parse time.
    out.version_tuple()?;
    Ok(out)
}

/// Render a `WheelMetadata` back into the canonical PEP 491 layout.
/// Round-trips with `parse_wheel_metadata`. Field order matches what
/// `bdist_wheel` emits: Wheel-Version, Generator, Root-Is-Purelib,
/// Tag(s), Build.
pub fn render_wheel_metadata(m: &WheelMetadata) -> String {
    let mut out = String::new();
    out.push_str("Wheel-Version: ");
    out.push_str(&m.wheel_version);
    out.push('\n');
    out.push_str("Generator: ");
    out.push_str(&m.generator);
    out.push('\n');
    out.push_str("Root-Is-Purelib: ");
    out.push_str(if m.root_is_purelib { "true" } else { "false" });
    out.push('\n');
    for tag in &m.tags {
        out.push_str("Tag: ");
        out.push_str(tag);
        out.push('\n');
    }
    if let Some(build) = &m.build {
        out.push_str("Build: ");
        out.push_str(build);
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal() -> WheelMetadata {
        WheelMetadata {
            wheel_version: "1.0".into(),
            generator: "bdist_wheel (0.40.0)".into(),
            root_is_purelib: true,
            tags: vec!["py3-none-any".into()],
            build: None,
        }
    }

    #[test]
    fn parses_minimal_wheel_file() {
        let src = "Wheel-Version: 1.0\nGenerator: bdist_wheel (0.40.0)\n\
                   Root-Is-Purelib: true\nTag: py3-none-any\n";
        let m = parse_wheel_metadata(src).unwrap();
        assert_eq!(m, minimal());
    }

    #[test]
    fn parses_with_build_number() {
        let src = "Wheel-Version: 1.0\nGenerator: bdist_wheel (0.40.0)\n\
                   Root-Is-Purelib: false\nTag: cp310-cp310-linux_x86_64\nBuild: 1\n";
        let m = parse_wheel_metadata(src).unwrap();
        assert!(!m.root_is_purelib);
        assert_eq!(m.build.as_deref(), Some("1"));
        assert_eq!(m.tags, vec!["cp310-cp310-linux_x86_64"]);
    }

    #[test]
    fn parses_multiple_tags_in_order() {
        let src = "Wheel-Version: 1.0\nGenerator: g\nRoot-Is-Purelib: true\n\
                   Tag: py3-none-any\nTag: py2-none-any\nTag: cp39-cp39-linux_x86_64\n";
        let m = parse_wheel_metadata(src).unwrap();
        assert_eq!(
            m.tags,
            vec!["py3-none-any", "py2-none-any", "cp39-cp39-linux_x86_64"]
        );
    }

    #[test]
    fn accepts_capitalized_bool_value_from_older_bdist_wheel() {
        let src = "Wheel-Version: 1.0\nGenerator: old\n\
                   Root-Is-Purelib: True\nTag: py3-none-any\n";
        let m = parse_wheel_metadata(src).unwrap();
        assert!(m.root_is_purelib);
    }

    #[test]
    fn unknown_keys_are_silently_ignored() {
        let src = "Wheel-Version: 1.0\nGenerator: g\nRoot-Is-Purelib: true\n\
                   Tag: py3-none-any\nFuture-Field: hello\n";
        let m = parse_wheel_metadata(src).unwrap();
        assert_eq!(m.wheel_version, "1.0");
        assert_eq!(m.generator, "g");
        assert!(m.root_is_purelib);
        assert_eq!(m.tags, vec!["py3-none-any"]);
        assert!(m.build.is_none());
    }

    #[test]
    fn blank_lines_are_tolerated() {
        let src = "Wheel-Version: 1.0\n\n\nGenerator: g\nRoot-Is-Purelib: true\n\
                   \nTag: py3-none-any\n";
        let m = parse_wheel_metadata(src).unwrap();
        assert_eq!(m.tags, vec!["py3-none-any"]);
    }

    #[test]
    fn trailing_whitespace_is_trimmed() {
        let src =
            "Wheel-Version: 1.0   \nGenerator: g\t\nRoot-Is-Purelib: true \nTag: py3-none-any\n";
        let m = parse_wheel_metadata(src).unwrap();
        assert_eq!(m.wheel_version, "1.0");
        assert_eq!(m.generator, "g");
    }

    #[test]
    fn missing_wheel_version_is_an_error() {
        let src = "Generator: g\nRoot-Is-Purelib: true\nTag: py3-none-any\n";
        let err = parse_wheel_metadata(src).unwrap_err();
        match err {
            WheelMetadataError::MissingField(f) => assert_eq!(f, "Wheel-Version"),
            other => panic!("expected MissingField(Wheel-Version), got {other:?}"),
        }
    }

    #[test]
    fn missing_generator_is_an_error() {
        let src = "Wheel-Version: 1.0\nRoot-Is-Purelib: true\nTag: py3-none-any\n";
        let err = parse_wheel_metadata(src).unwrap_err();
        match err {
            WheelMetadataError::MissingField(f) => assert_eq!(f, "Generator"),
            other => panic!("expected MissingField(Generator), got {other:?}"),
        }
    }

    #[test]
    fn missing_root_is_purelib_is_an_error() {
        let src = "Wheel-Version: 1.0\nGenerator: g\nTag: py3-none-any\n";
        let err = parse_wheel_metadata(src).unwrap_err();
        match err {
            WheelMetadataError::MissingField(f) => assert_eq!(f, "Root-Is-Purelib"),
            other => panic!("expected MissingField(Root-Is-Purelib), got {other:?}"),
        }
    }

    #[test]
    fn no_tags_is_an_error() {
        let src = "Wheel-Version: 1.0\nGenerator: g\nRoot-Is-Purelib: true\n";
        let err = parse_wheel_metadata(src).unwrap_err();
        assert!(matches!(err, WheelMetadataError::NoTags));
    }

    #[test]
    fn invalid_bool_value_is_an_error() {
        let src = "Wheel-Version: 1.0\nGenerator: g\nRoot-Is-Purelib: yes\nTag: py3-none-any\n";
        let err = parse_wheel_metadata(src).unwrap_err();
        match err {
            WheelMetadataError::InvalidBool { field, value } => {
                assert_eq!(field, "Root-Is-Purelib");
                assert_eq!(value, "yes");
            }
            other => panic!("expected InvalidBool, got {other:?}"),
        }
    }

    #[test]
    fn malformed_version_at_parse_time() {
        let src = "Wheel-Version: notaversion\nGenerator: g\n\
                   Root-Is-Purelib: true\nTag: py3-none-any\n";
        let err = parse_wheel_metadata(src).unwrap_err();
        assert!(matches!(err, WheelMetadataError::InvalidVersion(_)));
    }

    #[test]
    fn line_without_colon_is_an_error() {
        let src = "Wheel-Version 1.0\nGenerator: g\nRoot-Is-Purelib: true\nTag: py3-none-any\n";
        let err = parse_wheel_metadata(src).unwrap_err();
        match err {
            WheelMetadataError::InvalidLine { lineno, .. } => assert_eq!(lineno, 1),
            other => panic!("expected InvalidLine(1), got {other:?}"),
        }
    }

    #[test]
    fn empty_header_name_is_an_error() {
        let src =
            "Wheel-Version: 1.0\n: value\nGenerator: g\nRoot-Is-Purelib: true\nTag: py3-none-any\n";
        let err = parse_wheel_metadata(src).unwrap_err();
        match err {
            WheelMetadataError::InvalidLine { lineno, detail } => {
                assert_eq!(lineno, 2);
                assert!(detail.contains("empty"));
            }
            other => panic!("expected InvalidLine(empty), got {other:?}"),
        }
    }

    #[test]
    fn version_tuple_parses_components() {
        let m = minimal();
        assert_eq!(m.version_tuple().unwrap(), (1, 0));

        let mut m = minimal();
        m.wheel_version = "2.15".into();
        assert_eq!(m.version_tuple().unwrap(), (2, 15));
    }

    #[test]
    fn render_produces_canonical_layout() {
        let m = minimal();
        let body = render_wheel_metadata(&m);
        assert_eq!(
            body,
            "Wheel-Version: 1.0\nGenerator: bdist_wheel (0.40.0)\n\
             Root-Is-Purelib: true\nTag: py3-none-any\n"
        );
    }

    #[test]
    fn render_includes_build_when_present() {
        let mut m = minimal();
        m.build = Some("42".into());
        let body = render_wheel_metadata(&m);
        assert!(body.contains("Build: 42\n"));
    }

    #[test]
    fn render_emits_one_line_per_tag() {
        let mut m = minimal();
        m.tags = vec!["py3-none-any".into(), "cp310-cp310-linux_x86_64".into()];
        let body = render_wheel_metadata(&m);
        assert_eq!(body.matches("Tag: ").count(), 2);
        assert!(body.contains("Tag: py3-none-any"));
        assert!(body.contains("Tag: cp310-cp310-linux_x86_64"));
    }

    #[test]
    fn render_emits_false_for_platlib_wheels() {
        let mut m = minimal();
        m.root_is_purelib = false;
        let body = render_wheel_metadata(&m);
        assert!(body.contains("Root-Is-Purelib: false\n"));
    }

    #[test]
    fn round_trips_through_parse_and_render() {
        let mut m = minimal();
        m.tags = vec!["py3-none-any".into(), "py2-none-any".into()];
        m.build = Some("7".into());
        m.root_is_purelib = false;
        let body = render_wheel_metadata(&m);
        let parsed = parse_wheel_metadata(&body).unwrap();
        assert_eq!(parsed, m);
    }

    #[test]
    fn round_trips_minimal() {
        let m = minimal();
        let body = render_wheel_metadata(&m);
        let parsed = parse_wheel_metadata(&body).unwrap();
        assert_eq!(parsed, m);
    }

    #[test]
    fn display_messages_are_informative() {
        let messages = [
            WheelMetadataError::InvalidLine {
                lineno: 7,
                detail: "x".into(),
            }
            .to_string(),
            WheelMetadataError::MissingField("Generator".into()).to_string(),
            WheelMetadataError::InvalidBool {
                field: "Root-Is-Purelib".into(),
                value: "yes".into(),
            }
            .to_string(),
            WheelMetadataError::InvalidVersion("oops".into()).to_string(),
            WheelMetadataError::NoTags.to_string(),
        ];
        for msg in &messages {
            assert!(msg.starts_with("WHEEL:"), "got: {msg:?}");
        }
    }

    #[test]
    fn version_tuple_rejects_missing_dot() {
        let mut m = minimal();
        m.wheel_version = "10".into();
        assert!(matches!(
            m.version_tuple(),
            Err(WheelMetadataError::InvalidVersion(_))
        ));
    }
}
