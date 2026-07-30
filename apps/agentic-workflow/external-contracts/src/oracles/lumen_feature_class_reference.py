"""Lumen reference fixture for the two feature roots, plus its assertions.

Lumen is the archetype case the split exists for: domain search promises the
project exists to keep, alongside baselines every service of its archetype
carries.

Every id below is a real Lumen capability id, taken from Lumen's own capability
contract (`apps/lumen/README.md`, which is Lumen's `cap_path`; its
`CAPABILITIES.md` index is still empty pending the #1848 relocation). The four
non-core ids are genuine trait-derived baselines declared in
`src/cli/doc_mirror.rs`, so the fixture cannot name a class the derivation would
not produce.

On the two core ids: the work item's oracle wording is "Indexing and Querying".
Lumen has no `indexing` or `querying` capability; that promise is carried by
`search-core`, whose surface is literally `POST /index` + `POST /search`, and by
`lexical-search`. The fixture uses those real ids, because a fixture naming ids
Lumen does not have could not demonstrate the archetype it claims to be drawn
from.

The document is a fixture written into a temporary project. Lumen's production
contract is never opened for writing, and the caller proves that by digesting it
before and after.

Two earlier revisions of this docstring excused rules from coverage instead of
covering them, and both excuses were wrong. They are recorded here because the
reasoning was plausible each time, and the same shape is easy to reach for
again.

The first claimed a capability nested under both roots could not be falsified in
isolation. False: appending one bare `#### Search Core` heading inside the
non-core root yields exactly one blocker with all six capabilities still
parsing, because `scan_feature_roots` records root membership from headings
alone. It is now `MULTIPLY_CLASSIFIED_DOCUMENT`.

The second claimed the missing-root and unknown-root rules needed "co-occurring-
set assertions" and so belonged in their own slice. The *factual* half was
right -- deleting `### Non-Core Features` really does yield the missing-root
finding plus one field/root contradiction per stranded capability, and renaming
it really does yield unknown-root plus missing-root -- but the conclusion did
not follow. `document_blockers` already returns an ordered list, and the
falsifiers here already assert full list equality against it, so a co-occurring
set needs no machinery that is not already in use. Both are now falsified
directly as `MISSING_NON_CORE_ROOT_DOCUMENT` and `UNKNOWN_FEATURE_ROOT_DOCUMENT`.

The standing lesson: "this rule cannot be falsified by a single message" is a
statement about the *assertion shape*, never a reason to leave the rule
unbound.
"""

from __future__ import annotations

import hashlib
import re
from itertools import permutations
from pathlib import Path
from typing import Any


#: Lumen's domain search promises. `search-core` owns both the index and the
#: query surface; `lexical-search` owns BM25 ranking. Neither is a
#: trait-derived baseline, which is what makes them eligible to be core.
CORE_IDS = ("search-core", "lexical-search")
#: Trait-derived baselines Lumen carries, one per archetype family:
#: operations-observability, platform-delivery-lifecycle, security-governance,
#: and contract-quality-assurance.
NON_CORE_IDS = (
    "standard-operational-endpoints",
    "kubernetes-native-deployment",
    "security-hardening",
    "ec-gates-configured",
)
#: Claim counts are deliberately unequal per class (3 core, 5 non-core) so no
#: assertion can pass by pairing a core count with a non-core total. The gap is
#: two rather than one so that the counts stay unequal after the retired member
#: (1 claim) is excluded, which is what makes the retired document's verified
#: claim pair falsifiable against transposition as well as against inclusion.
CORE_CLAIM_COUNT = 3
NON_CORE_CLAIM_COUNT = 5

#: A real Lumen capability that is *not* a trait-derived baseline, used to
#: falsify the field/root agreement rule without also tripping the baseline
#: rule. Keeping the two falsifiers independent is the point.
CONFLICT_ID = "observability"

LUMEN_PRODUCTION_CONTRACT_PATHS = (
    "apps/lumen/CAPABILITIES.md",
    "apps/lumen/README.md",
)


def _capability(
    *,
    title: str,
    cap_id: str,
    feature_class: str | None,
    promise: str,
    surface: str,
    work_roots: tuple[str, ...],
    heading: str = "####",
    status: str = "verified",
) -> str:
    rows = "\n".join(
        f"| {root} | change | - | implemented | verified | smoke | `true` |"
        for root in work_roots
    )
    # `feature_class=None` is the pre-migration shape: no field at all. Emitting
    # an empty field instead would be a different document -- an author who
    # declared nothing, versus one who declared a blank.
    class_field = "" if feature_class is None else f"Feature Class: {feature_class}\n"
    return f"""{heading} {title}

ID: {cap_id}
Type: Service
{class_field}Surfaces:
- CLI: `{surface}` - {promise.lower().rstrip('.')}.
EC Dimensions:
- behavior: `true` - {cap_id} behavior gate.
Root WI: -
Status: {status}
Required Verification: smoke
Promise:
{promise}
Gate Inventory:
- tech-design/{cap_id}.md

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
{rows}
"""


#: (title, id, promise, surface, work roots) for each fixture member. The work
#: roots are what make the claim counts unequal across the two classes.
_CORE_MEMBERS = (
    (
        "Search Core",
        "search-core",
        "Index caller-owned records and return ranked external_ids only.",
        "lumen serve",
        ("query-planner-boolean-eval",),
    ),
    (
        "Lexical Search",
        "lexical-search",
        "Answer BM25 text queries through the analyzer-backed planner.",
        "lumen serve",
        ("bm25-ranking", "analyzer-pipeline"),
    ),
)

_NON_CORE_MEMBERS = (
    (
        "Standard Operational Endpoints",
        "standard-operational-endpoints",
        "Expose the standard health and readiness endpoints.",
        "lumen serve",
        ("operational-endpoint-set",),
    ),
    (
        "Kubernetes-Native Deployment",
        "kubernetes-native-deployment",
        "Deploy as a Kubernetes-native workload.",
        "lumen deploy",
        ("manifest-packaging",),
    ),
    (
        "Security Hardening",
        "security-hardening",
        "Enforce the archetype security baseline.",
        "lumen auth",
        ("transport-and-identity-hardening",),
    ),
    # This member's title deliberately does not slugify to its id, and it
    # deliberately carries two work roots. The title, because
    # `validate_capability_feature_roots` keys its root-membership lookup on
    # `slugify(&capability.title)` (capability.rs:10087) while every message it
    # emits names the *id* -- so a fixture whose titles always slugify to their
    # ids cannot tell the two keys apart, and an implementation that looked the
    # capability up by id would find no roots and report nothing. The second
    # work root, because it is what keeps the per-class claim counts unequal
    # once the retired member is excluded.
    (
        "Contract Gate Wiring",
        "ec-gates-configured",
        "Carry configured external-contract gates.",
        "lumen verify",
        ("gate-configuration", "gate-inventory-sync"),
    ),
)

def _slugify(value: str) -> str:
    """`capability.rs:11965`, restated so the divergence guard below is checkable.

    Not used to predict product output anywhere -- only to assert a property of
    this fixture's own inputs.
    """
    out: list[str] = []
    last_dash = False
    for ch in value:
        if ch.isascii() and ch.isalnum():
            out.append(ch.lower())
            last_dash = False
        elif not last_dash and out:
            out.append("-")
            last_dash = True
    return "".join(out).strip("-")


#: The fixture must keep at least one member whose heading title does not
#: slugify to its id, because the root-membership lookup is keyed on the slug of
#: the title while every finding names the id. Asserted rather than commented,
#: so a later rename that restores the coincidence fails here instead of
#: silently reopening the hole.
DIVERGENT_TITLE_IDS = tuple(
    member[1] for member in _CORE_MEMBERS + _NON_CORE_MEMBERS
    if _slugify(member[0]) != member[1]
)
assert DIVERGENT_TITLE_IDS, "no member's title diverges from its id"


_CONFLICT_MEMBER = (
    "Observability",
    CONFLICT_ID,
    "Emit the archetype observability signals.",
    "lumen serve",
    ("signal-set",),
)


def _section(
    member: tuple[Any, ...],
    feature_class: str | None,
    heading: str = "####",
    status: str = "verified",
) -> str:
    title, cap_id, promise, surface, work_roots = member
    return _capability(
        title=title,
        cap_id=cap_id,
        feature_class=feature_class,
        promise=promise,
        surface=surface,
        work_roots=work_roots,
        heading=heading,
        status=status,
    )


def _index_rows(members: tuple[tuple[Any, ...], ...], note: str) -> str:
    return "\n".join(
        f"| {member[0]} | - | implemented | verified | smoke | ready | verified; {note} |"
        for member in members
    )


def _document(
    core: tuple[tuple[Any, ...], ...],
    non_core: tuple[tuple[Any, ...], ...],
    *,
    core_class: str = "core",
    non_core_class: str = "non_core",
    retired_id: str | None = None,
) -> str:
    index = "\n".join(
        (
            _index_rows(core, "domain promise"),
            _index_rows(non_core, "archetype service baseline"),
        )
    )

    def _status(member: tuple[Any, ...]) -> str:
        return "retired" if member[1] == retired_id else "verified"

    core_body = "\n".join(
        _section(member, core_class, status=_status(member)) for member in core
    )
    non_core_body = "\n".join(
        _section(member, non_core_class, status=_status(member)) for member in non_core
    )
    return f"""# Lumen

## Brief

Lumen reference fixture: the domain search promises are core, the archetype
service baselines are non-core.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
{index}

### Core Features

{core_body}
### Non-Core Features

{non_core_body}"""


#: The reference document: both roots present, every class declared.
REFERENCE_DOCUMENT = _document(_CORE_MEMBERS, _NON_CORE_MEMBERS)


def _fail_claim_gate(document: str, claim: str) -> str:
    """Make one claim's gate command fail, leaving the document otherwise intact.

    The gate cell is the only thing that changes, so the claim still exists, is
    still `implemented`, and is still owned by the same capability -- it is
    unverified rather than absent, which is the state the readiness split has to
    attribute.
    """
    row = f"| {claim} | change | - | implemented | verified | smoke | "
    assert document.count(row + "`true` |") == 1, claim
    return document.replace(row + "`true` |", row + "`false` |", 1)


#: The eight operands of the `--human` readiness line, in the order it renders
#: them. Named once because the non-collision property below quantifies over
#: every pair of them.
READINESS_OPERAND_KEYS = (
    "core_verified_count",
    "core_capability_count",
    "core_verified_claim_count",
    "core_claim_count",
    "non_core_verified_count",
    "non_core_capability_count",
    "non_core_verified_claim_count",
    "non_core_claim_count",
)


def _partially_verified_document(claims: tuple[str, ...]) -> str:
    document = REFERENCE_DOCUMENT
    for claim in claims:
        document = _fail_claim_gate(document, claim)
    assert document.count("`false`") == len(claims)
    return document


#: Documents in which some claims are unverified, together with the exact
#: readiness operands each must produce.
#:
#: Every other `--verify` leg in this module reads `REFERENCE_DOCUMENT`, where
#: each class is fully verified and so `verified` and `total` are the same
#: integer in all four dimensions. On such a report an implementation that
#: rendered or accumulated a *total* where a verified count belongs is
#: indistinguishable from a correct one, which is a rule left unfalsified -- the
#: same reasoning the human leg already applied across the two classes, never
#: applied within one.
#:
#: There are two shapes rather than one because no single shape can make all
#: eight operands pairwise distinct: the four totals are fixed at 2, 3, 4 and 5
#: by the fixture's membership, and the four verified counts must then reuse
#: values from that range. Two shapes whose collisions do not overlap achieve
#: together what neither achieves alone, which the assertion below enforces
#: rather than leaves to inspection.
PARTIAL_VERIFICATION_SHAPES = (
    (
        "one-claim-in-each-class",
        ("analyzer-pipeline", "gate-inventory-sync"),
        ("lexical-search", "ec-gates-configured"),
        (1, 2, 2, 3, 3, 4, 4, 5),
    ),
    (
        "clustered-in-core-and-spread-in-non-core",
        (
            "bm25-ranking",
            "analyzer-pipeline",
            "manifest-packaging",
            "operational-endpoint-set",
        ),
        (
            "lexical-search",
            "lexical-search",
            "standard-operational-endpoints",
            "kubernetes-native-deployment",
        ),
        (1, 2, 1, 3, 2, 4, 3, 5),
    ),
)

PARTIALLY_VERIFIED_DOCUMENTS = {
    name: _partially_verified_document(claims)
    for name, claims, _failing, _operands in PARTIAL_VERIFICATION_SHAPES
}


def _colliding_operand_pairs(operands: tuple[int, ...]) -> set[frozenset[str]]:
    return {
        frozenset((left, right))
        for index, left in enumerate(READINESS_OPERAND_KEYS)
        for right in READINESS_OPERAND_KEYS[index + 1 :]
        if operands[index] == operands[READINESS_OPERAND_KEYS.index(right)]
    }


#: No operand may be zero: a zero renders identically to any other zero, so a
#: transposition confined to zeroed operands is invisible.
assert all(
    value > 0 for _n, _c, _f, operands in PARTIAL_VERIFICATION_SHAPES for value in operands
)
#: Within each shape, every verified count must fall short of its own total, or
#: the line could pair a total with itself.
assert all(
    operands[0] < operands[1]
    and operands[2] < operands[3]
    and operands[4] < operands[5]
    and operands[6] < operands[7]
    for _n, _c, _f, operands in PARTIAL_VERIFICATION_SHAPES
)
#: And no pair of operands may agree in *every* shape. A pair that agrees
#: everywhere the line is asserted is a pair the line could swap undetected --
#: not only across the two classes but across dimensions, which is how a
#: capability count could be rendered where a claim count belongs.
assert not set.intersection(
    *(
        _colliding_operand_pairs(operands)
        for _n, _c, _f, operands in PARTIAL_VERIFICATION_SHAPES
    )
), "the partial-verification shapes share a colliding operand pair"

#: Falsifier: the non-core root deleted outright. Every non-core capability is
#: then stranded under the surviving root, so the missing-root finding arrives
#: with one field/root contradiction per stranded capability. That co-occurrence
#: is why an earlier revision of this module excused the rule from coverage --
#: wrongly, since `document_blockers` returns an ordered list and the whole set
#: can simply be asserted.
MISSING_NON_CORE_ROOT_DOCUMENT = REFERENCE_DOCUMENT.replace(
    "### Non-Core Features", ""
)
assert MISSING_NON_CORE_ROOT_DOCUMENT != REFERENCE_DOCUMENT

#: Falsifier: the non-core root renamed to a heading outside the closed pair.
#: Yields unknown-root *and* missing-root, for the same reason.
UNKNOWN_FEATURE_ROOT_DOCUMENT = REFERENCE_DOCUMENT.replace(
    "### Non-Core Features", "### Optional Features"
)
assert UNKNOWN_FEATURE_ROOT_DOCUMENT != REFERENCE_DOCUMENT

#: Falsifier: a `Feature Class` value outside the closed pair. Unlike every
#: other document here this one does not report at all -- the parser refuses it
#: -- so it is asserted against the failure, not against a blocker list.
UNKNOWN_FEATURE_CLASS_VALUE = "optional"
UNKNOWN_FEATURE_CLASS_DOCUMENT = REFERENCE_DOCUMENT.replace(
    "Feature Class: non_core", f"Feature Class: {UNKNOWN_FEATURE_CLASS_VALUE}", 1
)
assert UNKNOWN_FEATURE_CLASS_DOCUMENT != REFERENCE_DOCUMENT
UNKNOWN_FEATURE_CLASS_ERROR = (
    f"unknown capability feature class `{UNKNOWN_FEATURE_CLASS_VALUE}`; "
    "expected core or non_core"
)


def _flat_document(brief: str, members: tuple[tuple[tuple[Any, ...], str | None], ...]) -> str:
    """A pre-migration document: no feature roots, `###` capability headings.

    Each member carries the class it *declares*, or `None` for the unclassified
    shape every adopter document had before #3059. The index lists every member,
    because a capability missing from the index is a different defect and would
    confound what the migration legs are asserting.
    """
    index = "\n".join(_index_rows((member,), "pre-migration") for member, _ in members)
    body = "".join(_section(member, declared, "###") for member, declared in members)
    return f"""# Lumen

## Brief

{brief}

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
{index}

{body}"""


#: The pre-migration shape with nothing classified: the only input from which
#: `aw capability migrate` has to derive every class itself rather than copy a
#: declaration.
UNCLASSIFIED_DOCUMENT = _flat_document(
    "Lumen reference fixture, pre-migration: nothing is classified.",
    tuple((member, None) for member in _CORE_MEMBERS + _NON_CORE_MEMBERS),
)

#: The one capability whose tracker state is live in `LIVE_TRACKER_DOCUMENT`,
#: and the two places it is carried. Both are needed, not one: `Root WI:` and
#: the work-root `WI` cell are erased by two different assignments, and
#: `root_wi_for_capability` (`capability.rs:9253-9270`) reads the field first
#: and falls back to the work-root value -- so a document carrying only the
#: field would render `-` the moment the field were blanked, whatever happened
#: to the fallback, and a document carrying only the fallback would never
#: exercise the field. Carrying both makes each assignment independently
#: observable in the rendered index.
LIVE_TRACKER_ID = "search-core"
LIVE_TRACKER_TITLE = "Search Core"
LIVE_TRACKER_WORK_ROOT = "query-planner-boolean-eval"
LIVE_ROOT_WI = "#91"
LIVE_WORK_ROOT_WI = "#92"


def _live_tracker_document() -> str:
    """`UNCLASSIFIED_DOCUMENT` with one capability's tracker state left live.

    Every other fixture document in this module is authored the way `aw wi`
    leaves one: `Root WI: -` and `-` in every work-root `WI` cell. That is a
    faithful shape, but it is also one from which erasing tracker state cannot
    change a single byte -- so the second of the two transformations
    `migrated_capability_document` documents (`capability.rs:8480-8493`) was
    bound by nothing until this document existed. Hand-authored adopter
    documents predating `aw capability` do carry these values, which is why the
    transformation is there at all.
    """
    document = UNCLASSIFIED_DOCUMENT
    marker = f"ID: {LIVE_TRACKER_ID}\n"
    assert document.count(marker) == 1, marker
    head, tail = document.split(marker, 1)
    assert tail.count("Root WI: -\n") >= 1
    tail = tail.replace("Root WI: -\n", f"Root WI: {LIVE_ROOT_WI}\n", 1)
    row = f"| {LIVE_TRACKER_WORK_ROOT} | change | - |"
    assert tail.count(row) == 1, row
    tail = tail.replace(
        row, f"| {LIVE_TRACKER_WORK_ROOT} | change | {LIVE_WORK_ROOT_WI} |", 1
    )
    return head + marker + tail


LIVE_TRACKER_DOCUMENT = _live_tracker_document()
assert LIVE_TRACKER_DOCUMENT.count(LIVE_ROOT_WI) == 1
assert LIVE_TRACKER_DOCUMENT.count(LIVE_WORK_ROOT_WI) == 1


def assert_migration_erases_document_stored_tracker_state(migrated: str) -> None:
    """Format migration drops every work-item reference the document stored.

    The rule and its reason are documented at `capability.rs:8488-8489`:
    delivery provenance is one-way, so a capability contract never stores a
    work-item reference back. `apps/agentic-workflow/CAPABILITIES.md` declares
    it implemented and verified. Deleting the whole
    `clear_document_stored_tracker_state` call, or any one of its three
    assignments, changes rendered output only for a document that carried
    tracker state to begin with -- which is what `LIVE_TRACKER_DOCUMENT` is for.

    Each of the three assignments is asserted through a different rendered
    surface, so no single one of them can be deleted silently:

    - `capability.current_state` -> the index `Root WI` column, and the
      section's own `Root WI:` field.
    - `gap.active_wi = None` -> also the index column, via the fallback in
      `root_wi_for_capability`. With the field already blanked, the fallback is
      the only thing that can put `#92` back there.
    - `work_root.wi` -> the work-root table's `WI` cell.

    The class derivation and the gate inventory are asserted too, so a
    migration that erased tracker state by destroying the section could not
    pass.
    """
    for value in (LIVE_ROOT_WI, LIVE_WORK_ROOT_WI):
        assert not re.search(rf"{re.escape(value)}(?!\d)", migrated), (
            f"document-stored tracker state {value} survived migration"
        )

    rows = {row[0]: row for row in _index_rows_parsed(migrated)}
    assert LIVE_TRACKER_TITLE in rows, sorted(rows)
    assert rows[LIVE_TRACKER_TITLE][1] == "-", rows[LIVE_TRACKER_TITLE]

    body = _capability_section_body(migrated, LIVE_TRACKER_TITLE)
    lines = [raw.strip() for raw in body.splitlines()]
    root_wi_fields = [line for line in lines if line.startswith("Root WI:")]
    assert root_wi_fields == ["Root WI: -"], root_wi_fields

    work_root_rows = [
        line for line in lines if line.startswith(f"| {LIVE_TRACKER_WORK_ROOT} |")
    ]
    assert len(work_root_rows) == 1, work_root_rows
    cells = [cell.strip() for cell in work_root_rows[0].strip("|").split("|")]
    assert cells[2] == "-", cells

    assert f"- tech-design/{LIVE_TRACKER_ID}.md" in body, body
    _assert_declared_class(migrated, LIVE_TRACKER_ID, "core")


#: Capability titles in the order the two roots impose, which is the order both
#: the index and the sections of a migrated document must be in.
CORE_TITLES = tuple(member[0] for member in _CORE_MEMBERS)
NON_CORE_TITLES = tuple(member[0] for member in _NON_CORE_MEMBERS)
GROUPED_TITLES = CORE_TITLES + NON_CORE_TITLES

#: The same unclassified shape, non-core first. `UNCLASSIFIED_DOCUMENT` is
#: core-first, so its raw document order and its grouped render order coincide
#: by construction -- migration could emit the index in raw order and every
#: assertion drawn from it would still pass. Reversing the two groups separates
#: those orders, which is the only input shape under which the index/section
#: agreement rule can fail at all.
NON_CORE_FIRST_DOCUMENT = _flat_document(
    "Lumen reference fixture, pre-migration: nothing is classified, non-core first.",
    tuple((member, None) for member in _NON_CORE_MEMBERS + _CORE_MEMBERS),
)
#: Asserted, not assumed: if a future edit to the member tuples made this
#: document core-first again, the fixed-point leg would go quiet rather than
#: fail, and the blindness this constant exists to remove would come back.
assert NON_CORE_FIRST_DOCUMENT.index(f"### {NON_CORE_TITLES[0]}\n") < (
    NON_CORE_FIRST_DOCUMENT.index(f"### {CORE_TITLES[0]}\n")
), "NON_CORE_FIRST_DOCUMENT must present the non-core group first"


def baseline_declared_core_document(cap_id: str) -> str:
    """Falsifier 1, per baseline: promote one baseline into the core root.

    Generated per id rather than fixed, so "every archetype service baseline is
    non-core" is asserted for every baseline the fixture names instead of for
    one representative.
    """
    promoted = next(member for member in _NON_CORE_MEMBERS if member[1] == cap_id)
    remaining = tuple(member for member in _NON_CORE_MEMBERS if member[1] != cap_id)
    return _document(_CORE_MEMBERS + (promoted,), remaining)


#: Baselines that belong to only one of the two registries the baseline rule
#: reads. `trait_derived_baseline_ids` is the union of
#: `doc_mirror::CAPABILITY_FAMILIES` baselines and `doc_mirror::TRAITS`
#: baselines, and every id in `NON_CORE_IDS` above sits in *both* -- so a
#: fixture that names only Lumen's own baselines cannot tell the union from
#: either half, and an implementation that dropped one registry entirely would
#: keep rejecting all four while waving through the ids only the dropped half
#: supplies. `agent-task-navigation` is family-only; the other two are
#: trait-only. Both trait-only ids are kept rather than one, so that moving a
#: single id between the registries cannot silently collapse this coverage back
#: to what it was.
REGISTRY_SPANNING_BASELINE_IDS = (
    "agent-task-navigation",
    "developer-agent-experience",
    "stateful-service-workload",
)
assert not set(REGISTRY_SPANNING_BASELINE_IDS) & set(NON_CORE_IDS + CORE_IDS), (
    "a registry-spanning baseline id also appears in the fixture's own "
    "membership, so it would not extend the rule beyond what NON_CORE_IDS covers"
)


def registry_spanning_baseline_core_document(cap_id: str) -> str:
    """Falsifier 1 again, on a baseline this fixture does not otherwise carry.

    The capability is added under the core root rather than promoted out of the
    non-core root, because Lumen does not own it: the claim under test is about
    the baseline registry, not about Lumen's membership. Every other capability
    keeps its correct class, so the single expected blocker is this rule and not
    a side effect of rearranging the document.
    """
    title = " ".join(word.capitalize() for word in cap_id.split("-"))
    member = (
        title,
        cap_id,
        f"Carry the {title.lower()} archetype baseline.",
        "lumen serve",
        (f"{cap_id}-work-root",),
    )
    return _document(_CORE_MEMBERS + (member,), _NON_CORE_MEMBERS)


#: Falsifier 2 -- the field and the containing root disagree. An implementation
#: that reads only the field, or only the root, accepts this. The capability is
#: not a baseline, so falsifier 1's rule cannot be what rejects it.
ROOT_FIELD_CONFLICT_DOCUMENT = _document(
    _CORE_MEMBERS + (_CONFLICT_MEMBER,),
    _NON_CORE_MEMBERS,
    core_class="core",
).replace(
    f"ID: {CONFLICT_ID}\nType: Service\nFeature Class: core",
    f"ID: {CONFLICT_ID}\nType: Service\nFeature Class: non_core",
)


#: Falsifier 3 -- the same root declared twice. This is the rule the earlier
#: whitelist-based filter structurally could not see: its message names neither
#: "feature class" nor "feature root", so it is the sharpest available proof that
#: `document_blockers` subtracts an environment rather than admitting a wording.
DUPLICATE_ROOT_DOCUMENT = REFERENCE_DOCUMENT.replace(
    "### Non-Core Features",
    "### Core Features\n\n### Non-Core Features",
    1,
)
assert DUPLICATE_ROOT_DOCUMENT.count("### Core Features") == 2

#: Falsifier 4 -- one capability listed under both roots. A bare heading is
#: enough: `scan_feature_roots` records root membership from headings alone, so
#: this yields a single blocker with all six capabilities still parsing, which is
#: the same shape as the three falsifiers above. Duplicating the whole section
#: instead would collapse the document to "no capability sections found", a
#: different defect and not this rule.
MULTIPLY_CLASSIFIED_DOCUMENT = (
    REFERENCE_DOCUMENT.rstrip("\n") + f"\n\n#### {_CORE_MEMBERS[0][0]}\n"
)
MULTIPLY_CLASSIFIED_ID = _CORE_MEMBERS[0][1]

#: A legacy capability table: the shape every adopter document had before the
#: canonical field style. It parses to zero capability sections and N legacy
#: rows, which is the *other* branch of the default-class rule -- `report`
#: attributes the row count wholly to non-core instead of letting the rows fall
#: out of both classes. The columns are the legacy header the parser recognizes.
_LEGACY_ROWS = (
    ("Search Core", "shipped", "none", "#1", "`lumen serve`"),
    ("Lexical Search", "shipped", "none", "#2", "`lumen serve`"),
    ("Security Hardening", "partial", "audit coverage", "#3", "`lumen auth`"),
)
LEGACY_ROW_COUNT = len(_LEGACY_ROWS)
LEGACY_TABLE_DOCUMENT = """# Lumen

## Brief

Lumen reference fixture, legacy shape: a pre-canonical capability table.

## Capabilities

| Capability | Current State | Gaps | Active WI | Evidence |
|---|---|---|---|---|
""" + "".join(
    f"| {name} | {state} | {gaps} | {wi} | {evidence} |\n"
    for name, state, gaps, wi, evidence in _LEGACY_ROWS
)

#: The blocker a legacy table is expected to raise. Asserted exactly, so the
#: shape is proven to be *diagnosed* rather than silently accepted.
LEGACY_TABLE_BLOCKER = (
    "legacy capability table detected; migrate rows to canonical field-style "
    "capability contracts under ## Capabilities"
)

#: The legacy rows split by the same derivation the section branch uses:
#: `migration_feature_class` on the slugified row title. Two authored promises,
#: one trait-derived baseline -- so an inverted derivation cannot satisfy this
#: by symmetry.
LEGACY_CORE_ROW_IDS = ("search-core", "lexical-search")
LEGACY_NON_CORE_ROW_IDS = ("security-hardening",)

#: One retired member, which is what makes the word "retained" in the totals
#: mean anything. `feature_class_totals` filters retired items out of the
#: per-class counts with the same conjunct that defines `capability_count`, so
#: unless some document carries a retired capability, every pair-sum assertion
#: holds vacuously in exactly that dimension.
RETIRED_ID = "security-hardening"
#: That member's work-root count, stated rather than inferred, so the expected
#: arithmetic below is readable without cross-referencing `_NON_CORE_MEMBERS`.
RETIRED_CLAIM_COUNT = 1
assert RETIRED_ID in NON_CORE_IDS
assert (
    len(next(m for m in _NON_CORE_MEMBERS if m[1] == RETIRED_ID)[4])
    == RETIRED_CLAIM_COUNT
)

#: The reference document with exactly one baseline retired. Generated through
#: the `status` parameter rather than text-patched, so it cannot silently stop
#: matching if the section layout changes.
RETIRED_MEMBER_DOCUMENT = _document(
    _CORE_MEMBERS, _NON_CORE_MEMBERS, retired_id=RETIRED_ID
)
assert RETIRED_MEMBER_DOCUMENT.count("Status: retired") == 1

#: `observability` is a real Lumen capability and *not* a trait-derived
#: baseline, so migration's derivation would call it core. Declaring it
#: `non_core` before migration therefore puts the declaration and the derivation
#: in direct conflict, which is the only shape that can tell "the author's class
#: is preserved" apart from "the derivation happened to agree".
DECLARED_CLASS_ID = CONFLICT_ID
DECLARED_CLASS = "non_core"

#: The pre-migration shape again, except one capability already states its class.
PARTIALLY_CLASSIFIED_DOCUMENT = _flat_document(
    "Lumen reference fixture, pre-migration: one capability states its class.",
    tuple((member, None) for member in _CORE_MEMBERS + _NON_CORE_MEMBERS)
    + ((_CONFLICT_MEMBER, DECLARED_CLASS),),
)


def digest_production_contract(repository_root: Path) -> dict[str, str]:
    """Digest Lumen's real contract so mutation of it cannot go unnoticed."""
    digests: dict[str, str] = {}
    for relative in LUMEN_PRODUCTION_CONTRACT_PATHS:
        path = repository_root / relative
        if not path.is_file():
            continue
        digests[relative] = hashlib.sha256(path.read_bytes()).hexdigest()
    assert digests, f"no Lumen production contract found under {repository_root}"
    return digests


def assert_production_contract_unmutated(
    before: dict[str, str],
    after: dict[str, str],
) -> None:
    assert before == after, f"Lumen production contract changed: {before} -> {after}"


#: A scratch project has no Python EC or TD inventory, so every report carries
#: exactly these three blockers regardless of the document. They are subtracted
#: by prefix (the messages embed a temporary path).
#:
#: Subtraction, not a whitelist of feature-class wordings. A whitelist silently
#: drops any finding phrased outside it — the duplicate-root message
#: ("duplicate `### Core Features` root (2 occurrences)") names neither "feature
#: class" nor "feature root" — which would turn the reference document's
#: `== []` assertion into a rubber stamp for every rule it failed to anticipate.
#: Subtracting a closed, pinned environment set is total: a finding this fixture
#: never imagined still shows up.
_ENVIRONMENT_BLOCKER_PREFIXES = (
    "Python EC inventory unavailable:",
    "Python TD inventory unavailable:",
    "td capability scan unavailable:",
)

#: The subset that every report carries no matter what the document contains.
_UNCONDITIONAL_ENVIRONMENT_PREFIXES = frozenset(_ENVIRONMENT_BLOCKER_PREFIXES[:2])


def document_blockers(report: dict[str, Any]) -> list[str]:
    """Every blocker the *document* caused, with the scratch environment removed.

    The environment set is asserted to be exactly what is expected, so a change
    in fixture surroundings fails loudly here instead of quietly widening or
    narrowing what the subtraction removes.
    """
    blockers = report["blockers"]
    environment = [
        blocker
        for blocker in blockers
        if blocker.startswith(_ENVIRONMENT_BLOCKER_PREFIXES)
    ]
    matched = {
        prefix
        for prefix in _ENVIRONMENT_BLOCKER_PREFIXES
        for blocker in environment
        if blocker.startswith(prefix)
    }
    # One blocker per matched prefix, never two, so the subtraction cannot
    # quietly absorb a repeat of an environment message.
    assert len(environment) == len(matched), (
        f"scratch-environment blockers repeated a prefix; got {environment}"
    )
    # The two inventory blockers are unconditional. The td scan is not: `report`
    # only scans TD refs when the document parsed at least one capability
    # section (capability.rs:6158), so a legacy-only document structurally
    # cannot emit it. Requiring all three here would fail on that shape for a
    # reason that has nothing to do with feature classes.
    assert matched.issuperset(_UNCONDITIONAL_ENVIRONMENT_PREFIXES), (
        f"expected the inventory blockers to be present; got {environment}"
    )
    return [blocker for blocker in blockers if blocker not in environment]


def assert_feature_class_attribution(report: dict[str, Any]) -> None:
    """The report attributes each capability to the class the document declares."""
    by_id = {item["id"]: item for item in report["capabilities"]}
    assert set(by_id) == set(CORE_IDS) | set(NON_CORE_IDS), sorted(by_id)
    for cap_id in CORE_IDS:
        assert by_id[cap_id]["feature_class"] == "core", by_id[cap_id]
    for cap_id in NON_CORE_IDS:
        assert by_id[cap_id]["feature_class"] == "non_core", by_id[cap_id]

    # Counts, not just per-item fields: an implementation could echo the parsed
    # field back and still roll every capability into one class.
    assert report["core_capability_count"] == len(CORE_IDS), report
    assert report["non_core_capability_count"] == len(NON_CORE_IDS), report
    assert report["core_claim_count"] == CORE_CLAIM_COUNT, report
    assert report["non_core_claim_count"] == NON_CORE_CLAIM_COUNT, report

    # Attribution is total: each pair sums to the retained total, so no
    # capability or claim can fall out of both classes.
    assert (
        report["core_capability_count"] + report["non_core_capability_count"]
        == report["capability_count"]
    ), report
    assert (
        report["core_claim_count"] + report["non_core_claim_count"] == report["claim_count"]
    ), report
    assert (
        report["core_verified_count"] + report["non_core_verified_count"]
        == report["verified_count"]
    ), report
    assert (
        report["core_verified_claim_count"] + report["non_core_verified_claim_count"]
        == report["verified_claim_count"]
    ), report

    # `Production | ready` in the index is a declaration; no claim gate has run,
    # so neither class may report verified claims.
    assert report["core_verified_claim_count"] == 0, report
    assert report["non_core_verified_claim_count"] == 0, report
    assert report["production_ready"] is False, report

    # The correctly classified document is the negative half of the falsifiers
    # below: whatever rule rejects them must stay silent here, or "rejected"
    # would carry no information.
    assert document_blockers(report) == [], report["blockers"]


def assert_unclassified_defaults_to_non_core(report: dict[str, Any]) -> None:
    """An undeclared class reads as non-core, and is still attributed.

    This is the report of `UNCLASSIFIED_DOCUMENT` *before* migration: no root, no
    field, nothing to parse. The default is the whole rule under test, so it is
    asserted through the totals rather than only through the per-item field --
    flipping the default to core moves all six capabilities and all seven claims
    across, which no other assertion in this fixture would see, because every
    other document it runs states its own answer.

    A document that declares nothing must also raise nothing: silence is a legal
    pre-migration state, not a defect to report.
    """
    assert document_blockers(report) == [], report["blockers"]

    for item in report["capabilities"]:
        assert item.get("feature_class") is None, item

    assert report["capability_count"] == len(CORE_IDS) + len(NON_CORE_IDS), report
    assert report["claim_count"] == CORE_CLAIM_COUNT + NON_CORE_CLAIM_COUNT, report

    assert report["core_capability_count"] == 0, report
    assert report["core_claim_count"] == 0, report
    assert report["non_core_capability_count"] == report["capability_count"], report
    assert report["non_core_claim_count"] == report["claim_count"], report


def assert_baseline_core_is_rejected(report: dict[str, Any], cap_id: str) -> None:
    """A trait-derived baseline declared core must be named as a blocker.

    Asserted exactly, and asserted to be the only feature-class blocker: a rule
    that rejected every document, or that rejected the right document while
    naming the wrong capability, would pass a mere `any(...)`.
    """
    assert document_blockers(report) == [
        f"trait-derived baseline capability `{cap_id}` is classified `core`; "
        "archetype baselines are always `non_core` and belong under `Non-Core Features`"
    ], report["blockers"]


def assert_root_field_conflict_is_rejected(report: dict[str, Any]) -> None:
    """A `Feature Class` field contradicting its containing root is a blocker.

    The message has to name both sides of the disagreement, because the author
    cannot tell which of the two to change from the fact of rejection alone.
    """
    assert document_blockers(report) == [
        f"capability `{CONFLICT_ID}` declares `Feature Class: non_core` but is "
        "nested under `Core Features`; make the field and the root agree"
    ], report["blockers"]


def assert_duplicate_root_is_rejected(report: dict[str, Any]) -> None:
    """The same feature root declared twice is a blocker, and the only one.

    Kept as its own falsifier because it is the one root rule whose message
    mentions neither the field nor the word "class". If `document_blockers` ever
    regresses to matching wordings instead of subtracting the environment, this
    assertion is what fails.
    """
    assert document_blockers(report) == [
        "duplicate `### Core Features` root (2 occurrences); "
        "merge them into one root under `## Capabilities`"
    ], report["blockers"]


def assert_missing_non_core_root_is_rejected(report: dict[str, Any]) -> None:
    """Deleting a canonical root is rejected, and so is every capability it stranded.

    Asserted as the whole ordered blocker list rather than as one message,
    because deleting the root cannot produce one message: the four non-core
    capabilities end up nested under `Core Features` while still declaring
    `non_core`, so the missing-root finding necessarily arrives with one
    field/root contradiction per stranded capability. Asserting the set is what
    makes the co-occurrence a property under test instead of a reason to skip
    the rule -- which is what an earlier revision of this module did.
    """
    assert len(report["capabilities"]) == len(CORE_IDS) + len(NON_CORE_IDS), report
    assert document_blockers(report) == [
        "capability document declares feature classes but is missing the "
        "`### Non-Core Features` root; add it under `## Capabilities` so both "
        "canonical roots exist",
        *(
            f"capability `{cap_id}` declares `Feature Class: non_core` but is "
            "nested under `Core Features`; make the field and the root agree"
            for cap_id in NON_CORE_IDS
        ),
    ], report["blockers"]


def assert_unknown_feature_root_is_rejected(report: dict[str, Any]) -> None:
    """A root outside the closed pair is named as unknown, not silently accepted.

    Renaming the root also removes it, so this too is asserted as the whole
    ordered list. The capabilities beneath it do *not* contradict their field
    here -- an unknown root is not `Core Features` -- which is what makes this
    list two long rather than five, and is the discriminating difference from
    the missing-root falsifier above.
    """
    assert len(report["capabilities"]) == len(CORE_IDS) + len(NON_CORE_IDS), report
    assert document_blockers(report) == [
        "capability document declares feature classes but is missing the "
        "`### Non-Core Features` root; add it under `## Capabilities` so both "
        "canonical roots exist",
        "unknown feature root `Optional Features`; the closed pair is "
        "`Core Features` and `Non-Core Features` — move its capabilities under "
        "one of those two",
    ], report["blockers"]


def assert_unknown_feature_class_value_is_refused(
    returncode: int, stderr: str
) -> None:
    """A `Feature Class` value outside the closed pair fails the command outright.

    This is the guard on the *other* direction of the same default the fixture
    already binds. `effective_feature_class` resolves an undeclared class to
    non-core, which is correct; a *mistyped* class must not take that same path,
    or `Feature Class: cire` would land silently in non-core and read as a
    deliberate classification. The parser refuses it instead, so this is the one
    document here asserted against a failure rather than a blocker list.
    """
    assert returncode != 0, "a feature class outside the closed pair must not report"
    assert UNKNOWN_FEATURE_CLASS_ERROR in stderr, stderr


def assert_human_report_renders_the_split(stdout: str, report: dict[str, Any]) -> None:
    """`--human` renders the same split the JSON envelope reports.

    The human line exists only to show the two classes, and it is built from its
    own format string rather than from the JSON serializer, so the core and
    non-core operands can be transposed there while every JSON-reading leg in
    this fixture stays green. Expected values are taken from the JSON report of
    the same run, so this asserts the two surfaces agree rather than restating
    a hardcoded count that could drift from both.
    """
    expected = (
        "readiness by feature class: "
        f"core={report['core_verified_count']}/{report['core_capability_count']} "
        f"capabilities, {report['core_verified_claim_count']}/"
        f"{report['core_claim_count']} claims; "
        f"non_core={report['non_core_verified_count']}/"
        f"{report['non_core_capability_count']} capabilities, "
        f"{report['non_core_verified_claim_count']}/"
        f"{report['non_core_claim_count']} claims"
    )
    lines = [line.strip() for line in stdout.splitlines()]
    assert expected in lines, (
        f"`aw capability report --human` did not render the split as the JSON "
        f"envelope reports it.\nexpected: {expected}\ngot: "
        + "\n".join(line for line in lines if "feature class" in line)
    )
    # The two classes must be distinguishable in that line, or a transposition
    # of equal operands would pass. Asserted on the fixture's real shape: it has
    # two core capabilities and four non-core ones.
    assert report["core_capability_count"] != report["non_core_capability_count"], (
        "the reference fixture must keep the two class counts different, or the "
        "human line could not detect a transposition"
    )
    assert report["core_claim_count"] != report["non_core_claim_count"], (
        "the reference fixture must keep the two claim counts different, or a "
        "transposition of the claim operands would pass"
    )
    # No operand may be zero. Four of the eight are verified counts, and on a
    # report with no gate execution they are all zero, so a transposition
    # confined to them is invisible and this leg would silently cover half of
    # what its name claims. The caller must drive it on a `--verify` report;
    # this guard is what forces that, rather than trusting the leg order to
    # stay correct.
    for key in READINESS_OPERAND_KEYS:
        assert report[key] > 0, (
            f"assert_human_report_renders_the_split must be driven on a report "
            f"whose eight readiness operands are all populated; `{key}` is "
            f"{report[key]}, and a transposition involving a zero cannot be "
            f"detected"
        )
    assert report["core_verified_count"] != report["non_core_verified_count"], report
    assert (
        report["core_verified_claim_count"] != report["non_core_verified_claim_count"]
    ), report
    # And each verified count must fall short of its own total. Distinguishing
    # the two *classes* says nothing about whether the line pairs each verified
    # count with the right denominator: on a fully verified report the two are
    # the same integer, so a line that rendered `core=2/2` by reading the total
    # twice renders exactly what a correct line renders. This is the guard that
    # forces the caller onto a partially verified report.
    assert report["core_verified_count"] < report["core_capability_count"], report
    assert report["core_verified_claim_count"] < report["core_claim_count"], report
    assert (
        report["non_core_verified_count"] < report["non_core_capability_count"]
    ), report
    assert (
        report["non_core_verified_claim_count"] < report["non_core_claim_count"]
    ), report


def assert_capability_under_both_roots_is_rejected(report: dict[str, Any]) -> None:
    """One capability under both roots is a blocker, and the only one.

    The negative half matters as much as the positive: every capability must
    still parse, so this cannot pass by the document having collapsed into
    something the parser gave up on.
    """
    assert len(report["capabilities"]) == len(CORE_IDS) + len(NON_CORE_IDS), report
    assert document_blockers(report) == [
        f"capability `{MULTIPLY_CLASSIFIED_ID}` is classified under both "
        "`Core Features` and `Non-Core Features`; keep exactly one root"
    ], report["blockers"]


def assert_legacy_rows_are_attributed_to_non_core(report: dict[str, Any]) -> None:
    """A legacy table's rows land wholly in non-core rather than nowhere.

    This is the second branch of the default-class rule. `UNCLASSIFIED_DOCUMENT`
    covers the branch where capability sections parse and each one defaults;
    this covers the branch where none parse at all and the count comes from
    `legacy_rows` instead (`apply_feature_class_totals`, capability.rs:1287).
    Neutering that branch leaves every other assertion in this fixture green
    while three capabilities sit in neither class.

    The shape under test is asserted rather than assumed -- `capabilities` empty
    with a non-zero `capability_count` is what routes through the branch, so if a
    future change makes legacy rows parse into sections this fails loudly instead
    of silently testing the other branch twice.
    """
    assert report["capabilities"] == [], report["capabilities"]
    assert report["capability_count"] == LEGACY_ROW_COUNT, report

    assert report["core_capability_count"] == 0, report
    assert report["core_claim_count"] == 0, report

    # The discriminating assertion: the row count is attributed, not dropped.
    # (`claim_count` is 0 in this shape -- legacy rows carry no work roots -- so
    # the claim pair-sum below is real but not what catches the mutation.)
    assert report["non_core_capability_count"] == report["capability_count"], report
    assert report["non_core_claim_count"] == report["claim_count"], report
    assert report["non_core_verified_count"] == report["verified_count"], report

    # Diagnosed, not silently accepted.
    assert document_blockers(report) == [LEGACY_TABLE_BLOCKER], report["blockers"]


def _assert_declared_class(migrated: str, cap_id: str, expected: str) -> None:
    """The `Feature Class:` field of one capability equals exactly `expected`.

    Exact equality rather than containment, so `non_core` cannot satisfy a
    `core` expectation by substring.
    """
    at = migrated.find(f"ID: {cap_id}\n")
    assert at != -1, f"`{cap_id}` missing from the migrated document"
    block = migrated[at:]
    line = next(
        stripped
        for stripped in (raw.strip() for raw in block.splitlines())
        if stripped.startswith("Feature Class:")
    )
    assert line == f"Feature Class: {expected}", f"{cap_id}: {line}"


def assert_retired_capability_is_excluded_from_both_classes(
    report: dict[str, Any],
) -> None:
    """A retired capability leaves both per-class counts and both totals.

    This is what earns the word "retained" in every other pair-sum assertion
    here. `feature_class_totals` excludes retired items with the same conjunct
    that defines `capability_count`, so with no retired member anywhere in the
    fixture the two filters could disagree and every pair still sum.

    Asserted to be about *attribution*, not parsing: the retired member is still
    present in `capabilities` carrying its declared class, so this cannot pass
    because the capability vanished from the document.
    """
    by_id = {item["id"]: item for item in report["capabilities"]}
    assert set(by_id) == set(CORE_IDS) | set(NON_CORE_IDS), sorted(by_id)
    retired = by_id[RETIRED_ID]
    assert retired["status"] == "retired", retired
    assert retired["feature_class"] == "non_core", retired

    # Retired leaves the totals...
    assert report["capability_count"] == len(CORE_IDS) + len(NON_CORE_IDS) - 1, report
    assert (
        report["claim_count"]
        == CORE_CLAIM_COUNT + NON_CORE_CLAIM_COUNT - RETIRED_CLAIM_COUNT
    ), report

    # ...and leaves the class it declared, rather than only one of the two.
    assert report["core_capability_count"] == len(CORE_IDS), report
    assert report["core_claim_count"] == CORE_CLAIM_COUNT, report
    assert report["non_core_capability_count"] == len(NON_CORE_IDS) - 1, report
    assert (
        report["non_core_claim_count"] == NON_CORE_CLAIM_COUNT - RETIRED_CLAIM_COUNT
    ), report

    # The pair-sums still hold, now against a total that actually excludes
    # something -- which is the whole point of this leg.
    assert (
        report["core_capability_count"] + report["non_core_capability_count"]
        == report["capability_count"]
    ), report
    assert (
        report["core_claim_count"] + report["non_core_claim_count"]
        == report["claim_count"]
    ), report

    # Retiring a capability is a legal shape, not a diagnosed one.
    assert document_blockers(report) == [], report["blockers"]


def assert_retired_is_excluded_from_the_verified_counts_too(
    report: dict[str, Any],
) -> None:
    """The retired exclusion holds in all four dimensions, not only in two.

    `feature_class_totals` applies one `status != Retired` filter to four
    accumulators (`capability.rs:1300-1310`). The unverified retired leg can
    only see two of them: with no gate execution both verified accumulators are
    zero, so an implementation that counted retired items into
    `verified_count` and `verified_claim_count` produced identical output. Half
    the exclusion was asserted vacuously, which is the same defect shape
    `assert_verified_split_is_non_degenerate` was added to fix on the reference
    document and left unfixed here.

    Run under `--verify`, where all four are populated, and pinned individually
    against the retained totals. The distinctness guards are what make
    "individually" mean something -- and the fixture's claim counts are spaced
    two apart precisely so that the claim pair stays unequal after one claim is
    retired away.
    """
    assert report["capability_count"] == len(CORE_IDS) + len(NON_CORE_IDS) - 1, report
    assert report["verified_count"] == report["capability_count"], report
    assert (
        report["verified_claim_count"]
        == CORE_CLAIM_COUNT + NON_CORE_CLAIM_COUNT - RETIRED_CLAIM_COUNT
    ), report

    assert report["core_verified_count"] == len(CORE_IDS), report
    assert report["non_core_verified_count"] == len(NON_CORE_IDS) - 1, report
    assert report["core_verified_claim_count"] == CORE_CLAIM_COUNT, report
    assert (
        report["non_core_verified_claim_count"]
        == NON_CORE_CLAIM_COUNT - RETIRED_CLAIM_COUNT
    ), report

    # Non-vacuity: neither pinned pair may be interchangeable.
    assert report["core_verified_count"] != report["non_core_verified_count"], report
    assert (
        report["core_verified_claim_count"] != report["non_core_verified_claim_count"]
    ), report

    # And the pairs must still exhaust the retained totals, so no verified
    # capability or claim can fall out of both classes.
    assert (
        report["core_verified_count"] + report["non_core_verified_count"]
        == report["verified_count"]
    ), report
    assert (
        report["core_verified_claim_count"] + report["non_core_verified_claim_count"]
        == report["verified_claim_count"]
    ), report

    # Still a legal shape, not a diagnosed one.
    assert document_blockers(report) == [], report["blockers"]


def assert_legacy_migration_derives_the_split(migrated: str) -> None:
    """Migration derives both roots for legacy *rows*, not only for sections.

    `render_capability_registry` groups legacy rows through a branch entirely
    separate from the one that handles capability sections, so the derivation
    rule the section legs bind could be inverted here and nothing else in this
    fixture would notice.
    """
    core_at = migrated.index("### Core Features")
    non_core_at = migrated.index("### Non-Core Features")
    assert core_at < non_core_at, migrated

    for cap_id in LEGACY_CORE_ROW_IDS:
        at = migrated.index(f"ID: {cap_id}")
        assert core_at < at < non_core_at, f"{cap_id} not under Core Features"
        _assert_declared_class(migrated, cap_id, "core")

    for cap_id in LEGACY_NON_CORE_ROW_IDS:
        at = migrated.index(f"ID: {cap_id}")
        assert at > non_core_at, f"{cap_id} not under Non-Core Features"
        _assert_declared_class(migrated, cap_id, "non_core")


def assert_migrated_legacy_document_is_accepted(report: dict[str, Any]) -> None:
    """The migrated legacy document is one the checker accepts.

    The negative half of the leg above: without it, migration could emit a
    collapsed or self-contradicting document and the string assertions would
    still find their substrings.
    """
    assert document_blockers(report) == [], report["blockers"]
    ids = {item["id"] for item in report["capabilities"]}
    assert ids == set(LEGACY_CORE_ROW_IDS) | set(LEGACY_NON_CORE_ROW_IDS), sorted(ids)
    assert report["core_capability_count"] == len(LEGACY_CORE_ROW_IDS), report
    assert report["non_core_capability_count"] == len(LEGACY_NON_CORE_ROW_IDS), report
    assert (
        report["core_capability_count"] + report["non_core_capability_count"]
        == report["capability_count"]
    ), report


def assert_migrated_legacy_index_lists_every_row(migrated: str) -> None:
    """The migrated legacy document indexes every row it turned into a section.

    Legacy rows reach the index through their own branch of
    `render_capability_index`, separate from the one the section legs exercise.
    Emptying that branch leaves a document with three capability sections and an
    empty table of contents, which every other assertion here would accept.

    Membership, not order, and deliberately so. The legacy branch renders rows in
    raw table order while the sections are grouped by root, so on a legacy table
    whose first row is a trait-derived baseline the two genuinely disagree at
    HEAD -- a real defect, filed separately rather than asserted here, because
    this fixture must not encode the product's current wrong answer as its
    expectation. Asserting order on this fixture would pass only because
    `_LEGACY_ROWS` happens to be core-first, which is the kind of accidental
    agreement `NON_CORE_FIRST_DOCUMENT` exists to eliminate, not to reintroduce.
    """
    rows = _index_rows_parsed(migrated)
    index_titles = [row[0] for row in rows]
    section_titles = _section_titles(migrated)
    assert len(index_titles) == LEGACY_ROW_COUNT, index_titles
    assert set(index_titles) == set(section_titles), (
        f"the migrated legacy index and its sections cover different "
        f"capabilities; index={sorted(index_titles)} sections={sorted(section_titles)}"
    )

    # Column 2 -- the rendered `Root WI` -- is asserted by
    # `assert_migration_erases_legacy_row_tracker_state`, and the history of
    # *why it was not* is worth keeping visible.
    #
    # An early revision excused it as an equivalent mutation because "no
    # `#1`/`#2`/`#3` survives anywhere". That premise is false: this leg's path
    # erases document-stored tracker state before rendering, but the *other*
    # entry point of `aw capability migrate` -- relocating a README-resident
    # legacy table -- preserves every `Active WI` as `Root WI`. Verified
    # directly, and bound by `assert_readme_relocation_preserves_tracker_state`.
    #
    # The revision after that kept the assertion off on a different ground:
    # asserting `-` would freeze current behavior as the contract while the two
    # paths disagree. Round 17 showed that ground does not hold either, and the
    # refutation is the asymmetry it created. The *preserving* side was already
    # frozen by the relocation assertion, so declining only the *erasing* side
    # did not avoid taking a position on the disagreement -- it took the
    # opposite one, freezing the side this repository has filed as possibly
    # wrong and leaving unasserted the side `capability.rs:8488-8489` documents,
    # `apps/agentic-workflow/CAPABILITIES.md` declares implemented and verified,
    # and whose deletion `aw capability migrate` will happily ship. A
    # disagreement between two paths is a reason to file a defect about one of
    # them, not a reason to leave the documented rule unbound.
    #
    # The standing lesson, recorded because it has now cost four rounds: "this
    # rule cannot be observed" is a claim about the inputs the fixture drives,
    # never about the rule. Enumerate the callers before excusing a mutation as
    # equivalent -- an unexercised caller looks exactly like an unobservable
    # rule.
    #
    # Each successive revision of this comment narrowed the excuse and each was
    # still too broad. "No `#1` survives anywhere" was refuted by the relocation
    # path. "The two migrate paths render through different functions" was
    # refuted too: they render through the *same* function,
    # `render_capability_registry`, which branches on whether the document parsed
    # into capability sections -- and only one of those branches was being
    # driven. The mutation was surviving because one input shape was missing, not
    # because any rule was unreachable. When a mutation cannot be killed, the
    # first hypothesis should be a missing input, and "unobservable" should be
    # the conclusion of an exhausted search rather than its premise.


def assert_migration_erases_legacy_row_tracker_state(migrated: str) -> None:
    """Format migration drops every legacy row's `Active WI`.

    `migrated_capability_document` documents two transformations
    (`capability.rs:8480-8493`): derive the class, and erase document-stored
    tracker state, because delivery provenance is one-way -- a work item
    references capability and claim ids, never the reverse. The class half is
    bound many times over in this case. This is the legacy-row half of the
    other one (`capability.rs:8502-8504`), whose deletion leaves migration
    shipping `| Search Core | #1 | ... |` into the canonical contract.

    Asserted three ways, because the first alone is weaker than it looks: the
    index cell could be `-` while the value survived in a section field, and a
    value could vanish from the index because the index broke rather than
    because it was erased. So the index column, every rendered `Root WI:` field,
    and the absence of the raw values from the whole document are each asserted,
    and the row count is pinned so an empty index cannot satisfy any of them.
    """
    rows = _index_rows_parsed(migrated)
    assert len(rows) == LEGACY_ROW_COUNT, rows
    for row in rows:
        assert row[1] == "-", (
            f"migrated legacy row {row[0]!r} still carries tracker state "
            f"{row[1]!r} in the Capability Index"
        )
    for raw in migrated.splitlines():
        line = raw.strip()
        if line.startswith("Root WI:"):
            assert line == "Root WI: -", (
                f"a migrated legacy section still carries tracker state: {line!r}"
            )
    for title, wi in LEGACY_ROW_TRACKER_STATE.items():
        assert not re.search(rf"{re.escape(wi)}(?!\d)", migrated), (
            f"{title}'s legacy `Active WI` {wi} survived format migration"
        )


def assert_migration_preserves_declared_class(migrated: str) -> None:
    """An author's stated class survives migration; only silence is filled in.

    `observability` is not a trait-derived baseline, so the derivation would put
    it under `Core Features`. It declared `non_core`, so it must come out
    `non_core` — and under the matching root, since migration must not emit the
    field/root contradiction the checker rejects. Asserted alongside the derived
    members, so "declaration wins" cannot be satisfied by migration simply
    declining to classify anything.
    """
    assert_migration_derives_the_split(migrated)

    non_core_root = migrated.find("### Non-Core Features")
    at = migrated.find(f"ID: {DECLARED_CLASS_ID}\n")
    assert at != -1, f"`{DECLARED_CLASS_ID}` missing from the migrated document"
    assert at > non_core_root, (
        f"`{DECLARED_CLASS_ID}` declared `{DECLARED_CLASS}` and must migrate "
        f"under the matching root, not the one its id would have derived"
    )

    block = migrated[at:]
    line = next(
        stripped
        for stripped in (raw.strip() for raw in block.splitlines())
        if stripped.startswith("Feature Class:")
    )
    assert line == f"Feature Class: {DECLARED_CLASS}", (
        f"migration overwrote a class the author stated: {line}"
    )


def assert_migration_derives_the_split(migrated: str) -> None:
    """`aw capability migrate` derives the class from the id, not from a copy.

    The input declared nothing, so every class in `migrated` was computed. This
    is the only assertion in the fixture that binds the derivation rule itself;
    everywhere else the document already states the answer, so the derivation
    could be inverted without any of the other assertions noticing.

    Containment is asserted alongside the field, because a document whose field
    says `core` while the capability sits under `Non-Core Features` is exactly
    the contradiction the checker rejects — migration must not produce it.
    """
    core_root = migrated.find("### Core Features")
    non_core_root = migrated.find("### Non-Core Features")
    assert core_root != -1, migrated
    assert non_core_root != -1, migrated
    assert core_root < non_core_root, "core root must precede non-core"

    for cap_id in CORE_IDS:
        at = migrated.find(f"ID: {cap_id}\n")
        assert at != -1, f"`{cap_id}` missing from the migrated document"
        assert core_root < at < non_core_root, (
            f"`{cap_id}` is an authored promise and must migrate under "
            f"`Core Features`, not `Non-Core Features`"
        )
    for cap_id in NON_CORE_IDS:
        at = migrated.find(f"ID: {cap_id}\n")
        assert at != -1, f"`{cap_id}` missing from the migrated document"
        assert at > non_core_root, (
            f"`{cap_id}` is a trait-derived baseline and must migrate under "
            f"`Non-Core Features`; migration must never manufacture a core "
            f"promise out of an archetype obligation"
        )

    # The field, not only the position: both halves have to agree, or the
    # migrated document is one the checker would turn around and reject.
    expected_classes = [(cap_id, "core") for cap_id in CORE_IDS] + [
        (cap_id, "non_core") for cap_id in NON_CORE_IDS
    ]
    for cap_id, expected in expected_classes:
        _assert_declared_class(migrated, cap_id, expected)


def _index_rows_parsed(migrated: str) -> list[list[str]]:
    """Every Capability Index row as its list of cells, in document order.

    Bounded to the index table itself: rows are taken from `### Capability
    Index` up to the next `###` heading, which is the first feature root.
    """
    start = migrated.index("### Capability Index")
    rest = migrated[start + len("### Capability Index") :]
    end = rest.find("\n### ")
    table = rest if end == -1 else rest[:end]
    rows = []
    for raw in table.splitlines():
        line = raw.strip()
        if not line.startswith("|") or set(line) <= set("|-: "):
            continue
        cells = [cell.strip() for cell in line.strip("|").split("|")]
        if cells and cells[0] == "Capability":
            continue
        rows.append(cells)
    return rows


def _index_titles(migrated: str) -> list[str]:
    """The first cell of every Capability Index row, in document order."""
    return [row[0] for row in _index_rows_parsed(migrated)]


def _section_titles(migrated: str) -> list[str]:
    return [
        raw.strip()[len("#### ") :].strip()
        for raw in migrated.splitlines()
        if raw.strip().startswith("#### ")
    ]


def assert_migration_reaches_a_fixed_point(migrated: str) -> None:
    """The migrated index and the migrated sections are in the same order.

    Migration groups the capability *sections* under the two roots but renders
    the *index* from its own pass. If that pass followed raw document order
    instead of the grouped order, migrating a document whose first capability is
    non-core would emit an index that disagrees with the sections; re-parsing it
    would yield a different document order and render a different index again --
    a migration with no fixed point, so no adopter document could ever converge.

    This is asserted on `NON_CORE_FIRST_DOCUMENT` rather than on
    `UNCLASSIFIED_DOCUMENT` because the core-first input cannot distinguish the
    two orders: they are the same list. Under the non-core-first input they
    differ, and the rule has something to be wrong about.

    Both halves are pinned to the expected grouped order rather than only to
    each other, so this cannot pass by both collapsing to raw input order
    together.
    """
    index_titles = _index_titles(migrated)
    section_titles = _section_titles(migrated)

    assert index_titles == section_titles, (
        f"the migrated index and its capability sections disagree on order; "
        f"index={index_titles} sections={section_titles}"
    )
    assert section_titles == list(GROUPED_TITLES), (
        f"migration must render core-then-non-core, got {section_titles}"
    )


#: The legacy row title -> `Active WI` mapping, derived from `_LEGACY_ROWS` so a
#: change to the fixture cannot silently desynchronize the expectation from it.
LEGACY_ROW_TRACKER_STATE = {name: wi for name, _state, _gaps, wi, _evidence in _LEGACY_ROWS}
assert len(set(LEGACY_ROW_TRACKER_STATE.values())) == LEGACY_ROW_COUNT, (
    "each legacy row must carry a distinct Active WI, or a renderer that emitted "
    "one row's tracker state for every row would satisfy this fixture"
)

#: The pointer `aw capability migrate` must leave behind in a README whose
#: capability table it relocated. Without it the relocation is a silent move: the
#: human-facing document loses its contract with no forwarding address.
README_CONTRACT_POINTER = "## Capability Contract"


def _legacy_title_is_core(title: str) -> bool:
    slug = title.strip().lower().replace(" ", "-")
    return slug in LEGACY_CORE_ROW_IDS


def _capability_section_text(migrated: str, title: str) -> str:
    """The body of one `#### <title>` section, up to the next heading."""
    marker = f"#### {title}\n"
    start = migrated.index(marker)
    rest = migrated[start + len(marker) :]
    end = rest.find("\n#### ")
    if end == -1:
        end = rest.find("\n### ")
    return rest if end == -1 else rest[:end]


def assert_readme_relocation_preserves_tracker_state(readme: str, migrated: str) -> None:
    """Relocating a README-resident legacy table preserves each row's tracker state.

    This binds the *second* entry point of `aw capability migrate`. The legacy leg
    above drives format migration of an existing CAPABILITIES.md; this one drives
    the branch that fires when a project has no CAPABILITIES.md at all and its
    README still carries the legacy table. That branch renders the parsed rows
    directly through `render_capability_registry`'s legacy path rather than
    routing them through the format-migration renderer, so it is the only input in
    this fixture under which a rendered tracker cell is non-blank -- and therefore
    the only one under which emptying that rendering is observable at all.

    Concretely: the format-migration path erases document-stored tracker state
    before rendering, so every `Root WI` cell it emits is already `-` and no
    mutation of that rendering changes its output. Here the legacy `Active WI`
    survives into both the index column and the section field, which are rendered
    by separate passes, so blanking either one is caught.

    The renderer this leg binds is the legacy one that reads `row.active_wi`. It
    is *not* `root_wi_for_capability`: `render_capability_registry` branches on
    `document.capabilities.is_empty()`, and a legacy table parses to zero
    capability sections, so this input takes the legacy branch. An earlier
    revision turned that observation into "relocation never calls that function",
    which is false -- relocation calls it for every README that parses into
    capability *sections*, and renders it into both the index column and the
    section field on live, unblanked tracker state. The correct statement is
    narrower and is about the input, not the path: *this document shape* cannot
    reach it. `assert_relocation_preserves_section_tracker_state` drives the shape
    that does.

    Tracker state is asserted as *preserved*, not as any particular literal: the
    expectation is derived from `_LEGACY_ROWS`, which is the input this leg
    writes. That direction stays honest if the fixture changes, and it is not a
    claim that preserving is more correct than erasing -- the two paths disagree
    at HEAD and that disagreement is filed as a defect, not settled here. What is
    asserted is only that this path does observably what it does, so it cannot be
    silently emptied.
    """
    rows = _index_rows_parsed(migrated)
    index_titles = [row[0] for row in rows]
    assert set(index_titles) == set(LEGACY_ROW_TRACKER_STATE), sorted(index_titles)

    # The index's Root WI column, per row, against that row's own legacy value.
    for row in rows:
        title = row[0]
        assert row[1] == LEGACY_ROW_TRACKER_STATE[title], (
            f"relocated index row {title!r} lost its tracker state; "
            f"expected {LEGACY_ROW_TRACKER_STATE[title]!r}, got {row[1]!r}"
        )

    # The same value again as a section field: the index and the sections are
    # rendered by separate passes, so one can be emptied without the other.
    for title, wi in LEGACY_ROW_TRACKER_STATE.items():
        section = _capability_section_text(migrated, title)
        assert f"Root WI: {wi}\n" in section, (
            f"relocated section {title!r} lost its tracker state; "
            f"expected 'Root WI: {wi}', section was:\n{section}"
        )

    # Relocation must still derive the split and group the sections under the
    # roots that derivation implies. Section order is pinned; index order is
    # membership only, for the reason recorded on the legacy-index leg.
    expected_sections = [
        name for name in LEGACY_ROW_TRACKER_STATE if _legacy_title_is_core(name)
    ] + [name for name in LEGACY_ROW_TRACKER_STATE if not _legacy_title_is_core(name)]
    assert _section_titles(migrated) == expected_sections, (
        f"relocation must group sections core-then-non-core, "
        f"got {_section_titles(migrated)}"
    )
    for cap_id in LEGACY_CORE_ROW_IDS:
        _assert_declared_class(migrated, cap_id, "core")
    for cap_id in LEGACY_NON_CORE_ROW_IDS:
        _assert_declared_class(migrated, cap_id, "non_core")

    # The README keeps a forwarding pointer and gives up the table itself, so the
    # relocation is observable from the document it moved the contract out of.
    assert README_CONTRACT_POINTER in readme, readme
    assert "CAPABILITIES.md" in readme, readme
    assert "| Capability | Current State |" not in readme, readme


#: Per-capability tracker state for the section-shaped relocation fixtures. The
#: values are distinct so a renderer that emitted one capability's `Root WI` for
#: every row could not satisfy the assertions drawn from them.
_ALL_MEMBERS = _CORE_MEMBERS + _NON_CORE_MEMBERS
SECTION_RELOCATION_WI = {
    member[0]: f"#{10 + index}" for index, member in enumerate(_ALL_MEMBERS)
}
assert len(set(SECTION_RELOCATION_WI.values())) == len(_ALL_MEMBERS)


def _section_readme(
    members: tuple[tuple[Any, ...], ...],
    classes: tuple[str | None, ...],
    brief: str,
) -> str:
    """A README whose capability contract is canonical `###` sections.

    This is the relocation input shape the legacy-table fixture cannot produce.
    A legacy table parses to zero capability sections, so it takes
    `render_capability_registry`'s legacy branch; a README shaped like this
    parses into `document.capabilities` and takes the other branch, which is
    where the section renderers and `root_wi_for_capability` live.

    Each capability carries its own `Root WI`, overwriting the neutral `-` the
    section builder emits, so relocation has real tracker state to preserve or
    lose.
    """
    body = ""
    for member, declared in zip(members, classes):
        body += _section(member, declared, "###").replace(
            "Root WI: -", f"Root WI: {SECTION_RELOCATION_WI[member[0]]}"
        )
    return f"""# Lumen

## Brief

{brief}

## Capabilities

{body}
## Contributing

See CONTRIBUTING.md.
"""


#: Relocation input: every capability a section, none classified. Migration has
#: to derive every class *and* render every section itself.
UNCLASSIFIED_SECTION_README = _section_readme(
    _ALL_MEMBERS,
    (None,) * len(_ALL_MEMBERS),
    "Lumen README-resident capability contract, nothing classified.",
)

#: Relocation input: the domain promises declare `core`, the baselines declare
#: nothing. Classified capabilities render under their root one level deeper;
#: unclassified ones keep their top-level position. Both halves render in the
#: same pass, so a renderer that dropped either would still produce a document.
PARTIALLY_CLASSIFIED_SECTION_README = _section_readme(
    _ALL_MEMBERS,
    ("core",) * len(_CORE_MEMBERS) + (None,) * len(_NON_CORE_MEMBERS),
    "Lumen README-resident capability contract, domain promises classified.",
)

#: Relocation input where one canonical class has no members at all. The
#: contract is that migration still emits *both* roots: an absent root is a
#: structural defect its own checker reports, so emitting only the populated one
#: would produce a document the product rejects.
ALL_CORE_SECTION_README = _section_readme(
    _CORE_MEMBERS,
    ("core",) * len(_CORE_MEMBERS),
    "Lumen README-resident capability contract, every capability core.",
)

#: Relocation input spanning all three render groups at once: two capabilities
#: declare `core`, one declares `non_core`, and three declare nothing.
#:
#: `capabilities_in_render_order` (`capability.rs:8908-8932`) concatenates three
#: groups -- `Some(Core)`, `Some(NonCore)`, then `None` -- and every other
#: relocation shape here leaves at most two of them non-empty *and* in an order
#: that happens to equal raw document order, so the array's order is
#: unobservable from them: permuting it renders the identical document. This
#: shape populates all three and interleaves them so that raw order and grouped
#: order differ, which is what makes the concatenation order falsifiable.
MIXED_SECTION_CLASSES = ("core", "core", None, None, None, "non_core")
assert len(MIXED_SECTION_CLASSES) == len(_ALL_MEMBERS)
MIXED_SECTION_README = _section_readme(
    _ALL_MEMBERS,
    MIXED_SECTION_CLASSES,
    "Lumen README-resident capability contract, all three render groups.",
)


def _mixed_titles_in_group_order(order: tuple[str | None, ...]) -> tuple[str, ...]:
    """The mixed shape's titles, concatenated in the given group order."""
    return tuple(
        member[0]
        for group in order
        for member, declared in zip(_ALL_MEMBERS, MIXED_SECTION_CLASSES)
        if declared == group
    )


#: The order the product's own group array implies for the mixed shape.
MIXED_SECTION_GROUPED_TITLES = _mixed_titles_in_group_order(("core", "non_core", None))
_MIXED_GROUP_PERMUTATIONS = tuple(
    _mixed_titles_in_group_order(order)
    for order in permutations(("core", "non_core", None))
)
assert len(set(_MIXED_GROUP_PERMUTATIONS)) == len(_MIXED_GROUP_PERMUTATIONS), (
    "every permutation of the three render groups must produce a distinct title "
    "order, or some reordering of `capabilities_in_render_order` would render "
    "the identical document and this shape could not falsify it"
)
assert MIXED_SECTION_GROUPED_TITLES != tuple(member[0] for member in _ALL_MEMBERS), (
    "the mixed shape must render in an order different from its raw document "
    "order, or an index that followed the input instead of the grouping would "
    "still agree with the sections"
)

#: The titles each relocation input is required to carry through.
UNCLASSIFIED_SECTION_TITLES = tuple(member[0] for member in _ALL_MEMBERS)


def _capability_section_body(migrated: str, title: str) -> str:
    """One capability section's body, at whichever heading level it rendered.

    Classified capabilities render at `####` under a feature root and
    unclassified ones at `###` at top level, so a level-fixed reader would
    silently skip half of a partially classified document -- and silently pass.
    """
    for level in ("#### ", "### "):
        marker = f"{level}{title}\n"
        start = migrated.find(marker)
        if start == -1:
            continue
        rest = migrated[start + len(marker) :]
        # Bounded at the next heading of this level *or shallower*. Stopping
        # only at `\n### ` would let a `####` section run on into its siblings,
        # which reads as this capability owning their fields.
        ends = [at for at in (rest.find(f"\n{level}"), rest.find("\n### ")) if at != -1]
        return rest if not ends else rest[: min(ends)]
    raise AssertionError(
        f"no capability section rendered for {title!r}; document was:\n{migrated}"
    )


def _rendered_capability_titles(migrated: str) -> list[str]:
    """Every capability section title, at either heading level, in order.

    The feature roots are headings at the same level as an unclassified
    capability section, so they are excluded by name rather than by level.
    """
    roots = {"Capability Index", "Core Features", "Non-Core Features"}
    titles = []
    for raw in migrated.splitlines():
        line = raw.strip()
        for level in ("#### ", "### "):
            if line.startswith(level):
                title = line[len(level) :].strip()
                if title not in roots:
                    titles.append(title)
                break
    return titles


def assert_relocation_preserves_section_tracker_state(
    migrated: str, *, expected_order: tuple[str, ...]
) -> None:
    """Relocating a section-shaped README preserves each capability's `Root WI`.

    This is the caller of `root_wi_for_capability` that `aw capability migrate`
    actually reaches. `render_relocated_capability_document` passes the *raw*
    parsed document to `render_capability_registry`, so unlike format migration
    -- which blanks document-stored tracker state before rendering -- the value
    arrives live and is rendered twice, into the index column and into the
    section field, by two separate passes.

    Asserted per capability against the value that capability declared, so a
    renderer emitting one capability's tracker state for all of them, or a
    constant, fails. As on the legacy leg this asserts only that the path does
    observably what it does; whether preserving or erasing is the right contract
    is a disagreement between the two entry points, filed separately rather than
    settled by this fixture.
    """
    rows = _index_rows_parsed(migrated)
    assert [row[0] for row in rows] == list(expected_order), (
        f"the relocated index must list every capability in render order, "
        f"got {[row[0] for row in rows]}"
    )
    for row in rows:
        expected = SECTION_RELOCATION_WI[row[0]]
        assert row[1] == expected, (
            f"relocated index row {row[0]!r} lost its tracker state; "
            f"expected {expected!r}, got {row[1]!r}"
        )
    for title, wi in SECTION_RELOCATION_WI.items():
        body = _capability_section_body(migrated, title)
        assert f"Root WI: {wi}\n" in body, (
            f"relocated section {title!r} lost its tracker state; "
            f"expected 'Root WI: {wi}', section was:\n{body}"
        )


#: Per-capability work-root WIs for the fallback shape below. Distinct per work
#: root, not merely per capability, because the fallback takes the *first*
#: non-empty one and `lexical-search` has two.
WORK_ROOT_WI = {
    work_root: f"#{70 + index}"
    for index, work_root in enumerate(
        work_root for member in _ALL_MEMBERS for work_root in member[4]
    )
}
assert len(set(WORK_ROOT_WI.values())) == len(WORK_ROOT_WI)
#: The WI each capability must end up with: its first work root's.
FIRST_WORK_ROOT_WI = {
    member[0]: WORK_ROOT_WI[member[4][0]] for member in _ALL_MEMBERS
}
#: The WIs that must appear nowhere, because they belong to a second work root
#: that the fallback is required to pass over.
SHADOWED_WORK_ROOT_WIS = tuple(
    WORK_ROOT_WI[work_root]
    for member in _ALL_MEMBERS
    for work_root in member[4][1:]
)
assert SHADOWED_WORK_ROOT_WIS, (
    "at least one member must carry a second work root, or 'the first WI wins' "
    "is indistinguishable from 'any WI wins'"
)


def _work_root_wi_readme() -> str:
    """The section README with every `Root WI` blank and the work roots numbered.

    `root_wi_for_capability` is two branches: the declared `Root WI:` field, and
    -- only when that is absent or `-` -- the first non-empty `active_wi` among
    the capability's work roots. Every other relocation input here declares its
    own `Root WI`, so the second branch is unreachable from them and an
    implementation that deleted it entirely renders identically.
    """
    document = UNCLASSIFIED_SECTION_README
    for wi in SECTION_RELOCATION_WI.values():
        document = document.replace(f"Root WI: {wi}", "Root WI: -")
    assert "Root WI: #" not in document, document
    for work_root, wi in WORK_ROOT_WI.items():
        marker = f"| {work_root} | change | - |"
        assert document.count(marker) == 1, (work_root, document.count(marker))
        document = document.replace(marker, f"| {work_root} | change | {wi} |")
    return document


WORK_ROOT_WI_SECTION_README = _work_root_wi_readme()


def assert_relocation_falls_back_to_the_first_work_root_wi(migrated: str) -> None:
    """With no declared `Root WI`, the first work root's WI is what renders.

    Both halves are asserted. That the fallback fires at all: every capability
    renders its first work root's WI rather than the `-` it declared, in the
    index column and in the section field alike. And that it takes the *first*
    one: `lexical-search` has two work roots, and the second one's WI must
    appear nowhere in the document -- otherwise "the first non-empty WI wins"
    is indistinguishable from "some WI wins".
    """
    rows = _index_rows_parsed(migrated)
    assert [row[0] for row in rows] == list(UNCLASSIFIED_SECTION_TITLES), (
        f"the relocated index must list every capability in document order, "
        f"got {[row[0] for row in rows]}"
    )
    for row in rows:
        expected = FIRST_WORK_ROOT_WI[row[0]]
        assert row[1] == expected, (
            f"relocated index row {row[0]!r} did not fall back to its first "
            f"work root; expected {expected!r}, got {row[1]!r}"
        )
    for title, wi in FIRST_WORK_ROOT_WI.items():
        body = _capability_section_body(migrated, title)
        assert f"Root WI: {wi}\n" in body, (
            f"relocated section {title!r} did not fall back to its first work "
            f"root; expected 'Root WI: {wi}', section was:\n{body}"
        )
    for shadowed in SHADOWED_WORK_ROOT_WIS:
        assert f"Root WI: {shadowed}" not in migrated, (
            f"{shadowed} belongs to a second work root and must not be chosen "
            f"as any capability's Root WI"
        )


def assert_relocation_renders_every_capability_section(
    migrated: str, report: dict[str, Any], *, expected_order: tuple[str, ...]
) -> None:
    """Relocation emits a section for every capability, not just an index.

    The index and the capability sections are rendered by separate passes, and
    the pass that renders *unclassified* capabilities is a separate loop again.
    A relocation that dropped the sections would still emit a complete-looking
    Capability Index, so the resulting document reads as populated while
    carrying no contract at all -- and the follow-up report parses zero
    capabilities.

    Asserted through both the rendered document and a re-report of it, because
    either alone is satisfiable: the text could contain the headings while the
    document fails to parse, and the report could be right about a document this
    fixture never actually read back.
    """
    assert _rendered_capability_titles(migrated) == list(expected_order), (
        f"relocation must render one section per capability, got "
        f"{_rendered_capability_titles(migrated)}"
    )
    parsed = [item["id"] for item in report["capabilities"]]
    expected_ids = [member[1] for member in _ALL_MEMBERS]
    assert sorted(parsed) == sorted(expected_ids), parsed
    assert report["capability_count"] == len(expected_ids), report
    assert document_blockers(report) == [], report["blockers"]


def assert_relocation_renders_the_three_groups_in_order(migrated: str) -> None:
    """Relocation renders core, then non-core, then whatever declared nothing.

    `capabilities_in_render_order` (`capability.rs:8908-8932`) is the single
    array that decides this, and both the Capability Index and the capability
    sections are rendered from it by separate passes. Every other relocation
    shape in this fixture leaves at most two of its three groups non-empty and
    in an order that coincides with raw document order, so permuting the array
    renders the byte-identical document -- the order is simply not observable
    from them. Here the three groups are populated and interleaved, so a
    permuted array renders a different document.

    Three things are asserted, because no two of them imply the third. The
    index and the sections agree, so a permutation that moved only one of the
    two passes leaves a document whose index permanently contradicts its own
    body. The sections are pinned to the grouped order, so both passes moving
    together is caught as well. And the grouped order is asserted at import to
    differ from raw input order under every permutation of the three groups, so
    "agrees with the sections" cannot be satisfied by both halves collapsing
    back to input order.
    """
    index_titles = _index_titles(migrated)
    rendered = _rendered_capability_titles(migrated)

    assert index_titles == rendered, (
        f"the relocated index and its capability sections disagree on order; "
        f"index={index_titles} sections={rendered}"
    )
    assert rendered == list(MIXED_SECTION_GROUPED_TITLES), (
        f"relocation must render core, then non-core, then unclassified; "
        f"got {rendered}"
    )


def assert_relocation_root_emission(
    migrated: str, *, declares_any_class: bool
) -> None:
    """Relocation emits the two roots exactly when the input classifies anything.

    `render_capability_registry` guards the whole two-root block on
    `document.capabilities.iter().any(|c| c.feature_class.is_some())`
    (`capability.rs:8619-8623`). Emitting the roots unconditionally would put a
    fully unclassified contract under `### Core Features` / `### Non-Core
    Features` headings it never claimed -- a silent classification -- and
    emitting them never would produce a document its own checker rejects for a
    missing root.

    Both directions are asserted, from the two relocation shapes that differ in
    exactly this input property: nothing classified, and the domain promises
    classified. Asserting only the positive direction would pass for a renderer
    that always emits both roots, which is the mutation this exists to kill.
    """
    for class_heading in ("### Core Features", "### Non-Core Features"):
        present = class_heading in migrated
        assert present == declares_any_class, (
            f"{class_heading} present={present} on a relocation whose input "
            f"declares_any_class={declares_any_class}"
        )


def assert_relocation_emits_both_roots_when_one_is_empty(
    migrated: str, report: dict[str, Any]
) -> None:
    """Both canonical roots survive relocation even when one has no members.

    Once any capability is classified the document is committed to the two-root
    shape, and a document missing a root is rejected by the checker. So the
    contract is not "emit the roots that have members" but "emit both", and the
    only input that can tell those apart is one where a class is empty.

    The re-report is what makes this more than a substring check: it asserts the
    emitted document is one the product itself accepts.
    """
    for root in ("### Core Features", "### Non-Core Features"):
        assert root in migrated, (
            f"relocation dropped {root!r} on a document with no members in that "
            f"class; migrated document was:\n{migrated}"
        )
    assert migrated.index("### Core Features") < migrated.index("### Non-Core Features")
    assert report["core_capability_count"] == len(_CORE_MEMBERS), report
    assert report["non_core_capability_count"] == 0, report
    assert document_blockers(report) == [], report["blockers"]


def assert_next_coverage_matches_the_report(
    coverage: dict[str, Any], report: dict[str, Any]
) -> None:
    """`aw capability next` renders the same split its report computed.

    A third rendering of the split, built by its own JSON literal rather than by
    the report serializer, so it can zero or transpose a field while every
    report-reading leg in this fixture stays green -- the same argument the
    `--human` leg makes, applied to the surface it also applies to.

    Honest limitation, stated because it bounds what this leg proves: `aw
    capability next` has no `--verify`, so the four verified operands are zero on
    both sides and their equality here is real but not falsifiable. The four
    populated operands carry the non-vacuity, and are guarded as distinct so a
    transposition among them cannot pass.
    """
    populated = (
        "core_capability_count",
        "non_core_capability_count",
        "core_claim_count",
        "non_core_claim_count",
    )
    verified = (
        "core_verified_count",
        "non_core_verified_count",
        "core_verified_claim_count",
        "non_core_verified_claim_count",
    )
    for field in populated + verified:
        assert coverage[field] == report[field], (
            f"`aw capability next` coverage disagrees with the report on "
            f"{field}: {coverage[field]!r} vs {report[field]!r}"
        )
    for field in populated:
        assert coverage[field] > 0, (
            f"{field} must be non-zero here, or its equality is 0 == 0 and a "
            f"coverage summary that zeroed it would pass"
        )
    assert coverage["core_capability_count"] != coverage["non_core_capability_count"]
    assert coverage["core_claim_count"] != coverage["non_core_claim_count"]
    # And the pairs still exhaust the totals *this surface* reports, so the split
    # cannot be right per field while the summary disagrees with itself.
    assert (
        coverage["core_capability_count"] + coverage["non_core_capability_count"]
        == coverage["capability_count"]
    ), coverage
    assert (
        coverage["core_claim_count"] + coverage["non_core_claim_count"]
        == coverage["claim_count"]
    ), coverage


def baseline_placed_core_document(cap_id: str) -> str:
    """Falsifier: a baseline nested under `Core Features` declaring no class.

    `baseline_declared_core_document` states `Feature Class: core`, so it is
    rejected on the field alone. The effective class is resolved as the declared
    field *or else* the containing root, and this document exercises the second
    half: nothing is declared, and the placement is what makes it core.
    """
    document = baseline_declared_core_document(cap_id)
    marker = f"ID: {cap_id}\nType: Service\nFeature Class: core\n"
    assert document.count(marker) == 1, (
        f"expected exactly one declared-core section for {cap_id}; "
        f"found {document.count(marker)}"
    )
    return document.replace(marker, f"ID: {cap_id}\nType: Service\n")


def assert_baseline_placed_core_is_rejected(
    report: dict[str, Any], cap_id: str
) -> None:
    """Placement alone is enough to classify, and enough to be rejected for it.

    The capability declares no class, so the report attributes it to the
    non-core default -- and it is still rejected, because the rule reads the
    containing root when the field is silent. Both halves are asserted: an
    implementation that only ever read the field would leave `blockers` empty
    here while every other baseline leg in this fixture still passed.
    """
    assert document_blockers(report) == [
        f"trait-derived baseline capability `{cap_id}` is classified `core`; "
        f"archetype baselines are always `non_core` and belong under "
        f"`Non-Core Features`"
    ], report["blockers"]
    by_id = {item["id"]: item for item in report["capabilities"]}
    assert by_id[cap_id].get("feature_class") is None, by_id[cap_id]


def case_varied_root_document(document: str) -> str:
    """The same document with both feature roots written in a different case.

    `feature_root_title` (`capability.rs:9960-9970`) lower-cases the heading
    before testing for the trailing `features` word, so a root is recognized
    however it is capitalized. Every other document in this fixture writes its
    roots in exact canonical case, which cannot tell that tolerance apart from a
    case-sensitive test: narrowing the check to `ends_with("Features")` leaves
    every one of them rendering and reporting identically.

    What makes the narrowing consequential rather than cosmetic is that an
    unrecognized root does not misclassify -- it disappears. The effective class
    is the declared field *or else* the containing root, so a capability that
    declared nothing under an unrecognized root resolves to no class at all,
    falls to the non-core default, and the rule that would have rejected it goes
    silent. A human who writes `### CORE FEATURES` gets a document that reads as
    classified and is checked as if it were not.
    """
    varied = document
    for canonical, upper in (
        ("### Core Features", "### CORE FEATURES"),
        ("### Non-Core Features", "### NON-CORE FEATURES"),
    ):
        assert document.count(canonical) == 1, (
            f"expected exactly one {canonical!r} heading to vary; "
            f"found {document.count(canonical)}"
        )
        varied = varied.replace(canonical, upper)
    assert varied != document
    return varied


def assert_case_varied_roots_are_read_like_canonical_ones(
    varied: dict[str, Any], canonical: dict[str, Any], cap_id: str
) -> None:
    """Case-varied roots produce the same verdict and the same split.

    Asserted against the canonical document's own report rather than against a
    literal, so this cannot drift away from what the canonical leg established,
    and asserted on the blocker *and* on every class-partitioned count: the
    blocker alone would pass an implementation that stopped recognizing the
    non-core root, since an unrecognized root and the non-core default look the
    same from a single rejected capability.
    """
    assert_baseline_placed_core_is_rejected(varied, cap_id)
    for field in (
        "capability_count",
        "claim_count",
        "core_capability_count",
        "non_core_capability_count",
        "core_claim_count",
        "non_core_claim_count",
    ):
        assert varied[field] == canonical[field], (
            f"case-varied roots changed {field}: {varied[field]!r} vs the "
            f"canonical document's {canonical[field]!r}"
        )
        assert varied[field] > 0, (
            f"{field} must be non-zero here, or its equality is 0 == 0 and a "
            f"report that zeroed the whole split would pass"
        )


#: The `Feature Class` spellings a human writes. The parser normalizes by
#: trimming backticks, case-folding, mapping `-` and space to `_`, and then
#: stripping a trailing `_features` -- so the *root headings themselves*
#: (`Core Features`, `Non-Core Features`) are accepted field values, which is
#: how an author who copies the heading into the field gets away with it.
#:
#: An earlier revision listed only four non-core spellings and omitted the whole
#: suffix family, while the case's declared oracle claimed *every* human spelling
#: was asserted. That was an overclaim, and the round-13 review proved it: a
#: mutation that made `parse_feature_class_cell` reject any value ending in
#: `_features` left the gate at exit 0.
CORE_CLASS_SPELLINGS = ("`Core`", "CORE", "Core Features", "`core_features`")
NON_CORE_CLASS_SPELLINGS = (
    "non-core",
    "NonCore",
    "Non-Core",
    "noncore",
    "Non-Core Features",
    "noncore features",
    "`non_core_features`",
    "NON_CORE FEATURES",
)

#: One document cannot exercise more spellings than it has capabilities, so the
#: spellings are partitioned into waves and the case reports each wave. Deriving
#: the wave count rather than hardcoding it means adding a spelling is a
#: one-line change that cannot silently go unexercised.
assert len(CORE_CLASS_SPELLINGS) % len(CORE_IDS) == 0, CORE_CLASS_SPELLINGS
assert len(NON_CORE_CLASS_SPELLINGS) % len(NON_CORE_IDS) == 0, NON_CORE_CLASS_SPELLINGS
HUMAN_SPELLING_WAVES = len(CORE_CLASS_SPELLINGS) // len(CORE_IDS)
assert HUMAN_SPELLING_WAVES == len(NON_CORE_CLASS_SPELLINGS) // len(NON_CORE_IDS), (
    "each wave must consume one spelling per capability in both classes"
)


def human_spelling_document(wave: int) -> str:
    """The reference document with every class restated the way a human types it.

    Each declaration is replaced with a *different* accepted spelling, so a wave
    exercises several distinct spellings in one report rather than one
    representative, and the waves together exercise the whole accepting set.
    Core stays core and non-core stays non-core, so the attribution assertion is
    the same one the canonical document satisfies -- a parser that resolved an
    accepted spelling to the wrong class, or refused it, fails here and nowhere
    else.
    """
    assert 0 <= wave < HUMAN_SPELLING_WAVES, wave
    document = REFERENCE_DOCUMENT
    assert document.count("Feature Class: core") == len(CORE_IDS)
    assert document.count("Feature Class: non_core") == len(NON_CORE_IDS)
    for spellings, ids, canonical in (
        (CORE_CLASS_SPELLINGS, CORE_IDS, "core"),
        (NON_CORE_CLASS_SPELLINGS, NON_CORE_IDS, "non_core"),
    ):
        start = wave * len(ids)
        for spelling in spellings[start : start + len(ids)]:
            document = document.replace(
                f"Feature Class: {canonical}", f"Feature Class: {spelling}", 1
            )
    assert "Feature Class: core\n" not in document
    assert "Feature Class: non_core\n" not in document
    return document


def assert_human_class_spellings_are_accepted(report: dict[str, Any]) -> None:
    """Every accepted spelling resolves to its canonical class with no blocker.

    Reusing the canonical attribution assertion is the point: the document says
    the same thing as `REFERENCE_DOCUMENT` in different words, so it must report
    identically, counts included. Refusing a spelling would fail the command
    outright, and resolving one to the wrong class would contradict the
    containing root and raise a blocker -- neither can pass quietly.
    """
    assert_feature_class_attribution(report)


def _field_less_reference() -> str:
    """`REFERENCE_DOCUMENT` with every `Feature Class` field deleted.

    Not blanked -- deleted. A blank field is an author who declared nothing in
    particular; a missing field is an author who has not begun classifying, and
    only the second reaches the rules below.
    """
    document = REFERENCE_DOCUMENT
    for line in ("Feature Class: core\n", "Feature Class: non_core\n"):
        assert line in document
        document = document.replace(line, "")
    assert "Feature Class:" not in document
    return document


#: A baseline moved under `Core Features` in a document that declares no class
#: field anywhere. `baseline_placed_core_document` covers placement while *other*
#: capabilities still declare fields, so the document is classified either way.
#: Here the roots are the only evidence of classification that exists.
_PLACED_BASELINE_MEMBER = _NON_CORE_MEMBERS[2]
PLACED_BASELINE_ID = _PLACED_BASELINE_MEMBER[1]


def _roots_only_document() -> str:
    document = _field_less_reference()
    section = _section(_PLACED_BASELINE_MEMBER, None, "####")
    assert document.count(section) == 1, document.count(section)
    document = document.replace(section, "")
    return document.replace("### Non-Core Features", section + "### Non-Core Features")


ROOTS_ONLY_DOCUMENT = _roots_only_document()

#: The same field-less document with both canonical roots renamed out of the
#: closed pair, so the *only* evidence that classification has begun is a pair of
#: unknown roots.
UNKNOWN_ROOTS_ONLY_DOCUMENT = (
    _field_less_reference()
    .replace("### Core Features", "### Optional Features")
    .replace("### Non-Core Features", "### Legacy Features")
)


def assert_roots_alone_classify_the_document(report: dict[str, Any]) -> None:
    """Feature roots are evidence of classification even with no field anywhere.

    `validate_capability_feature_roots` returns early unless the document
    "declares any class", and that test is a three-way disjunction: any
    capability carrying a field, **or** any canonical root present, **or** any
    unknown root present. Every other document in this fixture satisfies the
    first arm, which masks the other two entirely -- deleting the root arm left
    the whole case green.

    Here no capability declares anything, so the roots carry the whole decision:
    a baseline sits under `Core Features` and must be rejected for it. The
    per-capability assertion is the other half -- every capability is still
    *attributed* to the non-core default, because placement classifies the
    document without populating the field.
    """
    assert document_blockers(report) == [
        f"trait-derived baseline capability `{PLACED_BASELINE_ID}` is classified "
        f"`core`; archetype baselines are always `non_core` and belong under "
        f"`Non-Core Features`"
    ], report["blockers"]
    classes = {item["id"]: item.get("feature_class") for item in report["capabilities"]}
    assert set(classes.values()) == {None}, classes
    assert report["core_capability_count"] == 0, report
    assert report["non_core_capability_count"] == len(CORE_IDS) + len(NON_CORE_IDS), report


def assert_unknown_roots_alone_classify_the_document(report: dict[str, Any]) -> None:
    """An unknown root is evidence of classification too, and is diagnosed as one.

    The third arm of the same disjunction. With no field and no canonical root,
    a document carrying only unknown roots must still be diagnosed rather than
    waved through as pre-migration -- an author who invented their own root
    names has begun classifying and got it wrong, which is exactly the case the
    early return is not meant to cover.

    The whole ordered blocker set is asserted, not a membership test: this shape
    raises both missing-root findings *and* both unknown-root findings, and an
    implementation that reported only one kind would pass a weaker assertion.
    """
    assert document_blockers(report) == [
        "capability document declares feature classes but is missing the "
        "`### Core Features` root; add it under `## Capabilities` so both "
        "canonical roots exist",
        "capability document declares feature classes but is missing the "
        "`### Non-Core Features` root; add it under `## Capabilities` so both "
        "canonical roots exist",
        "unknown feature root `Optional Features`; the closed pair is "
        "`Core Features` and `Non-Core Features` — move its capabilities "
        "under one of those two",
        "unknown feature root `Legacy Features`; the closed pair is "
        "`Core Features` and `Non-Core Features` — move its capabilities "
        "under one of those two",
    ], report["blockers"]


#: The reference document with a non-root heading at the feature roots' own
#: level, carrying a nested heading that repeats a capability title. A root must
#: be closed by any heading at or above its level; if it were closed only by a
#: strictly shallower one, `Search Core` would be read as a member of the
#: non-core root as well as the core root and the document would be falsely
#: rejected for being under both.
#: The reference document plus an appendix that *shows* the canonical shape
#: inside a fenced code block. `scan_feature_roots` masks fenced lines
#: (`capability.rs:9979-9981`) before it parses headings, so nothing in the
#: fence is a root or a member. Both a root heading and a capability heading are
#: fenced, and the fenced root is the *non-core* one carrying the *core*
#: document's first capability, so dropping the mask breaks two rules at once
#: rather than one: the root is then declared twice, and `search-core` then sits
#: under both roots.
#:
#: This is not hypothetical repo hygiene. `apps/lumen/README.md` -- the file
#: this fixture digests before and after every relocation leg -- carries
#: heading-shaped lines inside fences, as do the CLI template README and
#: several other project READMEs.
FENCED_ROOT_DOCUMENT = (
    REFERENCE_DOCUMENT
    + """
## Appendix

The canonical shape this document follows:

```markdown
### Non-Core Features

#### Search Core

ID: search-core
Feature Class: non_core
```
"""
)
assert FENCED_ROOT_DOCUMENT.count("### Non-Core Features") == 2
assert FENCED_ROOT_DOCUMENT.count("#### Search Core") == 2


def assert_fenced_headings_are_not_read_as_structure(report: dict[str, Any]) -> None:
    """A heading inside a code fence is documentation, not classification.

    Asserted through the blockers *and* the counts. Without the fenced-line
    mask this document raises a duplicate-root finding and a both-roots finding
    for `search-core`, so an empty blocker list is only reachable when the mask
    holds -- and the counts are pinned alongside so that a mask which dropped
    the whole appendix (rather than only its heading lines) is distinguishable
    from one that read it correctly.
    """
    assert document_blockers(report) == [], report["blockers"]
    assert report["core_capability_count"] == len(CORE_IDS), report
    assert report["non_core_capability_count"] == len(NON_CORE_IDS), report
    assert report["core_claim_count"] == CORE_CLAIM_COUNT, report
    assert report["non_core_claim_count"] == NON_CORE_CLAIM_COUNT, report


SIBLING_HEADING_DOCUMENT = (
    REFERENCE_DOCUMENT + "\n\n### Appendix\n\n#### Search Core\n\nSee above.\n"
)


def assert_sibling_heading_closes_the_root(report: dict[str, Any]) -> None:
    """A heading at the root's own level ends that root's membership scope.

    This is the only assertion in the case that is falsified by an
    *over*-reporting implementation rather than an under-reporting one, which is
    why it exists: every other document here is asserted against the blockers it
    should raise, so a containment rule that swallowed too much of the document
    would go unnoticed. Tightening the scope test by one comparison makes this
    document raise a capability-under-both-roots finding that is simply untrue.
    """
    assert document_blockers(report) == [], report["blockers"]
    assert_feature_class_attribution(report)


#: The three capability-contract *reading* forms other than the canonical
#: field-style section. Each has its own `Feature Class` parser, and binding one
#: binds none of the others -- the round-13 review killed all three
#: independently while the case stayed green.
ALTERNATE_FORM_MEMBERS = (
    ("Search Core", "search-core", "core", "query-planner-boolean-eval"),
    ("Lexical Search", "lexical-search", "core", "bm25-ranking"),
    ("Security Hardening", "security-hardening", "non_core", "transport-and-identity-hardening"),
)
#: Deliberately unequal, so a form whose parser transposed the two classes fails.
ALTERNATE_FORM_CORE_COUNT = 2
ALTERNATE_FORM_NON_CORE_COUNT = 1
assert ALTERNATE_FORM_CORE_COUNT != ALTERNATE_FORM_NON_CORE_COUNT
assert ALTERNATE_FORM_CORE_COUNT == sum(
    1 for member in ALTERNATE_FORM_MEMBERS if member[2] == "core"
)
assert ALTERNATE_FORM_NON_CORE_COUNT == sum(
    1 for member in ALTERNATE_FORM_MEMBERS if member[2] == "non_core"
)


def _alternate_form_document(render: Any) -> str:
    index = "\n".join(
        f"| {title} | - | implemented | verified | smoke | ready | verified; form |"
        for title, _cap_id, _cls, _root in ALTERNATE_FORM_MEMBERS
    )
    core = "".join(render(*m) for m in ALTERNATE_FORM_MEMBERS if m[2] == "core")
    non_core = "".join(render(*m) for m in ALTERNATE_FORM_MEMBERS if m[2] == "non_core")
    return f"""# Lumen

## Brief

Lumen reference fixture rendered in an alternative capability-contract form.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
{index}

### Core Features

{core}### Non-Core Features

{non_core}"""


def _field_value_contract(title: str, cap_id: str, feature_class: str, work_root: str) -> str:
    return f"""#### {title}

| Field | Value |
|---|---|
| ID | {cap_id} |
| Type | Service |
| Feature Class | {feature_class} |
| Status | verified |
| Root WI | - |
| Required Verification | smoke |
| Promise | Promise for {cap_id}. |
| Surfaces | - CLI: `lumen serve` - {cap_id}. |
| EC Dimensions | - behavior: `true` - {cap_id} behavior gate. |
| Gate Inventory | - tech-design/{cap_id}.md |

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| {work_root} | change | - | implemented | verified | smoke | `true` |

"""


def _contract_table(title: str, cap_id: str, feature_class: str, work_root: str) -> str:
    return f"""#### {title}

| ID | Root WI | Status | Type | Feature Class | Surfaces | EC Dimensions | Promise | Required Verification | Gate Inventory |
|---|---:|---|---|---|---|---|---|---|---|
| {cap_id} | - | verified | Service | {feature_class} | - CLI: `lumen serve` - {cap_id}. | - behavior: `true` - gate. | Promise for {cap_id}. | smoke | - tech-design/{cap_id}.md |

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| {work_root} | change | - | implemented | verified | smoke | `true` |

"""


#: `| Field | Value |` contracts: the class is read by `value_for`, through the
#: alias set `featureclass` / `capabilityfeatureclass` / `featureroot`.
FIELD_VALUE_CONTRACT_DOCUMENT = _alternate_form_document(_field_value_contract)

#: One-row contract tables: the class is read by `find_table_column`, a
#: different lookup over the same alias set.
CONTRACT_TABLE_DOCUMENT = _alternate_form_document(_contract_table)


def _yaml_contract(title: str, cap_id: str, feature_class: str) -> str:
    return f"""## Capability: {title}
<!-- type: capability lang: yaml -->

```yaml
id: {cap_id}
status: candidate
feature_class: {feature_class}
capability_type: Service
promise: "Promise for {cap_id}."
current_state: "Implemented."
surfaces:
  - kind: CLI
    commands: ["lumen serve"]
    summary: "{cap_id} surface."
ec_dimensions:
  - dimension: behavior
    gate: "true"
    summary: "{cap_id} behavior gate."
```

"""


#: YAML-fenced contracts: the class is carried straight through from the
#: deserialized struct. These sections are `## Capability: <Title>` headings,
#: which sit *outside* both feature roots -- so for this form the declared field
#: is the only thing that can classify a capability at all, with no placement to
#: fall back on.
YAML_CONTRACT_DOCUMENT = (
    "# Lumen\n\n## Brief\n\nLumen reference fixture in YAML-fenced contract "
    "form.\n\n## Capabilities\n\n### Core Features\n\n### Non-Core Features\n\n"
    + "".join(
        _yaml_contract(title, cap_id, feature_class)
        for title, cap_id, feature_class, _root in ALTERNATE_FORM_MEMBERS
    )
)

#: The one finding the YAML form legitimately raises: it is a pre-canonical
#: authoring shape, and saying so is not a classification failure.
YAML_CONTRACT_BLOCKER = (
    "YAML capability sections detected; migrate README to canonical field-style "
    "capability contracts under ## Capabilities"
)


def assert_alternate_form_reads_the_class(
    report: dict[str, Any], *, form: str, expected_blockers: list[str]
) -> None:
    """The `Feature Class` declaration is honoured in this contract-reading form.

    The capability's promise is that canonical capability contracts parse from
    Markdown, and the product ships four reading forms for them. Binding the
    field-style section binds none of the other three: each has its own class
    lookup, and each was independently mutable to "never read the class" with
    the whole case still green.

    Asserted per capability *and* per class count. The per-capability check
    catches a form that dropped the field; the counts catch one that read it
    into the wrong class, and they are unequal so a transposition cannot pass.
    """
    assert document_blockers(report) == expected_blockers, (form, report["blockers"])
    classes = {item["id"]: item.get("feature_class") for item in report["capabilities"]}
    assert classes == {
        cap_id: feature_class
        for _title, cap_id, feature_class, _root in ALTERNATE_FORM_MEMBERS
    }, (form, classes)
    assert report["core_capability_count"] == ALTERNATE_FORM_CORE_COUNT, (form, report)
    assert report["non_core_capability_count"] == ALTERNATE_FORM_NON_CORE_COUNT, (
        form,
        report,
    )


def assert_verified_split_is_non_degenerate(report: dict[str, Any]) -> None:
    """The verified halves of the split are attributed per class, not just summed.

    The pair-sum assertions in `assert_feature_class_attribution` run against a
    report with no gate execution, where all four verified fields are zero. Every
    verified-dimension assertion there is therefore `0 + 0 == 0`: true of the
    correct implementation, and equally true of one that always writes zero or
    rolls every verified capability into a single class. Half the split's fields
    were being asserted vacuously.

    This runs the same document under `--verify`, where the counts are populated
    and the two classes differ in both dimensions, and pins each field
    individually. The distinctness guard is what makes "individually" mean
    something: if core and non-core happened to agree, a transposed or duplicated
    field would satisfy every equality here.
    """
    assert report["core_verified_count"] == len(CORE_IDS), report
    assert report["non_core_verified_count"] == len(NON_CORE_IDS), report
    assert report["core_verified_claim_count"] == CORE_CLAIM_COUNT, report
    assert report["non_core_verified_claim_count"] == NON_CORE_CLAIM_COUNT, report

    # Non-vacuity: the operands this leg pins must not be interchangeable.
    assert report["core_verified_count"] != report["non_core_verified_count"], report
    assert (
        report["core_verified_claim_count"] != report["non_core_verified_claim_count"]
    ), report

    # And they must still exhaust the totals, so the per-class figures cannot be
    # right while some verified capability falls out of both classes.
    assert (
        report["core_verified_count"] + report["non_core_verified_count"]
        == report["verified_count"]
    ), report
    assert (
        report["core_verified_claim_count"] + report["non_core_verified_claim_count"]
        == report["verified_claim_count"]
    ), report
    assert report["verified_count"] == report["capability_count"], report
    assert report["verified_claim_count"] == report["claim_count"], report


def assert_partial_verification_is_attributed_per_class(
    report: dict[str, Any],
    name: str,
    failing_capabilities: tuple[str, ...],
    operands: tuple[int, ...],
) -> None:
    """An unverified claim is subtracted from its own class, not from the other.

    The leg above reads a document where everything verifies, so it can only
    check that the verified counts equal the totals -- an implementation that
    attributed an unverified capability to the wrong class has nothing to get
    wrong there. Here two classes are each short by a different amount, and the
    eight operands are pinned to exact integers, so a claim charged to the wrong
    class moves two of them.
    """
    for key, value in zip(READINESS_OPERAND_KEYS, operands, strict=True):
        assert report[key] == value, (name, key, report[key], value, report)

    # The per-class figures must still exhaust the report-level ones, so a
    # capability cannot go unverified in its class and verified in the total.
    assert (
        report["core_verified_count"] + report["non_core_verified_count"]
        == report["verified_count"]
    ), report
    assert (
        report["core_verified_claim_count"] + report["non_core_verified_claim_count"]
        == report["verified_claim_count"]
    ), report
    # Non-vacuity for those two sums: unlike the fully verified leg, the totals
    # here are strictly larger, so neither sum can be satisfied by an
    # implementation that simply copies the totals across.
    assert report["verified_count"] < report["capability_count"], report
    assert report["verified_claim_count"] < report["claim_count"], report

    # And the failure is reported against the capability that owns the claim,
    # one message per unverified claim in document order. Without this, the
    # counts above could be right while the diagnostic named another capability.
    assert document_blockers(report) == [
        f"verification failed for {cap_id}: false" for cap_id in failing_capabilities
    ], (name, report["blockers"])
