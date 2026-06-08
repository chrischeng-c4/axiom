---
change: 1145
group: sdd-docs-phase
date: 2026-04-04
---

# Requirements

Add docs generation phase to SDD workflow: (1) Parse [sdd.docs] config with targets array from cclab/config.toml; (2) Extend state machine with DocsCheck/DocsCreated/DocsReviewed states between ChangeImplementationReviewed and ChangeMergeCreated; (3) Decision tree: config exists → crate match → run docs CRR, otherwise skip to merge; (4) Doc-writer agent creates/updates guide sections using change specs + CLI specs + config specs + scenarios as source material; (5) Doc-reviewer agent reviews for accuracy, completeness, audience fit; (6) Full CRR cycle same as implementation.
