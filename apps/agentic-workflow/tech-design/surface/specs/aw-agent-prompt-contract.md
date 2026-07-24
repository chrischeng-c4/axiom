---
id: aw-agent-prompt-contract
summary: Define the closed aw.prompt.v1 vocabulary and ASCII symbolic grammar used to project lifecycle state into concise agent instructions.
fill_sections: [changes]
capability_refs:
  - id: aw-core-client-model-workitem-first-artifact-lifecycle
    role: primary
    gap: typed-agent-prompt-contract
    claim: typed-agent-prompt-contract
    coverage: full
    rationale: "The agent-first CLI needs one deterministic prompt language whose terms and symbols preserve the workflow engine's lifecycle distinctions without becoming another state machine."
---

# AW Agent Prompt Contract

## Vocabulary Schema
<!-- type: doc lang: markdown -->

```python
from dataclasses import dataclass
from typing import Literal

Truth = Literal["unknown", "red", "green"]
TerminalLevel = Literal["stage", "change", "root"]
BlockerKind = Literal[
    "decision",
    "approval",
    "environment",
    "red_gate",
    "missing_evidence",
]

OPERATORS = ("->", "--gate->", ":=", "==", "!=", "in", "notin")


@dataclass(frozen=True)
class PromptContract:
    state: str
    artifact: str
    writable: tuple[str, ...]
    readonly: tuple[str, ...]
    transition: str
    verifier: str
    terminal: str
    guards: tuple[str, ...]
    blocker: BlockerKind | None
    resume: str | None
```

`aw.prompt.v1` is a projection schema. The AW workflow engine remains the sole
owner of state, transition selection, mutation, and completion. The prompt
contract never evaluates expressions, invokes Python, or authorizes a command
that is absent from the envelope's existing `next.command` /
`invoke.command`.

The additive `aw.cli.v1` projection is:

```json
{
  "agent_prompt": "state := td.authored\n...",
  "prompt_contract": {
    "schema_version": "aw.prompt.v1",
    "state": "td.authored",
    "artifact": {"kind": "td", "id": "#2440"},
    "scope": {
      "writable": ["tech-design/2440"],
      "readonly": ["external-contracts/2440"]
    },
    "transition": {
      "command": "aw ec verify --stage td --wi 2440",
      "next_state": "ec_td_verifying"
    },
    "verifier": {
      "command": "aw ec verify --stage td --wi 2440",
      "predicate": "EC[TD].behavior == green"
    },
    "terminal": {
      "level": "root",
      "predicate": "completion.workflow_complete == true"
    },
    "guards": ["action == done != completion.workflow_complete"],
    "resume_command": null
  }
}
```

`prompt_contract` is additive. Existing clients may ignore it and continue to
read `agent_prompt`, `next`, `completion`, payload, and HITL fields. Both
representations are rendered from the same Rust IR.

## Symbolic Logic
<!-- type: doc lang: markdown -->

```python
def python_spec_pipeline() -> tuple[str, ...]:
    return (
        "EC := unknown",
        "EC -> TD",
        "TD --gate-> EC[TD].behavior == green",
        "TD --gate-> EC[TD].security == green",
        "EC[TD] -> CB",
        "CB --gate-> EC[CB].behavior == green",
        "CB --gate-> EC[CB].security == green",
        "CB --gate-> EC[CB].stability == green",
        "CB --gate-> EC[CB].efficiency in {green, not-applicable}",
        "completion.workflow_complete == true",
    )
```

The vocabulary is closed:

- `unknown` means the verifier has not produced valid evidence; it is not red
  and never counts as green.
- `red` means a verifier ran and rejected its target with valid failure
  evidence.
- `green` means a verifier ran and accepted its target with valid evidence.
- `stage terminal` means the current child stage is done.
- `change closed` means the issue-platform change is closed with required
  evidence.
- `root complete` means and only means
  `completion.workflow_complete == true`.
- `owner` names the artifact allowed to change: invalid EC/oracle/evidence is
  owned by EC; a valid EC red result is owned by TD or CB according to target.
- `blocker` is one of `decision`, `approval`, `environment`, `red_gate`, or
  `missing_evidence`.

The symbolic grammar is deliberately small:

- `A -> B`: the workflow selects B after A.
- `A --gate-> V == green`: verifier V must be green before transition.
- `x := value`: bind a projection-local name.
- `==` and `!=`: equality predicates.
- `in` and `notin`: finite membership predicates.

No Unicode lookalikes or additional operators are canonical. Natural-language
instructions may explain the symbols, but they may not add lifecycle meaning.

For Python Spec projects, EC is authored first and is already executable
Python. TD is executable Python that describes the candidate design and its
unit tests. CB lives under `src/*`, is grouped by domain, and includes unit
tests. TD-stage verification requires behavior and security. Stability and
efficiency are added after TD generation and are required at CB-stage
verification according to target policy; Rust defaults efficiency to required.

## Production Projection
<!-- type: doc lang: markdown -->

Every production `WorkflowEnvelope` is projected through
`workflow_prompt_contract` during serialization. Invalid contracts fail
serialization; there is no fallback to incomplete hand-authored prompt prose.
The existing prose is retained only as `guidance` inside the typed IR and is
therefore rendered from the same source as the symbolic contract.

| command/state | owner | verifier |
|---|---|---|
| `aw ec check` | `external-contracts/**` | `EC.structure == green` |
| `aw ec review` | `external-contracts/**` | `EC.review == accepted` |
| `aw td check` | `tech-design/**` | `TD.compile == green` |
| `aw ec verify --stage td` | read-only EC + TD | behavior and security green |
| `aw cb gen` / `aw cb fill` | `src/**` | generated / HANDWRITE resolved |
| `aw cb check` | read-only CB | `CB.unit == green` |
| `aw ec verify --stage cb` | read-only EC + CB | all applicable dimensions |
| `aw wi close` | issue-platform change | `change.closed == true` |

Invalid oracle or stale evidence always routes to writable EC scope, never TD
or CB. Artifact-quality preflight IDs become typed guards. Rollup distinguishes
child dispatch, parked work, change closure, and root terminal state.

The conformance boundary requires every non-terminal contract to contain a
state, artifact, disjoint scope, runnable transition, verifier, terminal
condition, and guards. HITL additionally requires one typed blocker and the
exact resume command.

## Contract Examples
<!-- type: doc lang: markdown -->

```python
def test_prompt_language_contract() -> None:
    assert set(OPERATORS) == {
        "->", "--gate->", ":=", "==", "!=", "in", "notin"
    }
    assert "unknown" != "red"
    assert "stage terminal" != "root complete"
    assert "action == done" != "completion.workflow_complete == true"
```

The source tests also require `aw llm --topic prompt --format md|json` to
contain every canonical term/operator, the EC-first Python Spec pipeline, and
the authority boundary. Existing `model`, `td`, `ec`, `wi`, and `goal` topics
must not teach removed YAML/Mermaid-first or generated-EC semantics.

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/agentic-workflow/src/cli/llm.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Expose the prompt topic and align the existing orientation topics with the staged Python Spec lifecycle."
  - path: apps/agentic-workflow/tech-design/logic/aw-llm-offline-agent-orientation-command.md
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Register prompt as an offline deterministic orientation topic."
```
