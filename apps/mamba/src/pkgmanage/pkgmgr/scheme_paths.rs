// sysconfig-style install-scheme path layout (Tick 131).
//
// CPython's `sysconfig` module exposes a set of "install schemes"
// that map a logical category — `purelib`, `platlib`, `scripts`,
// `data`, `headers`, `include` — to a filesystem path. pip and uv
// consume these to know where to drop each kind of wheel file:
//
//   * `purelib`   — pure-Python `.py` modules
//   * `platlib`   — platform-specific extension modules
//   * `scripts`   — entry-point launchers and console scripts
//   * `data`      — `*.data/data/` payloads (arbitrary files)
//   * `headers`   — C extension headers
//   * `include`   — alternative name for headers in some schemes
//
// The four schemes that matter for install planning are:
//
//   * Posix-prefix  (`/usr` / system Python)
//   * Posix-user    (`~/.local`)
//   * Posix-venv    (the venv's own root)
//   * NT-…          (Windows equivalents — different layout)
//
// This module models a generic scheme value object and ships the
// canonical templates. The actual filesystem resolution belongs in
// `venv.rs` / the installer; this is the pure mapping table.

/// One scheme's set of paths, relative to the scheme's `base` (which
/// is either `prefix` for system/venv schemes or `userbase` for the
/// user scheme).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemePaths {
    pub purelib: String,
    pub platlib: String,
    pub scripts: String,
    pub data: String,
    pub headers: String,
    pub include: String,
}

/// Recognized scheme variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchemeKind {
    /// Posix system install (e.g. `/usr/lib/python3.12/site-packages`).
    PosixPrefix,
    /// Posix user install (e.g. `~/.local/lib/python3.12/site-packages`).
    PosixUser,
    /// Posix venv layout (within a venv's `lib/` tree).
    PosixVenv,
    /// Windows system install.
    NtPrefix,
    /// Windows user install.
    NtUser,
    /// Windows venv layout.
    NtVenv,
}

impl SchemeKind {
    /// Return the canonical template (placeholders preserved). Placeholders:
    ///   `{python_version}` — e.g. `3.12`
    ///   `{abiflags}`       — e.g. `""` on PEP 425 / `"d"` for debug builds
    ///   `{base}`           — the scheme's root (prefix or userbase)
    pub fn template(self) -> SchemePaths {
        match self {
            SchemeKind::PosixPrefix => SchemePaths {
                purelib: "{base}/lib/python{python_version}/site-packages".into(),
                platlib: "{base}/lib/python{python_version}/site-packages".into(),
                scripts: "{base}/bin".into(),
                data: "{base}".into(),
                headers: "{base}/include/python{python_version}{abiflags}/{dist_name}".into(),
                include: "{base}/include/python{python_version}{abiflags}".into(),
            },
            SchemeKind::PosixUser => SchemePaths {
                purelib: "{base}/lib/python{python_version}/site-packages".into(),
                platlib: "{base}/lib/python{python_version}/site-packages".into(),
                scripts: "{base}/bin".into(),
                data: "{base}".into(),
                headers: "{base}/include/python{python_version}{abiflags}/{dist_name}".into(),
                include: "{base}/include/python{python_version}{abiflags}".into(),
            },
            SchemeKind::PosixVenv => SchemePaths {
                purelib: "{base}/lib/python{python_version}/site-packages".into(),
                platlib: "{base}/lib/python{python_version}/site-packages".into(),
                scripts: "{base}/bin".into(),
                data: "{base}".into(),
                headers: "{base}/include/site/python{python_version}/{dist_name}".into(),
                include: "{base}/include/site/python{python_version}".into(),
            },
            SchemeKind::NtPrefix => SchemePaths {
                purelib: "{base}/Lib/site-packages".into(),
                platlib: "{base}/Lib/site-packages".into(),
                scripts: "{base}/Scripts".into(),
                data: "{base}".into(),
                headers: "{base}/Include/{dist_name}".into(),
                include: "{base}/Include".into(),
            },
            SchemeKind::NtUser => SchemePaths {
                purelib: "{base}/Python{python_version_nodot}/site-packages".into(),
                platlib: "{base}/Python{python_version_nodot}/site-packages".into(),
                scripts: "{base}/Python{python_version_nodot}/Scripts".into(),
                data: "{base}/Python{python_version_nodot}".into(),
                headers: "{base}/Python{python_version_nodot}/Include/{dist_name}".into(),
                include: "{base}/Python{python_version_nodot}/Include".into(),
            },
            SchemeKind::NtVenv => SchemePaths {
                purelib: "{base}/Lib/site-packages".into(),
                platlib: "{base}/Lib/site-packages".into(),
                scripts: "{base}/Scripts".into(),
                data: "{base}".into(),
                headers: "{base}/Include/{dist_name}".into(),
                include: "{base}/Include".into(),
            },
        }
    }
}

/// Inputs needed to resolve scheme templates to concrete paths.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemeContext {
    /// E.g. `3.12`.
    pub python_version: String,
    /// E.g. `""` (release) or `"d"` (PEP 425 debug build).
    pub abiflags: String,
    /// `{base}` value. For system/venv schemes this is the
    /// interpreter prefix; for user schemes this is the userbase.
    pub base: String,
    /// Distribution name (used by the `headers` template only).
    pub dist_name: String,
}

impl SchemeContext {
    pub fn python_version_nodot(&self) -> String {
        self.python_version.replace('.', "")
    }
}

/// Substitute the recognized placeholders inside `template`. Unknown
/// placeholders are left intact (caller can layer additional
/// substitutions on top, matching pip's behaviour).
pub fn render_path(template: &str, ctx: &SchemeContext) -> String {
    let nodot = ctx.python_version_nodot();
    template
        .replace("{python_version_nodot}", &nodot)
        .replace("{python_version}", &ctx.python_version)
        .replace("{abiflags}", &ctx.abiflags)
        .replace("{base}", &ctx.base)
        .replace("{dist_name}", &ctx.dist_name)
}

/// Convenience: render every field of a `SchemePaths` template.
pub fn render_scheme(template: &SchemePaths, ctx: &SchemeContext) -> SchemePaths {
    SchemePaths {
        purelib: render_path(&template.purelib, ctx),
        platlib: render_path(&template.platlib, ctx),
        scripts: render_path(&template.scripts, ctx),
        data: render_path(&template.data, ctx),
        headers: render_path(&template.headers, ctx),
        include: render_path(&template.include, ctx),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx_posix() -> SchemeContext {
        SchemeContext {
            python_version: "3.12".into(),
            abiflags: "".into(),
            base: "/usr".into(),
            dist_name: "foo".into(),
        }
    }

    fn ctx_user() -> SchemeContext {
        SchemeContext {
            python_version: "3.11".into(),
            abiflags: "".into(),
            base: "/home/me/.local".into(),
            dist_name: "bar".into(),
        }
    }

    fn ctx_venv() -> SchemeContext {
        SchemeContext {
            python_version: "3.12".into(),
            abiflags: "".into(),
            base: "/srv/app/.venv".into(),
            dist_name: "baz".into(),
        }
    }

    fn ctx_nt() -> SchemeContext {
        SchemeContext {
            python_version: "3.12".into(),
            abiflags: "".into(),
            base: "C:/Python312".into(),
            dist_name: "qux".into(),
        }
    }

    #[test]
    fn posix_prefix_scheme_renders_canonical_paths() {
        let rendered = render_scheme(&SchemeKind::PosixPrefix.template(), &ctx_posix());
        assert_eq!(rendered.purelib, "/usr/lib/python3.12/site-packages");
        assert_eq!(rendered.scripts, "/usr/bin");
        assert_eq!(rendered.include, "/usr/include/python3.12");
        assert_eq!(rendered.headers, "/usr/include/python3.12/foo");
    }

    #[test]
    fn posix_user_scheme_uses_user_base() {
        let rendered = render_scheme(&SchemeKind::PosixUser.template(), &ctx_user());
        assert_eq!(
            rendered.purelib,
            "/home/me/.local/lib/python3.11/site-packages"
        );
        assert_eq!(rendered.scripts, "/home/me/.local/bin");
    }

    #[test]
    fn posix_venv_scheme_uses_site_subdir_for_headers() {
        let rendered = render_scheme(&SchemeKind::PosixVenv.template(), &ctx_venv());
        assert_eq!(
            rendered.purelib,
            "/srv/app/.venv/lib/python3.12/site-packages"
        );
        assert_eq!(rendered.scripts, "/srv/app/.venv/bin");
        assert_eq!(rendered.include, "/srv/app/.venv/include/site/python3.12");
        assert_eq!(
            rendered.headers,
            "/srv/app/.venv/include/site/python3.12/baz"
        );
    }

    #[test]
    fn nt_prefix_scheme_uses_capitalized_paths() {
        let rendered = render_scheme(&SchemeKind::NtPrefix.template(), &ctx_nt());
        assert_eq!(rendered.purelib, "C:/Python312/Lib/site-packages");
        assert_eq!(rendered.scripts, "C:/Python312/Scripts");
        assert_eq!(rendered.include, "C:/Python312/Include");
    }

    #[test]
    fn nt_user_scheme_uses_python_version_nodot() {
        let rendered = render_scheme(&SchemeKind::NtUser.template(), &ctx_nt());
        assert_eq!(rendered.purelib, "C:/Python312/Python312/site-packages");
        assert_eq!(rendered.scripts, "C:/Python312/Python312/Scripts");
    }

    #[test]
    fn abiflags_substituted_into_headers() {
        let mut ctx = ctx_posix();
        ctx.abiflags = "d".into();
        let rendered = render_scheme(&SchemeKind::PosixPrefix.template(), &ctx);
        assert_eq!(rendered.include, "/usr/include/python3.12d");
    }

    #[test]
    fn unknown_placeholders_left_intact() {
        let ctx = ctx_posix();
        let raw = "{base}/extras/{unknown_key}";
        assert_eq!(render_path(raw, &ctx), "/usr/extras/{unknown_key}");
    }

    #[test]
    fn python_version_nodot_drops_separator_dots() {
        let ctx = SchemeContext {
            python_version: "3.13".into(),
            abiflags: "".into(),
            base: "/x".into(),
            dist_name: "n".into(),
        };
        assert_eq!(ctx.python_version_nodot(), "313");
    }

    #[test]
    fn purelib_and_platlib_match_on_canonical_layouts() {
        // pip + uv assume purelib == platlib for every scheme except
        // legacy distutils setups. Encode that invariant.
        for kind in [
            SchemeKind::PosixPrefix,
            SchemeKind::PosixUser,
            SchemeKind::PosixVenv,
            SchemeKind::NtPrefix,
            SchemeKind::NtUser,
            SchemeKind::NtVenv,
        ] {
            let t = kind.template();
            assert_eq!(t.purelib, t.platlib, "scheme {kind:?} purelib != platlib");
        }
    }

    #[test]
    fn dist_name_only_substitutes_into_headers() {
        let rendered = render_scheme(&SchemeKind::PosixPrefix.template(), &ctx_posix());
        // purelib / scripts / data / include don't carry {dist_name},
        // so swapping it should never affect them.
        let mut ctx2 = ctx_posix();
        ctx2.dist_name = "different-name".into();
        let rendered2 = render_scheme(&SchemeKind::PosixPrefix.template(), &ctx2);
        assert_eq!(rendered.purelib, rendered2.purelib);
        assert_eq!(rendered.scripts, rendered2.scripts);
        assert_eq!(rendered.include, rendered2.include);
        assert_ne!(rendered.headers, rendered2.headers);
    }
}
