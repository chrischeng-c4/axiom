// wheel_filename.rs — PEP 427 wheel filename grammar.
//
// A wheel filename is:
//
//     {distribution}-{version}(-{build tag})?-{python tag}-{abi tag}-{platform tag}.whl
//
// Each segment is `-`-free in canonical form (PEP 491 §"Escaping and
// Unicode": runs of `-` in the distribution or version are replaced
// with `_` before being interpolated into the filename). So a `.whl`
// always has either five `-`-separated segments (no build) or six
// (with build); anything else is malformed.
//
// PEP 427 also requires:
//   * `.whl` extension (lowercase).
//   * Build tag, if present, MUST start with an ASCII digit. That's
//     how parsers distinguish a 6-segment name with build tag from a
//     hypothetical 6-segment name without one (the wheel spec
//     reserves all-leading-digit forms for build tags).
//   * Each tag may itself be a `.`-joined compressed tag set (e.g.
//     `py2.py3`, `cp312.cp313`, `manylinux1_x86_64.manylinux2014_x86_64`).
//
// uv canonicalises every wheel URL through a parser like this. mamba's
// download / cache / install paths now share the same shape.

use crate::pkgmanage::pkgmgr::types::IndexError;

/// One decomposed wheel filename.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WheelFilename {
    /// PEP 491-escaped distribution name (`-` already replaced with
    /// `_` if it appeared in the source name).
    pub distribution: String,
    /// PEP 491-escaped version string.
    pub version: String,
    /// Build tag — digit-prefixed string. `None` for the common
    /// 5-segment form.
    pub build_tag: Option<String>,
    /// Compressed python-tag set, dot-joined (e.g. `cp312`,
    /// `py2.py3`).
    pub python_tag: String,
    /// Compressed ABI-tag set.
    pub abi_tag: String,
    /// Compressed platform-tag set.
    pub platform_tag: String,
}

impl WheelFilename {
    /// Iterate the python / abi / platform sub-tags expanded into
    /// `(py, abi, plat)` triples — one tag per Cartesian product
    /// entry. Matches PEP 425's "compatibility tag" expansion.
    pub fn expand_tags(&self) -> Vec<(String, String, String)> {
        let pys: Vec<&str> = self.python_tag.split('.').collect();
        let abis: Vec<&str> = self.abi_tag.split('.').collect();
        let plats: Vec<&str> = self.platform_tag.split('.').collect();
        let mut out = Vec::with_capacity(pys.len() * abis.len() * plats.len());
        for p in &pys {
            for a in &abis {
                for pl in &plats {
                    out.push(((*p).to_string(), (*a).to_string(), (*pl).to_string()));
                }
            }
        }
        out
    }

    /// Render back to the canonical filename form.
    pub fn render(&self) -> String {
        match &self.build_tag {
            Some(b) => format!(
                "{}-{}-{}-{}-{}-{}.whl",
                self.distribution,
                self.version,
                b,
                self.python_tag,
                self.abi_tag,
                self.platform_tag
            ),
            None => format!(
                "{}-{}-{}-{}-{}.whl",
                self.distribution, self.version, self.python_tag, self.abi_tag, self.platform_tag
            ),
        }
    }
}

/// Parse one wheel filename. Accepts the full `.whl` form only — to
/// parse the unsuffixed RECORD-style tag triple, see the `tags`
/// module.
pub fn parse_wheel_filename(filename: &str) -> Result<WheelFilename, IndexError> {
    let stem = match filename.strip_suffix(".whl") {
        Some(s) => s,
        None => return Err(pe(filename, "missing '.whl' extension")),
    };
    if stem.is_empty() {
        return Err(pe(filename, "wheel filename is empty before extension"));
    }
    let parts: Vec<&str> = stem.split('-').collect();
    let (distribution, version, build_tag, python_tag, abi_tag, platform_tag) = match parts.len() {
        5 => (
            parts[0].to_string(),
            parts[1].to_string(),
            None,
            parts[2].to_string(),
            parts[3].to_string(),
            parts[4].to_string(),
        ),
        6 => {
            let build = parts[2];
            if !build_tag_starts_with_digit(build) {
                return Err(pe(
                    filename,
                    "6-segment wheel name requires a digit-prefixed build tag in position 3",
                ));
            }
            (
                parts[0].to_string(),
                parts[1].to_string(),
                Some(build.to_string()),
                parts[3].to_string(),
                parts[4].to_string(),
                parts[5].to_string(),
            )
        }
        n => {
            return Err(pe(
                filename,
                &format!("expected 5 or 6 '-'-separated segments, got {n}"),
            ));
        }
    };
    // Every segment must be non-empty — PEP 491 leaves no room for
    // accidental `--` runs in the canonical filename.
    for (label, value) in [
        ("distribution", distribution.as_str()),
        ("version", version.as_str()),
        ("python-tag", python_tag.as_str()),
        ("abi-tag", abi_tag.as_str()),
        ("platform-tag", platform_tag.as_str()),
    ] {
        if value.is_empty() {
            return Err(pe(filename, &format!("empty {label} segment")));
        }
    }
    Ok(WheelFilename {
        distribution,
        version,
        build_tag,
        python_tag,
        abi_tag,
        platform_tag,
    })
}

fn build_tag_starts_with_digit(s: &str) -> bool {
    matches!(s.as_bytes().first(), Some(b) if b.is_ascii_digit())
}

fn pe(url: &str, detail: &str) -> IndexError {
    IndexError::ParseError {
        url: url.to_string(),
        detail: detail.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_minimal_five_segment_wheel() {
        let f = parse_wheel_filename("requests-2.31.0-py3-none-any.whl").unwrap();
        assert_eq!(f.distribution, "requests");
        assert_eq!(f.version, "2.31.0");
        assert_eq!(f.build_tag, None);
        assert_eq!(f.python_tag, "py3");
        assert_eq!(f.abi_tag, "none");
        assert_eq!(f.platform_tag, "any");
    }

    #[test]
    fn parses_six_segment_wheel_with_build_tag() {
        let f = parse_wheel_filename("foo-1.0-1-py3-none-any.whl").unwrap();
        assert_eq!(f.distribution, "foo");
        assert_eq!(f.version, "1.0");
        assert_eq!(f.build_tag.as_deref(), Some("1"));
    }

    #[test]
    fn parses_build_tag_with_suffix() {
        // pip and uv emit `0pre1`, `42dev0`, etc.
        let f = parse_wheel_filename("foo-1.0-0pre1-py3-none-any.whl").unwrap();
        assert_eq!(f.build_tag.as_deref(), Some("0pre1"));
    }

    #[test]
    fn parses_compressed_tag_sets() {
        let f = parse_wheel_filename("foo-1.0-py2.py3-none-any.whl").unwrap();
        assert_eq!(f.python_tag, "py2.py3");
        let f = parse_wheel_filename(
            "numpy-1.26.0-cp312-cp312-manylinux1_x86_64.manylinux2014_x86_64.whl",
        )
        .unwrap();
        assert_eq!(f.platform_tag, "manylinux1_x86_64.manylinux2014_x86_64");
    }

    #[test]
    fn parses_pep491_escaped_distribution() {
        // PEP 491 §"Escaping and Unicode": runs of `-` become `_` in
        // the filename. `zope-interface` → `zope_interface`.
        let f = parse_wheel_filename("zope_interface-5.5.2-cp312-cp312-linux_x86_64.whl").unwrap();
        assert_eq!(f.distribution, "zope_interface");
    }

    #[test]
    fn parses_version_with_local_segment() {
        // PEP 440 local versions use `+`, which is illegal in the
        // filename — pip replaces with `_` per PEP 491. We just pass
        // the escaped string through.
        let f = parse_wheel_filename("foo-1.0+cuda_117-py3-none-any.whl").unwrap();
        assert_eq!(f.version, "1.0+cuda_117");
    }

    #[test]
    fn rejects_missing_whl_extension() {
        let err = parse_wheel_filename("foo-1.0-py3-none-any.tar.gz").unwrap_err();
        assert!(err.to_string().contains("'.whl' extension"));
    }

    #[test]
    fn rejects_empty_filename() {
        let err = parse_wheel_filename(".whl").unwrap_err();
        assert!(err.to_string().contains("empty before extension"));
    }

    #[test]
    fn rejects_too_few_segments() {
        let err = parse_wheel_filename("foo-1.0-py3-none.whl").unwrap_err();
        assert!(err.to_string().contains("5 or 6"));
        assert!(err.to_string().contains("got 4"));
    }

    #[test]
    fn rejects_too_many_segments() {
        let err = parse_wheel_filename("foo-1.0-1-2-py3-none-any.whl").unwrap_err();
        assert!(err.to_string().contains("5 or 6"));
        assert!(err.to_string().contains("got 7"));
    }

    #[test]
    fn rejects_six_segment_without_digit_build_tag() {
        // `local` doesn't start with a digit, so this is a malformed
        // 6-segment name (probably PEP 491 escaping in distribution
        // gone wrong).
        let err = parse_wheel_filename("foo-1.0-local-py3-none-any.whl").unwrap_err();
        assert!(err.to_string().contains("digit-prefixed build tag"));
    }

    #[test]
    fn rejects_empty_segment() {
        let err = parse_wheel_filename("foo--py3-none-any.whl").unwrap_err();
        // Empty version segment after the double `-`.
        assert!(err.to_string().contains("empty version segment"));
    }

    #[test]
    fn expand_tags_cartesian_product() {
        let f = parse_wheel_filename("foo-1.0-py2.py3-none-any.whl").unwrap();
        let combos = f.expand_tags();
        assert_eq!(
            combos,
            vec![
                ("py2".into(), "none".into(), "any".into()),
                ("py3".into(), "none".into(), "any".into()),
            ]
        );
    }

    #[test]
    fn expand_tags_multi_platform() {
        let f =
            parse_wheel_filename("x-1.0-cp312-cp312-manylinux1_x86_64.manylinux2014_x86_64.whl")
                .unwrap();
        let combos = f.expand_tags();
        assert_eq!(combos.len(), 2);
        assert!(combos.contains(&("cp312".into(), "cp312".into(), "manylinux1_x86_64".into())));
        assert!(combos.contains(&(
            "cp312".into(),
            "cp312".into(),
            "manylinux2014_x86_64".into()
        )));
    }

    #[test]
    fn render_round_trips_five_segment() {
        let s = "requests-2.31.0-py3-none-any.whl";
        let f = parse_wheel_filename(s).unwrap();
        assert_eq!(f.render(), s);
    }

    #[test]
    fn render_round_trips_six_segment() {
        let s = "foo-1.0-1pre-py3-none-any.whl";
        let f = parse_wheel_filename(s).unwrap();
        assert_eq!(f.render(), s);
    }

    #[test]
    fn render_round_trips_compressed_tags() {
        let s = "numpy-1.26.0-cp312.cp313-cp312.cp313-manylinux1_x86_64.manylinux2014_x86_64.whl";
        let f = parse_wheel_filename(s).unwrap();
        assert_eq!(f.render(), s);
    }

    #[test]
    fn cpython_abi3_wheel() {
        let f = parse_wheel_filename("cryptography-42.0.5-cp37-abi3-manylinux_2_28_x86_64.whl")
            .unwrap();
        assert_eq!(f.distribution, "cryptography");
        assert_eq!(f.python_tag, "cp37");
        assert_eq!(f.abi_tag, "abi3");
        assert_eq!(f.platform_tag, "manylinux_2_28_x86_64");
    }

    #[test]
    fn musllinux_wheel() {
        let f = parse_wheel_filename("cffi-1.16.0-cp312-cp312-musllinux_1_1_aarch64.whl").unwrap();
        assert_eq!(f.platform_tag, "musllinux_1_1_aarch64");
    }

    #[test]
    fn macos_universal_wheel() {
        let f = parse_wheel_filename("pillow-10.2.0-cp312-cp312-macosx_11_0_arm64.whl").unwrap();
        assert_eq!(f.platform_tag, "macosx_11_0_arm64");
    }

    #[test]
    fn windows_wheel() {
        let f = parse_wheel_filename("lxml-4.9.4-cp312-cp312-win_amd64.whl").unwrap();
        assert_eq!(f.platform_tag, "win_amd64");
    }

    #[test]
    fn case_sensitive_extension() {
        // PEP 427 spells the extension lowercase. `.WHL` is rejected.
        let err = parse_wheel_filename("foo-1.0-py3-none-any.WHL").unwrap_err();
        assert!(err.to_string().contains("'.whl' extension"));
    }
}
