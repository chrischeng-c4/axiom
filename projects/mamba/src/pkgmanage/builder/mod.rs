// pkgmanage::builder — orchestrates `mamba build` and native-module force-link.
//
// Two responsibilities, kept side by side because they share the same gate:
//   - build_cmd : the `mamba build` subcommand body (clap → CompilerSession → cc)
//   - force_link: `use <kit> as _;` chain + startup self-check that the
//                 distributed_slice picked up every expected MambaModule.

pub mod build_cmd;
pub mod force_link;

pub use build_cmd::cmd_build;
pub use force_link::{assert_all_registered, EXPECTED_KITS};
