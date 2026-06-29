// First-party package providers for mamba-owned pure-Python compatibility
// distributions.
//
// These are normal venv-installed packages, not `mambalibs`. A provider
// package has a mamba-owned distribution name and may expose one or more
// upstream-compatible import packages such as `httpx`.

use anyhow::{Result, bail};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MambaProviderPackage {
    pub(crate) distribution: String,
    pub(crate) version: String,
    pub(crate) provider: String,
    pub(crate) provides: Vec<String>,
    pub(crate) compatibility: String,
    pub(crate) maturity: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProviderFile {
    pub(crate) relative_path: String,
    pub(crate) body: String,
}

pub(crate) fn resolve_mamba_package(
    requested_name: &str,
    requested_version: Option<&str>,
) -> Result<MambaProviderPackage> {
    let requested = normalize_distribution_name(requested_name);
    let Some(pkg) = catalog()
        .into_iter()
        .find(|p| normalize_distribution_name(&p.distribution) == requested)
    else {
        if requested == "httpx" {
            bail!(
                "`httpx` is the upstream import/API name, not a mamba-owned \
                 distribution; use `mamba add --provider mamba mamba-httpx-compat`"
            );
        }
        bail!("unknown mamba provider package `{requested_name}`");
    };

    if let Some(version) = requested_version {
        if version != pkg.version {
            bail!(
                "mamba provider package `{}` has version {}, not {}",
                pkg.distribution,
                pkg.version,
                version
            );
        }
    }

    Ok(pkg)
}

pub(crate) fn locked_mamba_package(
    distribution: &str,
    version: &str,
    provider: &str,
    provides: &[String],
    compatibility: &str,
    maturity: &str,
) -> Result<MambaProviderPackage> {
    let pkg = resolve_mamba_package(distribution, Some(version))?;
    if provider != pkg.provider {
        bail!(
            "mamba provider lock metadata mismatch for `{distribution}`: \
             provider `{provider}` != `{}`",
            pkg.provider
        );
    }
    if provides != pkg.provides {
        bail!(
            "mamba provider lock metadata mismatch for `{distribution}`: \
             provides {:?} != {:?}",
            provides,
            pkg.provides
        );
    }
    if compatibility != pkg.compatibility {
        bail!(
            "mamba provider lock metadata mismatch for `{distribution}`: \
             compatibility `{compatibility}` != `{}`",
            pkg.compatibility
        );
    }
    if maturity != pkg.maturity {
        bail!(
            "mamba provider lock metadata mismatch for `{distribution}`: \
             maturity `{maturity}` != `{}`",
            pkg.maturity
        );
    }
    Ok(pkg)
}

pub(crate) fn provider_files(pkg: &MambaProviderPackage) -> Result<Vec<ProviderFile>> {
    match normalize_distribution_name(&pkg.distribution).as_str() {
        "mamba-httpx-compat" => Ok(httpx_compat_files(pkg)),
        _ => bail!(
            "mamba provider package `{}` has no file payload",
            pkg.distribution
        ),
    }
}

pub(crate) fn normalize_distribution_name(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut prev_sep = false;
    for c in name.chars() {
        let is_sep = c == '-' || c == '_' || c == '.';
        if is_sep {
            if !prev_sep && !out.is_empty() {
                out.push('-');
            }
            prev_sep = true;
        } else {
            out.push(c.to_ascii_lowercase());
            prev_sep = false;
        }
    }
    if out.ends_with('-') {
        out.pop();
    }
    out
}

fn catalog() -> Vec<MambaProviderPackage> {
    vec![MambaProviderPackage {
        distribution: "mamba-httpx-compat".to_string(),
        version: "0.1.0".to_string(),
        provider: "mamba".to_string(),
        provides: vec!["httpx".to_string()],
        compatibility: "httpx".to_string(),
        maturity: "experimental".to_string(),
    }]
}

fn httpx_compat_files(pkg: &MambaProviderPackage) -> Vec<ProviderFile> {
    let dist_module = normalize_import_name(&pkg.distribution);
    let dist_info = format!("{dist_module}-{}.dist-info", pkg.version);
    vec![
        ProviderFile {
            relative_path: format!("{dist_module}/__init__.py"),
            body: provider_module_body(pkg),
        },
        ProviderFile {
            relative_path: "httpx/__init__.py".to_string(),
            body: httpx_init_body(pkg),
        },
        ProviderFile {
            relative_path: format!("{dist_info}/METADATA"),
            body: metadata_body(pkg),
        },
        ProviderFile {
            relative_path: format!("{dist_info}/INSTALLER"),
            body: "mamba\n".to_string(),
        },
    ]
}

fn provider_module_body(pkg: &MambaProviderPackage) -> String {
    format!(
        "__mamba_provider_distribution__ = {:?}\n\
         __mamba_provider__ = {:?}\n\
         __mamba_provider_provides__ = ({:?},)\n\
         __version__ = {:?}\n",
        pkg.distribution, pkg.provider, pkg.provides[0], pkg.version
    )
}

fn httpx_init_body(pkg: &MambaProviderPackage) -> String {
    format!(
        concat!(
            "# Pure-Python compatibility surface installed by mamba.\n",
            "import json as _json\n\n",
            "__mamba_provider_distribution__ = {:?}\n",
            "__mamba_provider__ = {:?}\n",
            "__mamba_provider_maturity__ = {:?}\n",
            "__version__ = {:?}\n\n",
            "class Response:\n",
            "    def __init__(self, status_code=200, text='', headers=None, url=None):\n",
            "        self.status_code = status_code\n",
            "        self.text = text\n",
            "        self.headers = dict(headers or {{}})\n",
            "        self.url = url\n\n",
            "    @property\n",
            "    def content(self):\n",
            "        return self.text.encode('utf-8')\n\n",
            "    def json(self):\n",
            "        return _json.loads(self.text)\n\n",
            "    def raise_for_status(self):\n",
            "        if self.status_code >= 400:\n",
            "            raise HTTPStatusError(f'HTTP status {{self.status_code}}')\n\n",
            "class HTTPError(Exception):\n",
            "    pass\n\n",
            "class HTTPStatusError(HTTPError):\n",
            "    pass\n\n",
            "def request(method, url, **kwargs):\n",
            "    raise NotImplementedError(\n",
            "        'mamba-httpx-compat is installed as a pure-Python compatibility package; '\n",
            "        'network transport is not implemented yet'\n",
            "    )\n\n",
            "def get(url, **kwargs):\n",
            "    return request('GET', url, **kwargs)\n\n",
            "def post(url, **kwargs):\n",
            "    return request('POST', url, **kwargs)\n\n",
            "__all__ = ['Response', 'HTTPError', 'HTTPStatusError', 'request', 'get', 'post']\n"
        ),
        pkg.distribution, pkg.provider, pkg.maturity, pkg.version
    )
}

fn metadata_body(pkg: &MambaProviderPackage) -> String {
    format!(
        "Metadata-Version: 2.3\n\
         Name: {}\n\
         Version: {}\n\
         Summary: mamba-owned pure-Python compatibility provider for {}\n\
         X-Mamba-Provider: {}\n\
         X-Mamba-Provides-Import: {}\n\
         X-Mamba-Compatibility: {}\n\
         X-Mamba-Maturity: {}\n",
        pkg.distribution,
        pkg.version,
        pkg.compatibility,
        pkg.provider,
        pkg.provides.join(","),
        pkg.compatibility,
        pkg.maturity
    )
}

fn normalize_import_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c == '-' || c == '.' {
                '_'
            } else {
                c.to_ascii_lowercase()
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_upstream_import_name_as_distribution() {
        let err = resolve_mamba_package("httpx", None)
            .unwrap_err()
            .to_string();
        assert!(err.contains("mamba-httpx-compat"), "{err}");
    }

    #[test]
    fn httpx_compat_payload_is_pure_python() {
        let pkg = resolve_mamba_package("mamba-httpx-compat", None).unwrap();
        let files = provider_files(&pkg).unwrap();
        assert!(files.iter().any(|f| f.relative_path == "httpx/__init__.py"));
        assert!(
            files
                .iter()
                .find(|f| f.relative_path == "httpx/__init__.py")
                .unwrap()
                .body
                .contains("class Response")
        );
    }
}
