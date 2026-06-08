---
change: mamba-config-unify
group: unify-mamba-config
date: 2026-04-09
status: answered
---

# Pre-Clarifications

### Q1: Canonical struct
- **Question**: Which MambaConfig struct should be the canonical one?
- **Answer**: config/schema.rs::MambaConfig is the canonical definition. It is richer (project metadata, build config, paths, per-crate CrateConfig). The driver/config.rs::MambaConfig is the simpler legacy one that will be removed. Methods from driver (discover, is_symbol_exposed) will be migrated to the canonical struct.

### Q2: Entry point field
- **Question**: How should entry_point work in the unified struct?
- **Answer**: Add an entry_point field to ProjectConfig. The driver and main.rs use entry_point to determine which .py file to compile/run.

### Q3: Expose filtering
- **Question**: Should expose filtering change?
- **Answer**: The canonical config/schema.rs already has per-crate expose lists as Vec<String> inside CrateConfig. The is_symbol_exposed() method will be adapted to work with the CrateConfig.expose field instead of the flat HashMap<String, Vec<String>> from driver/config.rs.

