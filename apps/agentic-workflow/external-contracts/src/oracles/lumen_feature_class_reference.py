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

What this fixture does *not* assert, so it is not read as proving more than it
does: `validate_capability_feature_roots` also rejects a missing root, an unknown
root, and a capability nested under both roots. None of the three is falsified
here. They cannot be added as single-message falsifiers -- deleting
`### Non-Core Features` yields the missing-root finding plus one field/root
contradiction per capability stranded under the surviving root, and renaming it
yields unknown-root plus missing-root -- so they need co-occurring-set assertions
and belong in their own slice. `document_blockers` is nonetheless total, so if
any of the three fires on a document this fixture *does* run, the run fails.
"""

from __future__ import annotations

import hashlib
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
#: Claim counts are deliberately unequal per class (3 core, 4 non-core) so no
#: assertion can pass by pairing a core count with a non-core total.
CORE_CLAIM_COUNT = 3
NON_CORE_CLAIM_COUNT = 4

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
Status: verified
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
    (
        "EC Gates Configured",
        "ec-gates-configured",
        "Carry configured external-contract gates.",
        "lumen verify",
        ("gate-configuration",),
    ),
)

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
) -> str:
    index = "\n".join(
        (
            _index_rows(core, "domain promise"),
            _index_rows(non_core, "archetype service baseline"),
        )
    )
    core_body = "\n".join(_section(member, core_class) for member in core)
    non_core_body = "\n".join(_section(member, non_core_class) for member in non_core)
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


def baseline_declared_core_document(cap_id: str) -> str:
    """Falsifier 1, per baseline: promote one baseline into the core root.

    Generated per id rather than fixed, so "every archetype service baseline is
    non-core" is asserted for every baseline the fixture names instead of for
    one representative.
    """
    promoted = next(member for member in _NON_CORE_MEMBERS if member[1] == cap_id)
    remaining = tuple(member for member in _NON_CORE_MEMBERS if member[1] != cap_id)
    return _document(_CORE_MEMBERS + (promoted,), remaining)


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
    assert len(environment) == len(_ENVIRONMENT_BLOCKER_PREFIXES), (
        f"scratch-environment blockers changed shape; expected one per "
        f"{_ENVIRONMENT_BLOCKER_PREFIXES}, got {environment}"
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
        block = migrated[migrated.find(f"ID: {cap_id}\n") :]
        line = next(
            stripped
            for stripped in (raw.strip() for raw in block.splitlines())
            if stripped.startswith("Feature Class:")
        )
        assert line == f"Feature Class: {expected}", f"{cap_id}: {line}"
