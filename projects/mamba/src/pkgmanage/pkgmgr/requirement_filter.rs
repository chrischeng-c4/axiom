// Marker-aware Requirement filtering (Tick 123).
//
// PEP 508 markers are evaluated against a runtime environment to decide
// whether a given Requirement applies. The existing `markers::evaluate`
// function does the per-string evaluation; the typed parser layer
// (`requirement_string::Requirement`, `pep621::ProjectTable.dependencies`,
// `pep723_typed::TypedScriptMetadata.dependencies`) exposes a Vec of
// already-parsed Requirements that callers need to filter against a
// MarkerEnv.
//
// This module is the bridge: it takes either a single Requirement or a
// slice and returns the applicable subset, suppressing entries whose
// marker evaluates to false.
//
// Behavior:
//   * A Requirement with `marker: None` is unconditionally applicable.
//   * A Requirement with `marker: Some(s)` is applicable iff
//     `markers::evaluate(s, env) == Ok(true)`.
//   * Marker parse errors (`MarkerError`) are surfaced to the caller —
//     a malformed marker is *not* silently treated as "applicable" or
//     "skipped".
//
// Activated extras: callers that want to include conditional
// dependencies based on `; extra == "<name>"` markers should populate
// `env.extras` before calling. The existing `markers::evaluate`
// already routes the `extra` variable through `env.extras` so this
// module needs no extras-specific code.

use crate::pkgmanage::pkgmgr::markers::{evaluate, MarkerEnv, MarkerError};
use crate::pkgmanage::pkgmgr::requirement_string::Requirement;

/// True when `req` applies to the given environment. Requirements
/// without a marker are always applicable.
pub fn applicable(req: &Requirement, env: &MarkerEnv) -> Result<bool, MarkerError> {
    match &req.marker {
        None => Ok(true),
        Some(m) => evaluate(m, env),
    }
}

/// Return references to the applicable subset of `reqs`. Stable input
/// order is preserved. Surfaces the first marker-parse error.
pub fn filter_applicable<'a>(
    reqs: &'a [Requirement],
    env: &MarkerEnv,
) -> Result<Vec<&'a Requirement>, MarkerError> {
    let mut out = Vec::with_capacity(reqs.len());
    for r in reqs {
        if applicable(r, env)? {
            out.push(r);
        }
    }
    Ok(out)
}

/// Owned-form counterpart. Consumes `reqs`, returns the applicable
/// subset by value. Useful when the caller is going to mutate or
/// re-serialize the result.
pub fn keep_applicable(
    reqs: Vec<Requirement>,
    env: &MarkerEnv,
) -> Result<Vec<Requirement>, MarkerError> {
    let mut out = Vec::with_capacity(reqs.len());
    for r in reqs {
        if applicable(&r, env)? {
            out.push(r);
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pkgmanage::pkgmgr::markers::MarkerEnv;

    fn linux_py312() -> MarkerEnv {
        let mut env = MarkerEnv::current_host();
        env.python_version = "3.12".to_string();
        env.python_full_version = "3.12.0".to_string();
        env.sys_platform = "linux".to_string();
        env.platform_system = "Linux".to_string();
        env.os_name = "posix".to_string();
        env
    }

    fn windows_py39() -> MarkerEnv {
        let mut env = MarkerEnv::current_host();
        env.python_version = "3.9".to_string();
        env.python_full_version = "3.9.0".to_string();
        env.sys_platform = "win32".to_string();
        env.platform_system = "Windows".to_string();
        env.os_name = "nt".to_string();
        env
    }

    fn parse(line: &str) -> Requirement {
        Requirement::parse(line).unwrap_or_else(|e| panic!("bad fixture {line:?}: {e:?}"))
    }

    #[test]
    fn requirement_without_marker_is_always_applicable() {
        let r = parse("requests>=2.31");
        assert!(applicable(&r, &linux_py312()).unwrap());
        assert!(applicable(&r, &windows_py39()).unwrap());
    }

    #[test]
    fn requirement_with_python_version_marker_filters_correctly() {
        let r = parse("typer>=0.12 ; python_version >= '3.10'");
        assert!(applicable(&r, &linux_py312()).unwrap()); // 3.12 satisfies
        assert!(!applicable(&r, &windows_py39()).unwrap()); // 3.9 fails
    }

    #[test]
    fn requirement_with_sys_platform_marker_filters_correctly() {
        let r = parse(r#"colorama ; sys_platform == "win32""#);
        assert!(applicable(&r, &windows_py39()).unwrap());
        assert!(!applicable(&r, &linux_py312()).unwrap());
    }

    #[test]
    fn requirement_with_negative_marker() {
        let r = parse(r#"pytest ; sys_platform != "win32""#);
        assert!(applicable(&r, &linux_py312()).unwrap());
        assert!(!applicable(&r, &windows_py39()).unwrap());
    }

    #[test]
    fn requirement_with_and_combined_marker() {
        let r = parse("rich ; python_version >= '3.10' and sys_platform == 'linux'");
        assert!(applicable(&r, &linux_py312()).unwrap());
        assert!(!applicable(&r, &windows_py39()).unwrap());

        let mut linux_py39 = linux_py312();
        linux_py39.python_version = "3.9".to_string();
        assert!(!applicable(&r, &linux_py39).unwrap());
    }

    #[test]
    fn requirement_with_or_combined_marker() {
        let r = parse("rich ; python_version < '3.10' or sys_platform == 'linux'");
        // Linux satisfies the OR
        assert!(applicable(&r, &linux_py312()).unwrap());
        // Windows Py3.9 satisfies the OR via python_version < 3.10
        assert!(applicable(&r, &windows_py39()).unwrap());
    }

    #[test]
    fn filter_applicable_preserves_order_and_drops_inapplicable() {
        let reqs = vec![
            parse("a>=1"),
            parse(r#"b ; sys_platform == "win32""#),
            parse("c"),
            parse(r#"d ; python_version >= '3.10'"#),
            parse(r#"e ; python_version < '3.10'"#),
        ];
        let env = linux_py312();
        let kept: Vec<&str> = filter_applicable(&reqs, &env)
            .unwrap()
            .iter()
            .map(|r| r.name.as_str())
            .collect();
        // a, c, d apply on linux py3.12.
        assert_eq!(kept, vec!["a", "c", "d"]);
    }

    #[test]
    fn keep_applicable_owned_form_returns_subset() {
        let reqs = vec![
            parse(r#"keep1 ; python_version >= '3.10'"#),
            parse(r#"drop1 ; sys_platform == "win32""#),
            parse("keep2"),
        ];
        let kept = keep_applicable(reqs, &linux_py312()).unwrap();
        assert_eq!(kept.len(), 2);
        assert_eq!(kept[0].name, "keep1");
        assert_eq!(kept[1].name, "keep2");
    }

    #[test]
    fn empty_input_returns_empty_output() {
        let reqs: Vec<Requirement> = vec![];
        assert!(filter_applicable(&reqs, &linux_py312()).unwrap().is_empty());
        assert!(keep_applicable(reqs, &linux_py312()).unwrap().is_empty());
    }

    #[test]
    fn extras_marker_routes_through_env_extras() {
        // `; extra == "argon2"` is conditional on the activated extras
        // set. Only included when env.extras contains "argon2".
        let r = parse(r#"argon2-cffi>=21 ; extra == "argon2""#);
        let mut env = linux_py312();
        env.extras.clear();
        assert!(!applicable(&r, &env).unwrap());

        env.extras.insert("argon2".to_string());
        assert!(applicable(&r, &env).unwrap());
    }

    #[test]
    fn surfaces_marker_parse_errors() {
        // Manually-constructed Requirement with a clearly-broken marker.
        // (requirement_string::Requirement preserves the marker tail
        // verbatim, so this exercises the markers::evaluate error path.)
        let r = Requirement {
            name: "x".to_string(),
            raw_name: "x".to_string(),
            extras: crate::pkgmanage::pkgmgr::extras_spec::ExtrasSpec::default(),
            specifier: None,
            url: None,
            marker: Some("python_version weird_op '3.10'".to_string()),
        };
        let err = applicable(&r, &linux_py312()).unwrap_err();
        // Surface — any MarkerError variant is acceptable for the
        // "broken operator" branch.
        let msg = format!("{err:?}");
        assert!(!msg.is_empty());
    }

    #[test]
    fn realistic_pep621_dependency_table_filters_for_linux() {
        // Mirrors the shape of a real-world pyproject.toml dependencies
        // list after routing through `Requirement::parse`.
        let reqs = vec![
            parse("flask>=2"),
            parse("click>=8"),
            parse(r#"colorama ; sys_platform == "win32""#),
            parse(r#"importlib-metadata; python_version < "3.10""#),
            parse(r#"jaraco.classes ; python_version >= "3.7""#),
        ];
        let env = linux_py312();
        let kept: Vec<&str> = filter_applicable(&reqs, &env)
            .unwrap()
            .iter()
            .map(|r| r.name.as_str())
            .collect();
        // flask, click, jaraco-classes apply (colorama=win32, importlib-metadata<3.10 drop)
        assert_eq!(kept, vec!["flask", "click", "jaraco-classes"]);
    }
}
