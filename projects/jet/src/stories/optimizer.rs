// <HANDWRITE gap="missing-generator:logic:stories-dep-optimizer" tracker="standardize-gap-projects-jet-src-stories-optimizer-rs" reason="Stories-mode dependency optimizer: route heavy third-party ESM imports through a cacheable browser ESM bundle so preview iframes do not evaluate hundreds of unbundled node_modules modules per story.">
//! Stories-mode dependency optimizer.
//!
//! This is deliberately scoped to preview-time third-party dependencies. Jet's
//! normal stories server can serve every resolved `node_modules` module through
//! `/@dep`, but large UI libraries such as Ant Design create hundreds of browser
//! ESM requests and a large evaluation cost. The optimizer keeps the existing
//! `/@dep` fallback and only rewrites known-heavy bare imports when the project
//! already has an esbuild binary available locally.

use anyhow::{anyhow, Context, Result};
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process::Command;

pub const OPTIMIZED_PREFIX: &str = "/__jet_stories_optimized/";
const CACHE_VERSION: &str = "v6-esbuild-external-dayjs-shared-locale";

pub fn optimized_route_for_specifier(root: &Path, specifier: &str) -> Option<String> {
    if !is_optimizable_specifier(specifier) || find_esbuild_binary(root).is_none() {
        return None;
    }
    Some(format!("{OPTIMIZED_PREFIX}{specifier}"))
}

pub fn optimized_dep_source(root: &Path, specifier: &str) -> Result<String> {
    if !is_optimizable_specifier(specifier) {
        return Err(anyhow!(
            "specifier is not eligible for stories optimization: {specifier}"
        ));
    }
    let esbuild = find_esbuild_binary(root)
        .ok_or_else(|| anyhow!("no local esbuild binary found for stories dependency optimizer"))?;
    let cache_dir = root.join(".jet/stories-optimized");
    fs::create_dir_all(&cache_dir)
        .with_context(|| format!("failed to create {}", cache_dir.display()))?;
    let output = cache_dir.join(cache_file_name(root, specifier));
    if output.is_file() {
        return fs::read_to_string(&output)
            .with_context(|| format!("failed to read optimized dependency {}", output.display()));
    }

    let temp_suffix = optimizer_temp_suffix();
    let output_stem = output
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("optimized");
    let temp = output.with_file_name(format!("{output_stem}.{temp_suffix}.tmp.js"));
    let entry = output.with_file_name(format!("{output_stem}.{temp_suffix}.entry.tmp.js"));
    fs::write(&entry, optimizer_entry_source(specifier))
        .with_context(|| format!("failed to write optimizer entry {}", entry.display()))?;
    let status = Command::new(&esbuild)
        .current_dir(root)
        .arg(&entry)
        .arg("--bundle")
        .arg("--format=esm")
        .arg("--platform=browser")
        .arg("--target=es2020")
        .arg("--main-fields=browser,module,main")
        .arg("--conditions=browser,default")
        .arg("--charset=utf8")
        .arg("--log-level=warning")
        .arg(format!("--banner:js={}", external_require_banner()))
        .arg("--external:react")
        .arg("--external:react-dom")
        .arg("--external:react-dom/client")
        .arg("--external:react/jsx-runtime")
        .arg("--external:@storybook/*")
        .arg("--external:dayjs")
        .arg("--external:dayjs/*")
        .arg(format!("--outfile={}", temp.display()))
        .status()
        .with_context(|| format!("failed to run esbuild at {}", esbuild.display()))?;
    let _ = fs::remove_file(&entry);
    if !status.success() {
        let _ = fs::remove_file(&temp);
        return Err(anyhow!(
            "esbuild failed while optimizing stories dependency {specifier} with status {status}"
        ));
    }
    fs::rename(&temp, &output).with_context(|| {
        format!(
            "failed to commit optimized dependency {} -> {}",
            temp.display(),
            output.display()
        )
    })?;
    fs::read_to_string(&output)
        .with_context(|| format!("failed to read optimized dependency {}", output.display()))
}

fn optimizer_temp_suffix() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    format!("{}-{nanos}", std::process::id())
}

fn optimizer_entry_source(specifier: &str) -> String {
    let quoted = format!("{specifier:?}");
    format!(
        "import * as mod from {quoted};\nexport default (mod.default ?? mod);\nexport * from {quoted};\n"
    )
}

fn external_require_banner() -> &'static str {
    r#"import * as __jetReact from "react";
import * as __jetReactDom from "react-dom";
import * as __jetReactDomClient from "react-dom/client";
import * as __jetReactJsxRuntime from "react/jsx-runtime";
import __jetDayjsDefault, * as __jetDayjsNamespace from "dayjs";
var __jetDayjs = Object.assign(__jetDayjsDefault, __jetDayjsNamespace);
var require = (id) => {
  if (id === "react") return __jetReact;
  if (id === "react-dom") return __jetReactDom;
  if (id === "react-dom/client") return __jetReactDomClient;
  if (id === "react/jsx-runtime") return __jetReactJsxRuntime;
  if (id === "dayjs") return __jetDayjs;
  throw new Error(`Dynamic require of ${id} is not supported`);
};
"#
}

fn is_optimizable_specifier(specifier: &str) -> bool {
    if specifier.starts_with('.') || specifier.starts_with('/') || specifier.contains("..") {
        return false;
    }
    if matches!(
        Path::new(specifier)
            .extension()
            .and_then(|ext| ext.to_str()),
        Some("css" | "scss" | "sass")
    ) {
        return false;
    }
    let package = package_name(specifier);
    if package.starts_with("@tw-tech/") || package.starts_with("@storybook/") {
        return false;
    }
    package == "antd"
        || package == "@ant-design/icons"
        || package == "@ant-design/compatible"
        || package == "lodash"
        || package == "lodash-es"
        || package == "react-big-calendar"
        || package.starts_with("rc-")
        || package.starts_with("@rc-component/")
}

fn package_name(specifier: &str) -> String {
    if specifier.starts_with('@') {
        let mut parts = specifier.split('/');
        let scope = parts.next().unwrap_or_default();
        let name = parts.next().unwrap_or_default();
        return format!("{scope}/{name}");
    }
    specifier.split('/').next().unwrap_or(specifier).to_string()
}

fn find_esbuild_binary(root: &Path) -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("ESBUILD_BINARY_PATH").map(PathBuf::from) {
        if path.is_file() {
            return Some(path);
        }
    }
    for candidate in [
        root.join("node_modules/.pnpm/node_modules/.bin/esbuild"),
        root.join("node_modules/.bin/esbuild"),
    ] {
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

fn cache_file_name(root: &Path, specifier: &str) -> String {
    format!(
        "{}-{}.js",
        sanitize_specifier(specifier),
        cache_hash(root, specifier)
    )
}

fn sanitize_specifier(specifier: &str) -> String {
    specifier
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '_' })
        .collect::<String>()
        .trim_matches('_')
        .to_string()
}

fn cache_hash(root: &Path, specifier: &str) -> String {
    let mut hasher = DefaultHasher::new();
    CACHE_VERSION.hash(&mut hasher);
    specifier.hash(&mut hasher);
    for file in [
        "package.json",
        "pnpm-lock.yaml",
        "package-lock.json",
        "yarn.lock",
    ] {
        let path = root.join(file);
        path.to_string_lossy().hash(&mut hasher);
        if let Ok(meta) = fs::metadata(&path) {
            meta.len().hash(&mut hasher);
            if let Ok(modified) = meta.modified() {
                if let Ok(duration) = modified.duration_since(std::time::UNIX_EPOCH) {
                    duration.as_secs().hash(&mut hasher);
                    duration.subsec_nanos().hash(&mut hasher);
                }
            }
        }
    }
    format!("{:016x}", hasher.finish())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn optimizer_routes_only_known_heavy_packages() {
        assert!(is_optimizable_specifier("antd"));
        assert!(is_optimizable_specifier("antd/lib/locale/en_US"));
        assert!(is_optimizable_specifier("@ant-design/icons"));
        assert!(is_optimizable_specifier("lodash/isFunction"));
        assert!(is_optimizable_specifier("react-big-calendar"));
        assert!(!is_optimizable_specifier(
            "react-big-calendar/lib/css/react-big-calendar.css"
        ));
        assert!(!is_optimizable_specifier("@tw-tech/shared-ui-general"));
        assert!(!is_optimizable_specifier("react"));
        assert!(!is_optimizable_specifier("../local"));
    }

    #[test]
    fn external_require_banner_maps_react_externals() {
        let banner = external_require_banner();
        assert!(banner.contains("import * as __jetReact from \"react\""));
        assert!(banner.contains("id === \"react\""));
        assert!(banner.contains("id === \"react-dom/client\""));
        assert!(banner.contains("id === \"dayjs\""));
    }

    #[test]
    fn optimizer_entry_source_preserves_default_and_named_exports() {
        let entry = optimizer_entry_source("react-big-calendar/lib/addons/dragAndDrop");
        assert!(
            entry.contains("import * as mod from \"react-big-calendar/lib/addons/dragAndDrop\"")
        );
        assert!(entry.contains("export default (mod.default ?? mod)"));
        assert!(entry.contains("export * from \"react-big-calendar/lib/addons/dragAndDrop\""));
    }
}
