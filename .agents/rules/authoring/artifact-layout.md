---
id: authoring.artifact-layout
scope: []
activation: always
targets: [claude, codex, agy]
enforcement: advisory
required_references:
  - CONTRIBUTING.md
---
# Right-sized artifact layout

## Intent

Make paths and filenames a reliable table of contents so agents can locate one concern with minimal reads.

## Rules

- Keep one coherent concern per file and split artifacts whose readers, reviews, or execution are independent.
- Use semantic directories as the taxonomy and explicit leaf names that identify the case or responsibility.
- Preserve cohesion when parts share setup or must evolve together.
- Use the project-local META docs for scoped rules and the repository CONTRIBUTING contract for repo-wide rules.

## Verification

- Inspect the changed directory listing and confirm each path identifies its role without opening the file.
- Run the producer or linter that keeps fine-grained generated artifacts consistent.

## References

- `CONTRIBUTING.md` section “Authoring principle: right-sized files, semantic paths, explicit names”.
