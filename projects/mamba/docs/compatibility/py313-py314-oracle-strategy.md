# Py3.13 / Py3.14 Oracle Strategy

Issue: #1115

## Baseline

Mamba's default CPython replacement target remains Python 3.12. The current
oracle contract stays anchored on the ensured `tests/cpython/.cache/oracle-env/bin/python3`
interpreter, and promotion/readiness claims continue to mean "matches the
CPython 3.12 baseline" until this document is explicitly revised.

## Multi-Version Oracle Lanes

Python 3.13 and Python 3.14 are opt-in oracle lanes until explicitly promoted.
They are for drift detection, fixture authoring, and scoped compatibility work,
not for default replacement-readiness scoring.

- Default lane: `tests/cpython/.cache/oracle-env/bin/python3`
- Py3.13 lane: `tests/cpython/.cache/oracle-env-3.13/bin/python3`
- Py3.14 lane: `tests/cpython/.cache/oracle-env-3.14/bin/python3`
- `MAMBA_ORACLE_PYTHON` remains the override hook for targeted local checks.

The harness/runtime default must not silently switch from 3.12 to 3.13/3.14
without an explicit promotion decision and follow-up gate updates.

## Fixture Metadata Gate

Fixture records need an explicit version marker for oracle ownership. The
preferred record field is `python_version`, with values like `3.12`, `3.13`,
and `3.14`. Equivalent wording is acceptable only if the field still makes the
same contract machine-readable:

- `python_version = "3.12"` means the fixture is part of the default replacement
  baseline and must stay runnable against the 3.12 oracle.
- `python_version = "3.13"` or `python_version = "3.14"` means the fixture is an
  opt-in lane artifact until promoted.
- No 3.13/3.14-only fixture should displace or delete the 3.12 coverage row it
  extends.
- Governance/schema gates should reject version-specific additions that do not
  declare a record field for oracle lane ownership.

## PEP 594 Removed-Battery Policy

PEP 594 removals are version-gated/retired for 3.13+ lanes without deleting
3.12 coverage. If a module exists in CPython 3.12 but is removed in 3.13+,
then:

- keep the 3.12 fixture and oracle evidence intact;
- mark 3.13/3.14 absence as a version-gated expectation, not a regression;
- retire only the 3.13+ lane obligation for the removed module without deleting 3.12 coverage;
- do not rewrite history by deleting the 3.12 contract row.

This applies to removed-battery surfaces such as `asynchat`, `asyncore`, and
`smtpd`, and the same rule should be used for future removal-driven churn.

## Promotion Rules

Py3.13 and Py3.14 stay opt-in until all of the following are written down and
accepted in repo governance:

1. the replacement target changes from 3.12;
2. the fixture record field is live and linted;
3. removed-battery/version-gated cases have an explicit policy;
4. the affected conformance/oracle docs and schema gates are updated together.

Until then, readiness dashboards and replacement claims remain 3.12-scoped.

## Child Work-Item Atomization

The issue should split into child WIs by drift category rather than one mixed
upgrade blob:

- PEP 696 TypeVar defaults
- PEP 701 follow-up fixture/doc alignment
- PEP 649 lazy annotations
- PEP 749 `annotationlib`
- PEP 750 template strings
- free-threading opportunity assessment
- PEP 594 removed-battery gating/docs
- multi-version oracle lane docs and metadata gates

## Repo Evidence Pointers

- `projects/mamba/README.md` keeps the public `py3.13 / py3.14 feature candidates`
  roadmap table.
- `projects/mamba/tests/harness/cpython/conventions/FIXTURE-LAYOUT.md` documents
  today's default oracle path and 3.12 baseline assumptions.
- `projects/mamba/tests/governance/schema_gates/strict_type_accounting_gate_704.rs`
  already locks version-specific and removed-module examples that this strategy
  needs to keep consistent.
