# User-model/import dogfood record

Scope: only `artifact:user-model/import`. The preserved observable contract is
that `user_model.model` imports successfully and exposes an instantiable
`User` class before and after native generation.

Rollback: remove generated `pyproject.toml`, `src/`, and `tests/`, restore the
fixture from git, and continue using `tech-design/src` as the reference model.
The independently authored `external-contracts/` tree is not generated and is
therefore retained across rollback.

Measured by `python_artifact_dogfood`: EC check/review/lock, TD compile/lock,
TD-stage EC, target generation, native unit inventory, CB-stage EC, close, and
project rollup run in one bounded test with an explicit hop budget.

Authoring friction found during migration:

- The fixture path promised by #2307 did not exist and had to be materialized.
- TD-stage EC originally emitted an incomplete native-generation command.
- Python TD check did not bind a TD lock before terminal code-check.
- Marker-free Python CB fill/check incorrectly entered the legacy Markdown
  phase machine.
- CB-stage success originally routed back to the same goal instead of close.

No new framework was introduced; the fixture uses the public CLI and existing
Python artifact protocol adapters.
