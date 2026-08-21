// pkgmanage::source — fetch backends for `[crates.*]` entries in mamba.toml.
//
// Empty placeholder for Wave 1. Future homes for:
//   - path::resolve   (path = "..." entries — currently inlined in driver)
//   - git::fetch      (git = "..." entries — B1 vendor-at-install)
//   - registry::fetch (version = "..." entries — pulls from a PyPI-compatible index)
//
// Today, path-based entries are wired by the cargo workspace synthesised in
// `pkgmanage::builder::build_cmd`; this module exists so future fetch backends
// have an owned address before any wire is cut.
