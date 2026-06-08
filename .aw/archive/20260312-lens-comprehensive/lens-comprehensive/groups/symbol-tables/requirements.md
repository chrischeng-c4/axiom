---
change: lens-comprehensive
group: symbol-tables
date: 2026-03-12
---

# Requirements

Add symbol table builders for 5 new languages following the existing pattern in semantic/symbols/{python,typescript,rust}.rs. Each builder extracts language-specific symbols:
- JavaScript: functions, classes, variables, imports/exports
- Dockerfile: FROM stages, ENV vars, EXPOSE ports, LABEL keys, ARG declarations
- Terraform/HCL: resources, data sources, variables, outputs, locals, modules
- Kubernetes YAML: resources (name+kind+namespace), labels, selectors, service→deployment refs
- GitLab CI: jobs, stages, variables, templates, include references

Register all builders in semantic/symbols/mod.rs. Ensure hover and go-to-definition work for these languages via the daemon.
