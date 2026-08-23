---
name: project-readme-check
description: Validate an app or library product-document set after a core document or an adopted protocol, client, indexing, querying, GKE, client-integration, or migration guide is edited. Checks the shared structure, capability sources and gates, objective support states, roadmap outcomes, links, and cross-document consistency. Then uses one clean-context reader to test comprehension. Read-only; it does not repair documents, update tracker state, or run product gates.
---

# Project README Check

Use this skill for product documents under `apps/<name>` and `libs/<name>`.

The repository-neutral scripts own the deterministic contracts. This skill
owns the review sequence. The legacy `/aw:meta-check` answers broader
repository-rot questions. It does not define these document formats.

## Choose the validation mode

Use the full document-set mode when `STATUS.md` and `ROADMAP.md` exist, or when
the user asks for the shared product-document format. This mode validates the
three core files as one contract. It also includes conventional protocol,
generated-client, indexing, querying, GKE, and client-integration guides when
they exist. A linked `docs/migration-*.md` guide is included when the README
adopts it.

Use README-only compatibility mode when a project has not yet migrated. Report
that mode as partial document-set verification. Do not claim that STATUS or
ROADMAP was checked when either file is absent.

## Full deterministic pass

Run from the repository root:

```bash
python3 scripts/meta/project_docs_contract.py check <path> --format json
```

`<path>` can be the project directory, a core document, or an adopted
supporting guide.

The script first applies the existing README contract. It then checks:

- the fixed STATUS headings, state definitions, and Support matrix;
- stable support IDs and the exact `Supported`, `Limited`, and `Not supported`
  states;
- a resolvable executable gate for every Supported and Limited row;
- a ROADMAP outcome or non-goal for every Limited and Not supported boundary;
- the fixed ROADMAP horizons and stable outcome IDs;
- concrete outcome, boundary, completion-evidence, and tracking fields;
- the absence of self-graded progress, owner fields, schedule fields, and
  percentages;
- relative files and Markdown anchors;
- README links to both companion documents;
- README links to every adopted protocol, generated-client, indexing, querying,
  GKE, client-integration, or migration guide;
- fixed supporting-guide headings and contract, language, or compatibility
  tables;
- links and anchors inside the adopted supporting guides; and
- fully Supported surfaces that incorrectly also appear as future work.

Exit `0` means the deterministic contract passed. Exit `1` means findings.
Exit `2` means the validator CLI or target was invalid. Do not convert either
failure into a clean result.

The scripts validate gate names and paths. They do not execute product gates or
decide whether gate behavior proves a product promise.

## Full clean-reader pass

Only after the full deterministic pass succeeds, generate the review task:

```bash
python3 scripts/meta/project_docs_contract.py prompt <path>
```

Start one new subagent with no inherited conversation context. Use
`fork_turns="none"` when the runtime exposes that option. Give the subagent
exactly the printed prompt. Do not add an intended answer, prior findings, or a
summary.

The subagent must read only the exact files listed by the generated prompt.
Compare its JSON with the deterministic JSON:

- every listed SHA-256 value must match the current file;
- capability names, IDs, and source sets must match exactly;
- the support surface set, IDs, states, scopes, and material limits must agree;
- roadmap item names, IDs, and horizons must match exactly;
- every contract-map, generated-client language, and migration compatibility
  row must agree;
- current indexing and querying facts must stay separate from target behavior;
- current environment support, runtime topology, client workload, and target
  integration behavior must stay separate;
- the reader must recover the purpose, product boundary, primary workflow, and
  main functional surfaces; and
- `cross_document_contradictions` must be empty.

Accept faithful paraphrases for purpose, boundaries, workflow, scopes, and
limits. A semantic failure needs a missing required entry, a wrong ID, source,
state, or horizon, or a concrete contradiction. Do not fail only because a
detail is clearly delegated to a named maintained source.

The comprehension score is diagnostic. Never pass or fail from the number
alone. A missing detail is acceptable when the document set clearly points to
the maintained contract that owns it.

If any SHA differs, discard the review and run one new clean-reader pass against
the current bytes. If subagents are unavailable, report the deterministic pass
as partial verification. Do not invent a clean-reader verdict.

## README-only compatibility mode

Run:

```bash
python3 scripts/meta/readme_contract.py check <path> --format json
python3 scripts/meta/readme_contract.py prompt <path>
```

The README validator checks the product-first section order, functional
sections, flat capabilities, source contributions, links, and resolvable gate
names. Give its printed prompt to one clean-context subagent. Compare the
README SHA, capabilities, sources, workflow, surfaces, and
`blocking_contradictions` exactly as the prompt requests.

## Verdict

Report full `PASS` only when the full deterministic and clean-reader passes
succeed.

Report `README-ONLY PASS` when both README-only passes succeed but companion
documents are absent. This is not full document-set compliance.

Report `FAIL` with exact script findings or concrete comprehension blockers.
Keep structural defects separate from semantic findings. Never edit documents
as part of this read-only skill.
