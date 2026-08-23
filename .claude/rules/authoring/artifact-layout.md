# Right-sized artifact layout

## Intent

Make paths and filenames a reliable table of contents so agents can locate one concern with minimal reads.

## Rules

- Keep one coherent concern per file and split artifacts whose readers, reviews, or execution are independent.
- Use semantic directories as the taxonomy and explicit leaf names that identify the case or responsibility.
- Preserve cohesion when parts share setup or must evolve together.
- Use the project-local META docs for scoped rules, and the authoring chapters of the repository `CONTRIBUTING.md` for repo-wide ones.

## Verification

- Inspect the changed directory listing and confirm each path identifies its role without opening the file.
- Under `.claude/skills/aw:*/`, run `.claude/aw/verification/check_plugin.py`: it
  asserts the skill directories on disk are exactly the eight `_paths.SKILLS`
  names, so a ninth directory that nobody registered fails rather than loading
  unnoticed. Outside that tree there is no such check and the listing is the
  whole of it. Note the caller problem: nothing in this repository runs
  `.claude/aw/verification/run_all.py`, so this is a check a human runs.

## References

- `CONTRIBUTING.md` section “Authoring principle: right-sized files, semantic paths, explicit names”.
