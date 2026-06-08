---
id: semantic-vat-sandbox
summary: Semantic coverage for "projects/vat/src/sandbox"
fill_sections: [schema, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    gap: host-process-execution-and-gpu-visibility
    claim: host-process-execution-and-gpu-visibility
    coverage: full
    rationale: "This semantic TD covers vat sandbox backend selection and host-process isolation behavior for the agent-native GPU-native container capability."
---

# Semantic TD: vat/sandbox

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "vat/sandbox"
  source_group: "projects/vat/src/sandbox"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "projects/vat/src/sandbox/seatbelt.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "SeatbeltBackend"
            kind: "struct"
            public: true
          - name: "available"
            kind: "function"
            public: true
          - name: "name"
            kind: "function"
            public: false
          - name: "resolve"
            kind: "function"
            public: false
          - name: "profile_for"
            kind: "function"
            public: false
          - name: "which"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/vat/src/sandbox"
      - path: "projects/vat/src/sandbox/mod.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["service_method"]
        symbols:
          - name: "process"
            kind: "module"
            public: true
          - name: "seatbelt"
            kind: "module"
            public: true
          - name: "pick"
            kind: "function"
            public: true
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/vat/src/sandbox"
      - path: "projects/vat/src/sandbox/process.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["data_model", "service_method"]
        symbols:
          - name: "ProcessBackend"
            kind: "struct"
            public: true
          - name: "name"
            kind: "function"
            public: false
          - name: "resolve"
            kind: "function"
            public: false
        source_evidence_node:
          layer: "backend"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "projects/vat/src/sandbox"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/vat/src/sandbox/mod.rs"
    action: modify
    section: schema
    description: |
      Generate this vat Rust source unit from the aggregate TD AST source group.
    impl_mode: codegen
    replaces:
      - "<whole-file>"
    rust_source: |
      //! Pluggable isolation backends.
      //!
      //! The differentiator of vat is the state layer, not the isolation mechanism —
      //! so isolation is a trait with swappable implementations. v1 ships:
      //!
      //! - [`process::ProcessBackend`] — run the command as a plain host process
      //!   confined to the rootfs as its working directory. Zero friction, full
      //!   native GPU/IO. The default.
      //! - [`seatbelt::SeatbeltBackend`] — wrap the command in a macOS seatbelt
      //!   profile (`sandbox-exec`) that confines writes to the rootfs while leaving
      //!   the Metal GPU reachable (it's still a host process).
      //!
      //! A future Linux backend will add a namespaces + overlayfs implementation
      //! behind this same trait; the VM path (Virtualization.framework) would slot
      //! in here too — at the cost of the GPU story, which is the whole point of
      //! *not* taking that path on Apple Silicon.
      
      pub mod process;
      pub mod seatbelt;
      
      use std::path::Path;
      
      use crate::spec::{EnvSpec, Isolation};
      
      /// An isolation backend resolves the user's command into the *actual* program
      /// + argv to exec (e.g. seatbelt wraps it in `sandbox-exec`). The caller then
      /// runs that resolved command inside the vat workspace with the spec env.
      /// @spec projects/vat/tech-design/semantic/source/projects-vat-src-sandbox-mod-rs.md#source
      pub trait Sandbox {
          /// Short stable name, surfaced in events/state (`"process"`, `"seatbelt"`).
          fn name(&self) -> &'static str;
      
          /// Resolve `(program, args)` to the program + argv actually exec'd.
          /// `rootfs` is the vat's copy-on-write workspace (seatbelt scopes writes
          /// to it).
          fn resolve(&self, rootfs: &Path, program: &str, args: &[String]) -> (String, Vec<String>);
      }
      
      /// Pick a backend for a spec. Falls back to the process backend on any
      /// platform that doesn't support the requested isolation, after warning —
      /// the workspace clone still applies, so the vat is never *less* isolated than
      /// plain `cd` + run.
      /// @spec projects/vat/tech-design/semantic/source/projects-vat-src-sandbox-mod-rs.md#source
      pub fn pick(spec: &EnvSpec) -> Box<dyn Sandbox> {
          match spec.isolation {
              Isolation::None => Box::new(process::ProcessBackend),
              Isolation::Seatbelt => {
                  if cfg!(target_os = "macos") && seatbelt::available() {
                      Box::new(seatbelt::SeatbeltBackend)
                  } else {
                      eprintln!(
                          "vat: seatbelt isolation requested but unavailable on this host; \
                           using process backend (workspace is still copy-on-write)."
                      );
                      Box::new(process::ProcessBackend)
                  }
              }
          }
      }
  - path: "projects/vat/src/sandbox/process.rs"
    action: modify
    section: schema
    description: |
      Generate this vat Rust source unit from the aggregate TD AST source group.
    impl_mode: codegen
    replaces:
      - "<whole-file>"
    rust_source: |
      //! Host-process backend.
      //!
      //! The default and simplest sandbox: the command runs as an ordinary macOS (or
      //! Linux) process whose working directory is the vat's copy-on-write rootfs.
      //! There is no syscall confinement here — that is intentional. It keeps the
      //! workload fully native, which is exactly why the Apple GPU is reachable
      //! (nothing is virtualized). Disposability comes from the COW workspace:
      //! whatever the command writes lands in the rootfs and can be diffed,
      //! snapshotted, forked, or thrown away.
      
      use std::path::Path;
      
      use crate::sandbox::Sandbox;
      
      /// @spec projects/vat/tech-design/semantic/source/projects-vat-src-sandbox-process-rs.md#source
      pub struct ProcessBackend;
      
      /// @spec projects/vat/tech-design/semantic/source/projects-vat-src-sandbox-process-rs.md#source
      impl Sandbox for ProcessBackend {
          fn name(&self) -> &'static str {
              "process"
          }
      
          fn resolve(&self, _rootfs: &Path, program: &str, args: &[String]) -> (String, Vec<String>) {
              // Run the command verbatim; cwd/env are applied by the caller.
              (program.to_string(), args.to_vec())
          }
      }
  - path: "projects/vat/src/sandbox/seatbelt.rs"
    action: modify
    section: schema
    description: |
      Generate this vat Rust source unit from the aggregate TD AST source group.
    impl_mode: codegen
    replaces:
      - "<whole-file>"
    rust_source: |
      //! macOS seatbelt backend.
      //!
      //! Wraps the command in `sandbox-exec` with a generated profile that allows
      //! broad reads (so toolchains resolve) but confines **writes** to the vat's
      //! rootfs and the system temp dirs. The GPU is untouched: a seatbelt'd process
      //! is still a host process, so Metal/MPS/MLX keep working — the contrast with
      //! Docker's Linux VM holds even under isolation.
      //!
      //! `sandbox-exec` is deprecated by Apple but remains functional and is the
      //! pragmatic v1 mechanism. A future backend may move to the Endpoint Security
      //! / App Sandbox entitlement route; this trait boundary makes that swap local.
      
      use std::path::Path;
      
      use crate::sandbox::Sandbox;
      
      /// @spec projects/vat/tech-design/semantic/source/projects-vat-src-sandbox-seatbelt-rs.md#source
      pub struct SeatbeltBackend;
      
      /// Is `sandbox-exec` present on this host?
      /// @spec projects/vat/tech-design/semantic/source/projects-vat-src-sandbox-seatbelt-rs.md#source
      pub fn available() -> bool {
          which("sandbox-exec").is_some()
      }
      
      /// @spec projects/vat/tech-design/semantic/source/projects-vat-src-sandbox-seatbelt-rs.md#source
      impl Sandbox for SeatbeltBackend {
          fn name(&self) -> &'static str {
              "seatbelt"
          }
      
          fn resolve(&self, rootfs: &Path, program: &str, args: &[String]) -> (String, Vec<String>) {
              // Wrap the command in `sandbox-exec -p <profile> -- <program> <args>`.
              let profile = profile_for(rootfs);
              let mut argv = vec!["-p".to_string(), profile, program.to_string()];
              argv.extend(args.iter().cloned());
              ("sandbox-exec".to_string(), argv)
          }
      }
      
      /// Build a seatbelt profile string confining writes to `rootfs` + temp.
      fn profile_for(rootfs: &Path) -> String {
          let root = rootfs.display();
          // (allow default) then deny writes, then re-allow writes only under the
          // rootfs subtree and temp. Reads stay open so interpreters/toolchains
          // resolve their libraries.
          format!(
              "(version 1)\n\
               (allow default)\n\
               (deny file-write*)\n\
               (allow file-write* (subpath \"{root}\"))\n\
               (allow file-write* (subpath \"/private/tmp\"))\n\
               (allow file-write* (subpath \"/private/var/folders\"))\n\
               (allow file-write* (subpath \"/tmp\"))\n"
          )
      }
      
      /// Minimal PATH lookup (no extra deps).
      fn which(bin: &str) -> Option<std::path::PathBuf> {
          let path = std::env::var_os("PATH")?;
          std::env::split_paths(&path)
              .map(|dir| dir.join(bin))
              .find(|candidate| candidate.is_file())
      }
```
