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

#: The capability fields an author may legally omit, each guarded by its own
#: emptiness check inside `render_markdown_capability_section_at_level`.
#:
#: Named as a closed set rather than left implicit, so that adding a fifth
#: optional field to the product without extending the fixture is a mismatch a
#: reader can see rather than a silently unbound guard.
WITHHOLDABLE_FIELDS = frozenset(
    {"type", "surfaces", "ec_dimensions", "required_verification"}
)

#: `CapabilityStatus` (`capability.rs:453-460`), restated as a closed set.
#:
#: Restated rather than sampled: two of the six columns of the Capability Index
#: are derived from the status by a `match` over this enum, so a fixture that
#: declares four of the six leaves two arms of each derivation unreached and
#: freely rewritable. The set is what lets the varied-status document below
#: assert that it walks the enum instead of asserting that it declares several
#: values.
CAPABILITY_STATUSES = frozenset(
    {"candidate", "confirmed", "auditing", "blocked", "verified", "retired"}
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
    work_root_cells: dict[str, tuple[str, str, str, str, str]] | None = None,
    multi_item: bool = False,
    withheld: frozenset[str] = frozenset(),
) -> str:
    assert withheld <= WITHHOLDABLE_FIELDS, sorted(withheld - WITHHOLDABLE_FIELDS)
    if work_root_cells is None:
        rows = "\n".join(
            f"| {root} | change | - | implemented | verified | smoke | `true` |"
            for root in work_roots
        )
    else:
        rows = "\n".join(
            "| {root} | {kind} | - | {impl} | {verification} | {maturity} | {gate} |".format(
                root=root,
                kind=work_root_cells[root][0],
                impl=work_root_cells[root][1],
                verification=work_root_cells[root][2],
                maturity=work_root_cells[root][3],
                gate=work_root_cells[root][4],
            )
            for root in work_roots
        )
    # `feature_class=None` is the pre-migration shape: no field at all. Emitting
    # an empty field instead would be a different document -- an author who
    # declared nothing, versus one who declared a blank.
    class_field = "" if feature_class is None else f"Feature Class: {feature_class}\n"
    # Every other document declares exactly one item per list, which leaves an
    # implementation that renders only the first element of each byte-identical
    # to one that renders all of them.
    surface_items = [_member_surface_item(title, surface, promise)]
    dimension_items = [_member_ec_dimension_item(title, cap_id)]
    gate_items = [f"tech-design/{cap_id}.md"]
    if multi_item:
        surface_items.append(MULTI_ITEM_SURFACE_ITEM)
        # Prepended rather than appended, because EC dimensions are the one list
        # field whose rendered order is *not* the declared order:
        # `dedupe_ec_dimensions` collapses them through a `BTreeMap` keyed by
        # kind, so they come back in `CapabilityEcDimensionKind` order. This
        # member's own dimension is `behavior` and its second is `security`,
        # which is already that order, so appending left the sort and the
        # declared order rendering the identical document.
        dimension_items.insert(0, MULTI_ITEM_EC_DIMENSION_ITEM)
        gate_items.append(MULTI_ITEM_GATE_INVENTORY_ITEM)
    surfaces_field = "".join(f"- {item}\n" for item in surface_items)
    dimensions_field = "".join(f"- {item}\n" for item in dimension_items)
    # No field at all, not an empty one -- the same distinction `class_field`
    # draws above.
    #
    # Four of a capability's fields are optional at the input, and
    # `render_markdown_capability_section_at_level` guards each of them on its
    # own emptiness check. Every capability of every document here declared all
    # four, so all four guards were free in the direction of absence: forcing
    # any one of them to `true` rendered the identical document. They are
    # withheld through one mechanism rather than four flags because the defect
    # that produced this fix was a fix applied to `Surfaces:` alone while its
    # three siblings in the same block stayed unbound -- a per-field flag is the
    # shape that let the other three be forgotten.
    type_block = "" if "type" in withheld else f"Type: {_member_type(title)}\n"
    surfaces_block = "" if "surfaces" in withheld else f"Surfaces:\n{surfaces_field}"
    dimensions_block = (
        "" if "ec_dimensions" in withheld else f"EC Dimensions:\n{dimensions_field}"
    )
    required_block = (
        ""
        if "required_verification" in withheld
        else f"Required Verification: {_member_required_verification(title)}\n"
    )
    gates_field = "".join(f"- {item}\n" for item in gate_items)
    dependencies = _member_dependencies(title)
    dependencies_field = (
        ""
        if not dependencies
        else "Dependencies:\n" + "".join(f"- {dep}\n" for dep in dependencies)
    )
    return f"""{heading} {title}

ID: {cap_id}
{type_block}{class_field}{surfaces_block}{dimensions_block}{dependencies_field}Root WI: -
Status: {status}
{required_block}Promise:
{promise}
Gate Inventory:
{gates_field}
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
    # This member's promise deliberately contains a `|` *and* spans two lines.
    # `markdown_cell` performs two substitutions and this is the only cell that
    # reaches either: the pipe escape was bound from the start, the newline fold
    # beside it was not, and deleting `.replace('\n', "<br>")` rendered this
    # whole fixture byte for byte. The two lines are not a contrivance --
    # `append_markdown_contract_field_value` (`capability.rs:10957-10959`) joins
    # continuation lines with a newline for `promise` and with `<br>` for every
    # other field, so a multi-line promise is the one value that can carry a raw
    # newline into a table cell at all. Unfolded, the Notes cell breaks its row
    # in half and the index stops parsing.
    #
    # It is the one field
    # that reaches `markdown_cell` (`capability.rs:9244-9250`) as free author
    # text: the Capability Index's `Notes` column falls back to the promise when
    # the input carries no index of its own (`capability.rs:8970-8974`), which is
    # every relocation shape here. Without the escape the row gains a column and
    # the index stops parsing, so the escape is what keeps a promise from
    # corrupting the table around it -- and no other fixture cell contains one.
    (
        "Contract Gate Wiring",
        "ec-gates-configured",
        "Carry configured external-contract gates | one inventory entry per gate.\n"
        "Refuse an inventory whose entries and gates disagree.",
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


#: Per-capability `Type:`, keyed by title.
#:
#: Every member used to declare `Type: Service`, which made
#: `render_markdown_capability_section_at_level`'s type field
#: (`capability.rs:9026-9028`) unfalsifiable: a renderer that ignored
#: `capability.capability_type` and printed the literal `Service` produced the
#: byte-identical document. Varying it is safe as well as necessary --
#: `required_ec_dimensions` (`capability_type.rs:116-126`) makes every other
#: variant's production-required dimension set a *subset* of `Service`'s, so no
#: member becomes harder to satisfy than it was.
MEMBER_TYPE = {
    "Search Core": "Service",
    "Lexical Search": "DeveloperTool",
    "Standard Operational Endpoints": "Devops",
    "Kubernetes-Native Deployment": "RuntimeTool",
    "Security Hardening": "SecurityTool",
    "Contract Gate Wiring": "AgentFirst",
    "Observability": "Devops",
}

#: Per-capability `Required Verification:`, keyed by title.
#:
#: Uniformly `smoke` before, which left `capability_maturity_summary`
#: (`capability.rs:9826-9840`) and the field that renders it
#: (`capability.rs:9032-9035`) both satisfiable by a constant. One member
#: declares two maturities, because the field parses as a list
#: (`parse_maturity_list`, `capability.rs:11786-11790`) and a renderer that
#: printed only the first would otherwise be indistinguishable.
#:
#: No member declares the bare `smoke` that `capability_maturity_summary`
#: substitutes when the field is absent. That is deliberate: the varied-status
#: document withholds this field from one capability, and if the withholding
#: capability's declared value were also `smoke`, "the fallback was substituted"
#: and "the declared value was carried" would render the same cell.
MEMBER_REQUIRED_VERIFICATION = {
    "Search Core": "conformance",
    "Lexical Search": "smoke, conformance",
    "Standard Operational Endpoints": "conformance, negative",
    "Kubernetes-Native Deployment": "corpus",
    "Security Hardening": "negative",
    "Contract Gate Wiring": "dogfood",
    "Observability": "smoke",
}

#: Per-capability EC-dimension runner, keyed by title. The runner is its own
#: sub-field of the rendered `EC Dimensions:` item
#: (`render_ec_dimension_field_items`, `capability.rs:9177-9198`); when every
#: member declared `` `true` `` the runner half was a constant even though the
#: summary half differed.
MEMBER_EC_RUNNER = {
    "Search Core": "true",
    "Lexical Search": "lumen-bench --suite ranking",
    "Standard Operational Endpoints": "curl -fsS localhost:8080/readyz",
    "Kubernetes-Native Deployment": "kubectl apply --dry-run=server -k deploy",
    "Security Hardening": "lumen auth --self-test",
    "Contract Gate Wiring": "aw ec verify --project lumen",
    "Observability": "true",
}

#: Per-capability surface *kind*, keyed by title.
#:
#: `render_surface_field_items` (`capability.rs:9154-9175`) rebuilds the item as
#: `kind: commands - summary`, three sub-fields from three separate reads. Every
#: member used to declare `CLI`, so the kind read was satisfiable by the literal
#: `"CLI"` while the varying command text alone kept the *whole item* pairwise
#: distinct -- a guard on the assembled item cannot see a constant sub-field.
MEMBER_SURFACE_KIND = {
    "Search Core": "HTTP",
    "Lexical Search": "CLI",
    "Standard Operational Endpoints": "Probe",
    "Kubernetes-Native Deployment": "Kubernetes",
    "Security Hardening": "Identity",
    "Contract Gate Wiring": "MCP",
    "Observability": "OTLP",
}

#: Per-capability EC dimension *kind*, keyed by title. Same sub-field shape as
#: `MEMBER_SURFACE_KIND`: the rendered item is `dimension: \`runner\` - summary`
#: and every member declared `behavior`, so that read was a constant even after
#: the runner half was varied.
#:
#: `CapabilityEcDimensionKind` (`capability.rs:654-669`) is a closed four-value
#: enum, so six members cannot be pairwise distinct. The guard below asserts the
#: fixture exercises the *whole* vocabulary instead, which is the same rule the
#: work-root enum columns are held to.
MEMBER_EC_DIMENSION = {
    "Search Core": "behavior",
    "Lexical Search": "efficiency",
    "Standard Operational Endpoints": "stability",
    "Kubernetes-Native Deployment": "behavior",
    "Security Hardening": "security",
    "Contract Gate Wiring": "stability",
    "Observability": "behavior",
}
assert {
    MEMBER_EC_DIMENSION[member[0]] for member in _CORE_MEMBERS + _NON_CORE_MEMBERS
} == {"behavior", "efficiency", "security", "stability"}, (
    "MEMBER_EC_DIMENSION must exercise every CapabilityEcDimensionKind, or the "
    "unused arms of the enum's as_str are unfalsifiable"
)

#: Per-capability `Dependencies:`, keyed by title; members absent from this map
#: declare no such field at all.
#:
#: No fixture capability declared one, so the whole `Dependencies:` block
#: (`capability.rs:9055-9060`) could be deleted without failing anything --
#: while the product's own doc comment (`capability.rs:8490-8492`) names product
#: dependencies as carried through untouched, and the only assertion of that was
#: a colocated Rust `--lib` invariant.
#:
#: Every dependency points at a capability in the same class as the one
#: declaring it, so this fixture adds no cross-class dependency edge to
#: whatever the readiness rules make of one.
#: This map is what each member *declares*. What the product renders for it is
#: `_member_rendered_dependencies`, which is not the same list: dependencies are
#: parsed through a `BTreeSet` (`capability.rs:11721-11731`), so they come back
#: sorted and deduplicated whatever order the author wrote them in.
#:
#: One member declares more than one, because `capability.rs:9055-9060` loops the
#: whole vector while a fixture giving every declaring member exactly one leaves
#: `.take(1)` -- a renderer that emits the first dependency and drops the rest --
#: byte-identical. That is the same arity blind spot `Surfaces`, `EC Dimensions`
#: and `Gate Inventory` close on their own document; `Dependencies` was left out
#: of it, and is closed here instead because this field is asserted on every
#: document rather than on one.
#:
#: That member's declaration is deliberately neither sorted nor duplicate-free,
#: so the two canonicalising halves of the parse are separately observable: in
#: sorted order, a `Vec` that preserved the author's order would render the same
#: bytes, and without the repeat, dropping the deduplication would.
MEMBER_DEPENDENCIES = {
    "Lexical Search": ("search-core",),
    "Contract Gate Wiring": (
        "standard-operational-endpoints",
        "security-hardening",
        "standard-operational-endpoints",
    ),
}
_MULTI_DEPENDENCY_DECLARED = MEMBER_DEPENDENCIES["Contract Gate Wiring"]
assert len(set(_MULTI_DEPENDENCY_DECLARED)) == 2, (
    "the member closing the arity gap has to render two distinct dependencies, "
    "or truncating the dependency loop to its first item is unobservable"
)
assert len(_MULTI_DEPENDENCY_DECLARED) > len(set(_MULTI_DEPENDENCY_DECLARED)), (
    "it has to declare one of them twice, or the deduplication half of the "
    "parse renders the same bytes as a plain vector"
)
assert tuple(sorted(set(_MULTI_DEPENDENCY_DECLARED))) != tuple(
    dict.fromkeys(_MULTI_DEPENDENCY_DECLARED)
), (
    "it has to declare them out of sorted order, or the sorting half of the "
    "parse renders the same bytes as the author's own order"
)

#: Per-capability surface *command list*, keyed by title; members absent from
#: this map declare the single command their member tuple names.
#:
#: `render_surface_field_items` (`capability.rs:9153-9172`) joins a surface's
#: whole command vector with `" + "`. Every member declared exactly one command,
#: which leaves both halves of that composition unobservable at once:
#: `.join(" ~~ ")` and `.take(1)` after `.commands.iter()` each render the
#: byte-identical document. The earlier round raised the *item* list to two and
#: left the list *inside* the item at one.
#:
#: The values are Lumen's own: `apps/lumen/README.md:156` declares
#: `` HTTP: `POST /index`, `POST /search` `` for this capability, so the
#: two-command shape is the production input, not an invented one. This
#: repository's own `apps/agentic-workflow/CAPABILITIES.md:39` carries six.
MEMBER_SURFACE_COMMANDS = {
    "Search Core": ("POST /index", "POST /search"),
}

#: Defaults for the synthetic members some fixtures splice in (registry-spanning
#: probes and the like). Those documents are built to falsify a different rule and
#: never have these fields read back, so they keep the historical values.
_DEFAULT_TYPE = "Service"
_DEFAULT_REQUIRED_VERIFICATION = "smoke"
_DEFAULT_EC_RUNNER = "true"
_DEFAULT_SURFACE_KIND = "CLI"
_DEFAULT_EC_DIMENSION = "behavior"


def _member_type(title: str) -> str:
    return MEMBER_TYPE.get(title, _DEFAULT_TYPE)


def _member_required_verification(title: str) -> str:
    return MEMBER_REQUIRED_VERIFICATION.get(title, _DEFAULT_REQUIRED_VERIFICATION)


def _member_ec_runner(title: str) -> str:
    return MEMBER_EC_RUNNER.get(title, _DEFAULT_EC_RUNNER)


def _member_surface_kind(title: str) -> str:
    return MEMBER_SURFACE_KIND.get(title, _DEFAULT_SURFACE_KIND)


def _member_ec_dimension(title: str) -> str:
    return MEMBER_EC_DIMENSION.get(title, _DEFAULT_EC_DIMENSION)


def _member_dependencies(title: str) -> tuple[str, ...]:
    """What the member's `Dependencies:` block declares, in the authored order."""
    return MEMBER_DEPENDENCIES.get(title, ())


def _member_rendered_dependencies(title: str) -> tuple[str, ...]:
    """What the product must render back for it.

    `parse_dependency_list` (`capability.rs:11721-11731`) collects the parsed ids
    into a `BTreeSet` before handing them on, so the rendered block is sorted and
    deduplicated rather than a copy of what the author wrote. Derived here rather
    than written out beside the declaration, so the two cannot drift into an
    expectation that agrees with the fixture only because both were edited.
    """
    return tuple(sorted(set(_member_dependencies(title))))


def _member_surface_commands(title: str, surface: str) -> tuple[str, ...]:
    """The commands a member's single surface declares, in the authored order."""
    return MEMBER_SURFACE_COMMANDS.get(title, (surface,))


def _member_surface_item(title: str, surface: str, promise: str) -> str:
    """The exact `Surfaces:` item a member declares, and must get back.

    Restated here rather than in the fixture text so the authored document and
    the expectation cannot drift apart into an assertion that pins the oracle to
    itself.

    The command half is a *list* joined with `" + "`, which is the renderer's own
    composition (`capability.rs:9153-9160`) and the one member declaring two is
    what makes both the separator and the traversal observable.

    Only the promise's first line is borrowed. One member's promise spans two
    lines so that `markdown_cell`'s newline fold is reachable, and a surface item
    is a single list entry -- carrying the second line in here would put a raw
    newline inside a `- ` item, which is a different parse, not a longer summary.
    """
    commands = " + ".join(
        f"`{command}`" for command in _member_surface_commands(title, surface)
    )
    headline = promise.splitlines()[0]
    return f"{_member_surface_kind(title)}: {commands} - {headline.lower().rstrip('.')}."


def _member_ec_dimension_item(title: str, cap_id: str) -> str:
    """The exact `EC Dimensions:` item a member declares, and must get back."""
    dimension = _member_ec_dimension(title)
    return f"{dimension}: `{_member_ec_runner(title)}` - {cap_id} {dimension} gate."


#: The second item of each list field on the one member that declares two.
#:
#: Kept in its own document (`MULTI_ITEM_SECTION_README`) rather than added to
#: every member: the EC dimensions a capability declares become gates in the
#: report, so widening them everywhere would move the claim arithmetic the
#: per-class counts are pinned to. Each value differs from that member's first
#: item in the sub-field the truncation would hide -- kind, dimension, and path.
MULTI_ITEM_TITLE = "Kubernetes-Native Deployment"
MULTI_ITEM_SURFACE_ITEM = (
    "CLI: `lumen deploy --dry-run` - render the manifests without applying them."
)
MULTI_ITEM_EC_DIMENSION_ITEM = (
    "security: `lumen deploy --verify-signatures` - "
    "kubernetes-native-deployment security gate."
)
MULTI_ITEM_GATE_INVENTORY_ITEM = "tech-design/kubernetes-native-deployment-rollout.md"


_TITLE_BY_ID = {
    member[1]: member[0]
    for member in _CORE_MEMBERS + _NON_CORE_MEMBERS + (_CONFLICT_MEMBER,)
}


def _member_type_for_id(cap_id: str) -> str:
    """The `Type:` a section for `cap_id` renders with.

    Documents built by rewriting a rendered section have to address the type by
    the same rule that wrote it; hardcoding `Service` here is what made those
    rewrites silently no-op once the fixture stopped declaring one type
    everywhere.
    """
    return _member_type(_TITLE_BY_ID.get(cap_id, ""))


#: Which per-member field maps are pairwise distinct across the six document
#: members, and which cannot be.
#:
#: The distinction is not a matter of care taken. `MEMBER_EC_DIMENSION` draws
#: from `EcDimensionKind`, a closed four-value enum, and no assignment of four
#: values to six members is injective -- so the strongest available statement
#: about that map is that it *covers* its enum, not that it separates its
#: members. The other four maps draw from vocabularies large enough to be
#: pairwise distinct, and are.
#:
#: Stated as a guard rather than left to prose because the prose got it wrong:
#: the durable evidence for this case claimed values "pairwise distinct per
#: capability, down to the surface kind and the EC dimension kind", which was
#: true of the surface kind and impossible of the dimension kind. A `> 1` guard
#: cannot tell those two situations apart, so it accepted the map and the label
#: describing it went unchallenged for eight rounds.
_PAIRWISE_DISTINCT_MEMBER_MAPS = (
    ("MEMBER_TYPE", MEMBER_TYPE),
    ("MEMBER_REQUIRED_VERIFICATION", MEMBER_REQUIRED_VERIFICATION),
    ("MEMBER_EC_RUNNER", MEMBER_EC_RUNNER),
    ("MEMBER_SURFACE_KIND", MEMBER_SURFACE_KIND),
)
_ENUM_COVERING_MEMBER_MAPS = (("MEMBER_EC_DIMENSION", MEMBER_EC_DIMENSION),)
for _map_name, _member_map in _PAIRWISE_DISTINCT_MEMBER_MAPS + _ENUM_COVERING_MEMBER_MAPS:
    assert {
        member[0] for member in _CORE_MEMBERS + _NON_CORE_MEMBERS
    } <= set(_member_map), f"{_map_name} must cover every document member"
for _map_name, _member_map in _PAIRWISE_DISTINCT_MEMBER_MAPS:
    _values = [_member_map[member[0]] for member in _CORE_MEMBERS + _NON_CORE_MEMBERS]
    assert len(set(_values)) == len(_values), (
        f"{_map_name} draws from a vocabulary wide enough to separate all six "
        f"document members and must do so; a repeat means a renderer that "
        f"answered for the wrong capability could still render this document. "
        f"Repeated: {sorted({v for v in _values if _values.count(v) > 1})}"
    )


def _section(
    member: tuple[Any, ...],
    feature_class: str | None,
    heading: str = "####",
    status: str = "verified",
    work_root_cells: dict[str, tuple[str, str, str, str, str]] | None = None,
    multi_item: bool = False,
    withheld: frozenset[str] = frozenset(),
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
        work_root_cells=work_root_cells,
        multi_item=multi_item,
        withheld=withheld,
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
_CONFLICT_TYPE = _member_type_for_id(CONFLICT_ID)
_CONFLICT_MARKER = f"ID: {CONFLICT_ID}\nType: {_CONFLICT_TYPE}\nFeature Class: core"
ROOT_FIELD_CONFLICT_DOCUMENT = _document(
    _CORE_MEMBERS + (_CONFLICT_MEMBER,),
    _NON_CORE_MEMBERS,
    core_class="core",
)
assert ROOT_FIELD_CONFLICT_DOCUMENT.count(_CONFLICT_MARKER) == 1, (
    "the conflict rewrite must find exactly one declared-core section for "
    f"{CONFLICT_ID}"
)
ROOT_FIELD_CONFLICT_DOCUMENT = ROOT_FIELD_CONFLICT_DOCUMENT.replace(
    _CONFLICT_MARKER,
    f"ID: {CONFLICT_ID}\nType: {_CONFLICT_TYPE}\nFeature Class: non_core",
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
#:
#: Every cell is pairwise distinct across the three rows. Two of them used to
#: share `shipped` and two used to share `none`, which is the fixture-coincidence
#: shape: a per-row assertion on a column whose values collide is satisfied by a
#: renderer that mixed those two rows up.
_LEGACY_ROWS = (
    ("Search Core", "shipped", "none", "#1", "`lumen serve`"),
    ("Lexical Search", "partial rollout", "analyzer coverage", "#2", "`lumen index`"),
    ("Security Hardening", "partial", "audit coverage", "#3", "`lumen auth`"),
)
LEGACY_ROW_COUNT = len(_LEGACY_ROWS)
for _column, _label in ((1, "Current State"), (2, "Gaps"), (3, "Active WI"), (4, "Evidence")):
    assert len({row[_column] for row in _LEGACY_ROWS}) == LEGACY_ROW_COUNT, (
        f"legacy rows must differ in every column; {_label} collides, and a "
        f"colliding column cannot tell a per-row renderer from a mixed-up one"
    )
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
    # (`claim_count` and `verified_count` are both 0 in this shape -- legacy
    # rows carry no work roots and no gate has run -- so the two pair-sums below
    # are `0 == 0` and neither can catch the mutation. They are kept because a
    # projection that attributed the rows but *invented* claims or verifications
    # for them would break them, and dropped from the credit this leg claims.)
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
    assert_legacy_index_rows_read_unverified(migrated)

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


def _legacy_row_id(title: str) -> str:
    return title.strip().lower().replace(" ", "-")


def _expected_legacy_section_body(
    title: str, state: str, gaps: str, wi: str, evidence: str
) -> str:
    """`render_legacy_capability_section`'s whole literal, for one row.

    Restated as one block rather than as a handful of substrings because the
    renderer *is* one `format!` literal (`capability.rs:9209-9223`): every field
    in it that the row does not supply -- `Status`, `Required Verification`, and
    the four readiness cells of the work-root row -- is a constant the product
    chose, and a per-substring assertion on the row-derived cells leaves all of
    them free. That is not a cosmetic gap: `Required Verification: smoke` and
    `planned | planned | smoke` are what mark a freshly migrated legacy row as
    unverified, and rewriting them to `implemented | verified | conformance`
    makes `aw capability migrate` mint a green readiness claim for a contract no
    one has written yet.
    """
    cap_id = _legacy_row_id(title)
    feature_class = "core" if cap_id in LEGACY_CORE_ROW_IDS else "non_core"
    return (
        f"\nID: {cap_id}\n"
        f"Root WI: {wi}\n"
        f"Status: candidate\n"
        f"Feature Class: {feature_class}\n"
        f"Required Verification: smoke\n"
        f"Promise:\n{state}\n"
        f"Gate Inventory:\n- {evidence}\n"
        f"\n"
        f"| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |\n"
        f"|---|---|---:|---|---|---|---|\n"
        f"| {gaps} | epic | {wi} | planned | planned | smoke | {evidence} |\n"
    )


def assert_migrated_legacy_sections_carry_their_row_content(
    migrated: str, *, tracker_state: dict[str, str]
) -> None:
    """Each legacy row's own cells land in the section it becomes, and nothing else does.

    `render_legacy_capability_section` (`capability.rs:9209-9223`) is one format
    literal that reads five cells off the row. Only the evidence cell was bound;
    the row's `Current State`, which becomes the section's `Promise`, and its
    `Gaps`, which becomes the name of the single work root, could each be
    replaced by a constant with nothing noticing -- so migration could turn three
    distinct legacy capabilities into three sections describing the same thing.

    Asserted per row against that row's own cells, which is only discriminating
    because `_LEGACY_ROWS` is pairwise distinct in every column; a colliding
    column would let a renderer that swapped two rows pass.

    Asserted as equality against the *whole* rendered block rather than as three
    substrings, for the reason recorded on `_expected_legacy_section_body`: the
    fields the row does not supply are the readiness constants, and those are
    precisely the ones a migration must not be free to invent. Equality also pins
    the field *order* and the absence of any field or row the renderer does not
    emit -- a `Type:` line spliced into every migrated legacy section is a class
    the author never declared, and a second work-root row appended after the
    emitted one is a claim with a gate the author never wrote (`report` accepts
    it: `claim_count` goes 1 -> 2 with no blocker). A prefix comparison sees
    neither, which is why the earlier `startswith` form here was wrong and is
    recorded as such rather than quietly replaced.

    Compared byte for byte, separator included. `_capability_section_body` cuts
    at the newline that begins the next heading, so a section the renderer
    followed with `\\n\\n` gives back a body ending in exactly one `\\n` -- which
    is what the literal ends with, and the comparison is therefore direct.

    An earlier revision compared after `rstrip("\\n")` on both sides, on the
    ground that the count of blank lines before the next heading is section
    *separation* rather than this capability's content and was pinned by the
    assertions that read the document frame and the section order. That ground
    was false: collapsing the literal's trailing `\\n\\n` to `\\n` -- which runs
    the last work-root row up against the following heading, so the table and the
    heading are one block -- passed on both entry points with the frame and every
    ordering assertion green. The separator is content, and nothing else here
    reads it.

    The document's *last* section has no following heading to be cut at, so its
    body runs to the end of the file and keeps the `\\n` the cut consumes for
    every other section. A revision that named that section and compared it after
    `rstrip("\\n")` on both sides was still an exemption: appending one more
    newline to the end of the whole document (`render_capability_registry`) is
    exactly the byte that section's body ends with, and it passed. The last
    section is compared against the same block plus that one newline instead, so
    the document's own ending is bound rather than skipped.

    `tracker_state` is the WI each section is expected to render, which differs
    by entry point: format migration erases document-stored tracker state before
    rendering, README relocation preserves it. The expectation is passed in
    rather than assumed, so this leg can bind both callers without asserting that
    either answer is the right one -- that disagreement is filed separately.
    """
    last_title = _rendered_capability_titles(migrated)[-1]
    for title, state, gaps, _wi, evidence in _LEGACY_ROWS:
        expected = _expected_legacy_section_body(
            title, state, gaps, tracker_state[title], evidence
        )
        body = _capability_section_body(migrated, title)
        if title == last_title:
            expected += "\n"
        assert body == expected, (
            f"migrated legacy section {title!r} is not the block its row "
            f"renders; expected:\n{expected!r}\ngot:\n{body!r}"
        )


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


#: The five trailing cells `render_capability_index`'s legacy branch writes for
#: every migrated row (`capability.rs:8941-8948`).
#:
#: None of them comes off the row: they are the constants the product chose to
#: mark a just-migrated legacy capability as *not yet verified*. Nothing read
#: them back, so the whole readiness half of the migrated index could be flipped
#: green -- `implemented | verified | conformance | ready` -- and `aw capability
#: migrate` would ship a table asserting production readiness for capabilities
#: whose contract has not been written, with this fixture's gate still passing.
LEGACY_INDEX_ROW_TAIL = (
    "planned",
    "planned",
    "smoke",
    "not_ready",
    "migrated from legacy table; confirm promise",
)


def assert_legacy_index_rows_read_unverified(migrated: str) -> None:
    """Every migrated legacy index row still reads as unverified and unready."""
    rows = _index_rows_parsed(migrated)
    assert len(rows) == LEGACY_ROW_COUNT, rows
    columns = ("Impl", "Verification", "Maturity", "Production", "Notes")
    for row in rows:
        actual = tuple(row[2:7])
        assert actual == LEGACY_INDEX_ROW_TAIL, (
            f"migrated legacy index row {row[0]!r} must read as unverified; "
            f"expected {dict(zip(columns, LEGACY_INDEX_ROW_TAIL))}, got "
            f"{dict(zip(columns, actual))}"
        )


#: The Capability Index header row and separator, exactly as the renderer writes
#: them (`capability.rs:8936-8940`).
#:
#: Pinned as literal text because this header is not decoration: it is the key
#: `parse_capability_index_summaries` reads the columns back by
#: (`find_table_column`, `capability.rs:9351-9356`). Every assertion drawn from
#: `_index_rows_parsed` locates its cells *positionally*, so renaming a column
#: changes nothing any of them can see -- while the document the rename produces
#: is one the product's own parser can no longer key, and the next `aw capability
#: migrate` over it silently replaces the orphaned column with `-`.
INDEX_HEADER_ROW = (
    "| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |"
)
INDEX_SEPARATOR_ROW = "|---|---:|---|---|---|---|---|"


def assert_index_header_is_the_declared_contract(migrated: str) -> None:
    """The rendered index carries the header its own parser keys on."""
    expected = f"### Capability Index\n\n{INDEX_HEADER_ROW}\n{INDEX_SEPARATOR_ROW}\n"
    assert expected in migrated, (
        f"the rendered Capability Index must carry the declared header and "
        f"separator; expected:\n{expected!r}\ndocument was:\n{migrated}"
    )


def _index_rows_parsed(migrated: str) -> list[list[str]]:
    """Every Capability Index row as its list of cells, in document order.

    Bounded to the index table itself: rows are taken from `### Capability
    Index` up to the next `###` heading, which is the first feature root.

    Splits on unescaped `|` only, and unescapes `\\|` back to `|`. That is not a
    convenience: one fixture promise contains a pipe and reaches this table
    through the `Notes` fallback, so a renderer that skipped
    `markdown_cell`'s escaping would produce a row with one column too many.
    Reading it with a naive `split("|")` would silently absorb that extra column
    and the corrupted table would still parse.
    """
    # Every caller of this helper reads a *rendered* document, and every one of
    # them addresses its cells by position -- so the header the columns are named
    # by is invisible to all of them. Pinned here rather than at each call site
    # so a new leg that parses rows cannot forget it.
    assert_index_header_is_the_declared_contract(migrated)
    start = migrated.index("### Capability Index")
    rest = migrated[start + len("### Capability Index") :]
    end = rest.find("\n### ")
    table = rest if end == -1 else rest[:end]
    rows = []
    for raw in table.splitlines():
        line = raw.strip()
        if not line.startswith("|") or set(line) <= set("|-: "):
            continue
        cells = _split_escaped_row(line)
        if cells and cells[0] == "Capability":
            continue
        rows.append(cells)
    return rows


def _split_escaped_row(line: str) -> list[str]:
    """One Markdown table row's cells, honouring `\\|` as literal content."""
    body = line.strip()
    if body.startswith("|"):
        body = body[1:]
    if body.endswith("|") and not body.endswith("\\|"):
        body = body[:-1]
    cells: list[str] = []
    current: list[str] = []
    index = 0
    while index < len(body):
        char = body[index]
        if char == "\\" and index + 1 < len(body) and body[index + 1] == "|":
            current.append("|")
            index += 2
            continue
        if char == "|":
            cells.append("".join(current).strip())
            current = []
            index += 1
            continue
        current.append(char)
        index += 1
    cells.append("".join(current).strip())
    return cells


def _index_titles(migrated: str) -> list[str]:
    """The first cell of every Capability Index row, in document order."""
    return [row[0] for row in _index_rows_parsed(migrated)]


def _section_titles(migrated: str) -> list[str]:
    return [
        raw.strip()[len("#### ") :].strip()
        for raw in migrated.splitlines()
        if raw.strip().startswith("#### ")
    ]


def assert_migration_index_and_sections_agree_on_order(migrated: str) -> None:
    """The migrated index and the migrated sections are in the same order.

    Named for what it asserts. This was `assert_migration_reaches_a_fixed_point`
    for twenty-nine rounds, and it does not run migration twice: order agreement
    is a *precondition* for convergence, not convergence. The rule the old name
    claimed is bound by
    `assert_migration_is_idempotent_at_its_fixed_point` instead.

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


#: The sentence the migrate tick emits once there is nothing left to migrate,
#: as a format string over the document it examined. Asserted whole and with
#: the path substituted, so a constant reassurance that names no document --
#: which is what an unconditional early return would emit -- cannot pass.
MIGRATION_FIXED_POINT_STDOUT = "{cap_path} already uses canonical Markdown capability format"

#: The exact ordered `(status, changed, kind)` triples migration is expected to
#: emit before it converges, one entry per arrival path. `kind` is pinned
#: because the two migrations are separate phases in a required order -- a
#: contract sitting in the README is relocated first and reformatted second --
#: and because a single tick performing both would report one triple, not two.
MIGRATION_TICKS_FROM_README = (
    ("migrated", True, "location_migration_required"),
    ("migrated", True, "format_migration_required"),
)
#: The same for a contract already resident in CAPABILITIES.md: no relocation
#: is due, so the format phase is the whole of the work.
MIGRATION_TICKS_FROM_CAPABILITIES = (("migrated", True, "format_migration_required"),)

#: How many ticks the driver runs past convergence. Two, not one, so that
#: "unchanged" is asserted to *stay* unchanged: a tick that alternates between
#: rewriting and not rewriting reports a clean no-op every other run.
MIGRATION_TICKS_PAST_CONVERGENCE = 2


def assert_migration_is_idempotent_at_its_fixed_point(
    ticks: list[tuple[dict, str]],
    expected_migrating_ticks: tuple[tuple[str, bool, str], ...],
    *,
    subject: str,
) -> None:
    """Migration converges, and the tick that reports no change made none.

    `ticks` is `(envelope, document text)` per `aw capability migrate` run, in
    order, driven until the envelope stops reporting a change plus
    `MIGRATION_TICKS_PAST_CONVERGENCE` further runs.

    Convergence is the property that makes migration safe to re-run, and it was
    bound by nothing: every leg here migrates exactly once and reads the result.
    A tick that rewrote the document on every invocation -- or one that reported
    `unchanged` while still writing -- passed all of them, and would keep
    passing while `aw capability migrate` never terminated for an adopter
    driving it to completion.

    Four separable things are asserted, because the interesting failures are not
    all the same failure:

    * The migrating prefix is the exact ordered triple list, so a run that
      skipped a phase, ran them in the other order, or performed both in one
      tick is caught. `changed` and `status` are pinned alongside `kind`
      because `kind` names the *check that ran*, not its outcome: the product
      reports `format_migration_required` on the tick that migrates and on the
      tick that finds nothing to do alike. That is asserted below rather than
      left as a comment, so a later revision cannot quietly start reading
      `kind` as the discriminator and believe it has bound something.
    * Every tick after the prefix reports `unchanged` / `changed is False` and
      emits the fixed-point sentence naming its own document. These are one
      observation, not three: the product derives `changed` from whether its
      own stdout starts with `"migrated "`, and `status` from `changed`. They
      are asserted together anyway, because the derivation is a product detail
      the contract does not promise and a future revision computing `changed`
      from the filesystem would leave the envelope fields unpinned.
    * The document is byte-identical from the last migrating tick onward. This
      is the only half that can catch a migration which rewrites the file while
      reporting a no-op -- precisely because `changed` is a re-read of stdout
      rather than an observation of the document, a tick that returned the
      fixed-point sentence and wrote anyway would report `unchanged` and pass
      every envelope assertion above it.
    * The loop stopped after exactly the expected phases plus the runs past
      convergence. The driver ticks until it has seen
      `MIGRATION_TICKS_PAST_CONVERGENCE` consecutive no-ops, under a bound, so
      the tick count is an observation rather than a constant: a migration
      needing a third phase, or one never converging and stopping at the bound,
      lands on a different length.

    What is deliberately *not* asserted is the content of the converged
    document. Two defects reachable from these inputs -- #3264, which drops
    every `Root WI` during the format phase, and #3265, which truncates a
    promise at `markdown_cell`'s own `\\|` escape when the index table is read
    back -- are preserved at the fixed point. Pinning the converged bytes would
    hold both losses in place. Idempotence is orthogonal to them: fixing either
    changes what the fixed point contains, not that there is one.
    """
    expected_length = len(expected_migrating_ticks) + MIGRATION_TICKS_PAST_CONVERGENCE
    assert len(ticks) == expected_length, (
        f"{subject}: the tick loop ran {len(ticks)} times where {expected_length} "
        f"was expected -- {len(expected_migrating_ticks)} migrating phase(s) "
        f"followed by {MIGRATION_TICKS_PAST_CONVERGENCE} consecutive no-ops; a "
        f"longer run means migration needed a phase it is not supposed to need, "
        f"or never converged and stopped at the driver's bound"
    )

    observed = [
        (envelope.get("status"), envelope.get("changed"), envelope.get("result", {}).get("kind"))
        for envelope, _document in ticks
    ]
    migrating = observed[: len(expected_migrating_ticks)]
    assert migrating == list(expected_migrating_ticks), (
        f"{subject}: migration did not run the phases it is required to run, in "
        f"order; expected {list(expected_migrating_ticks)} got {migrating}"
    )

    settled = ticks[len(expected_migrating_ticks) :]
    assert len(settled) >= MIGRATION_TICKS_PAST_CONVERGENCE, (
        f"{subject}: {len(settled)} tick(s) observed past the migrating phases, "
        f"fewer than the {MIGRATION_TICKS_PAST_CONVERGENCE} an alternating "
        f"rewrite would survive"
    )
    for offset, (envelope, _document) in enumerate(settled):
        tick = len(expected_migrating_ticks) + offset + 1
        assert envelope.get("status") == "unchanged", (
            f"{subject}: tick {tick} reports status={envelope.get('status')!r}, so "
            f"migration has not converged"
        )
        assert envelope.get("changed") is False, (
            f"{subject}: tick {tick} reports changed={envelope.get('changed')!r}, so "
            f"migration has not converged"
        )
        stdout = (envelope.get("result", {}).get("stdout") or "").strip()
        expected_stdout = MIGRATION_FIXED_POINT_STDOUT.format(
            cap_path=envelope.get("cap_path")
        )
        assert stdout == expected_stdout, (
            f"{subject}: tick {tick} does not report its own document as already "
            f"canonical; expected {expected_stdout!r} got {stdout!r}"
        )

    settled_kind = settled[0][0].get("result", {}).get("kind")
    last_migrating_kind = expected_migrating_ticks[-1][2]
    assert settled_kind == last_migrating_kind, (
        f"{subject}: the converged tick reports kind={settled_kind!r} where the "
        f"last migrating tick reported {last_migrating_kind!r}; `kind` names the "
        f"check that ran and cannot be read as whether it changed anything, and "
        f"an assertion that read it that way would bind nothing"
    )

    converged_document = ticks[len(expected_migrating_ticks) - 1][1]
    for offset, (_envelope, document) in enumerate(settled):
        tick = len(expected_migrating_ticks) + offset + 1
        assert document == converged_document, (
            f"{subject}: tick {tick} reported no change but rewrote the document; "
            f"{len(converged_document)} bytes before, {len(document)} after"
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


#: The title a relocated document and its residue pointer must carry. Relocation
#: titles the document after the *project*, not after the README it moved the
#: contract out of -- every fixture README here is headed `# Lumen` while the
#: fixture project is `demo`, so the two are deliberately different strings and
#: an implementation that echoed the README H1 would be visible.
RELOCATION_PROJECT_TITLE = "Demo"
assert RELOCATION_PROJECT_TITLE not in LEGACY_TABLE_DOCUMENT, (
    "the relocation title must not already appear in the input document, or "
    "'the product titled it correctly' is satisfied by copying the input"
)


#: The exact frame `render_relocated_capability_document` (`capability.rs:13127-13137`)
#: wraps a relocated contract in: the title, a `## Brief` naming the project, and
#: the `## Capabilities` preamble that tells a reader which of the three input
#: dialects the document below is.
RELOCATED_DOCUMENT_FRAME = (
    "# {title}\n"
    "\n"
    "## Brief\n"
    "\n"
    "Machine-readable capability contract for {title}.\n"
    "\n"
    "## Capabilities\n"
    "\n"
    "Canonical field-style capability contracts below are machine-readable input "
    "for `aw capability`; YAML and legacy tables are migration input only.\n"
    "\n"
)


def assert_relocated_document_is_the_declared_frame(migrated: str) -> None:
    """A relocated contract opens with the frame the product promises, byte for byte.

    Everything else in this fixture reads the relocated document by *searching*
    it -- for a section, a row, an id -- so the frame the product writes around
    that content was never read at all. Both prose blocks were free: the `##
    Brief` body could be rewritten to `TODO.` and the `## Capabilities` preamble,
    which is the only thing telling a reader that the contract below is canonical
    field style rather than migration input, could be dropped entirely. Neither
    is decoration: relocation is the step that turns a human README into the
    machine-readable document `aw capability` parses, and the frame is what says
    so.

    Asserted as a prefix rather than by containment so the heading *order* is
    pinned too -- a document whose `## Capabilities` preceded its `## Brief`
    would satisfy three containment checks and still be the wrong document.
    """
    expected = RELOCATED_DOCUMENT_FRAME.format(title=RELOCATION_PROJECT_TITLE)
    assert migrated.startswith(expected), (
        f"relocated document did not open with the declared frame; expected it "
        f"to start with:\n{expected!r}\ngot:\n{migrated[: len(expected) + 120]!r}"
    )


#: What the residue pointer must forward *to*. Held as its own constant because
#: the href is the whole point of the pointer: a heading that reads
#: "Capability Contract" while linking somewhere else is worse than no pointer.
README_CONTRACT_HREF = "[CAPABILITIES.md](CAPABILITIES.md)"


def assert_readme_residue_forwards_to_the_contract(readme: str) -> None:
    """The residue's pointer names the file the contract actually moved to.

    `render_readme_capability_migration_residue` (`capability.rs:13142-13170`)
    appends a heading, a sentence, and a link. Only the heading text and the bare
    string `CAPABILITIES.md` were bound, and both survive an implementation that
    retargets the href -- `[CAPABILITIES.md](docs/nowhere.md)` still contains
    `CAPABILITIES.md` and still carries the heading. The residue is the *only*
    thing left in the README after relocation takes the table away, so a broken
    href is the difference between a reader finding the contract and losing it.
    """
    expected = (
        f"{README_CONTRACT_POINTER}\n\nMachine-readable capability contract for "
        f"{RELOCATION_PROJECT_TITLE}. Full contract:\n{README_CONTRACT_HREF}.\n"
    )
    assert expected in readme, (
        f"residue did not carry the pointer block; expected:\n{expected!r}\n"
        f"README was:\n{readme}"
    )


#: Per-capability tracker state for the section-shaped relocation fixtures. The
#: values are distinct so a renderer that emitted one capability's `Root WI` for
#: every row could not satisfy the assertions drawn from them.
_ALL_MEMBERS = _CORE_MEMBERS + _NON_CORE_MEMBERS
SECTION_RELOCATION_WI = {
    member[0]: f"#{10 + index}" for index, member in enumerate(_ALL_MEMBERS)
}
assert len(set(SECTION_RELOCATION_WI.values())) == len(_ALL_MEMBERS)


#: Each fixture member's own promise text, keyed by title. Pairwise distinct, so
#: "the section carries its own promise" cannot be satisfied by a renderer that
#: copied one capability's promise into all of them.
MEMBER_PROMISE = {member[0]: member[2] for member in _ALL_MEMBERS}
assert len(set(MEMBER_PROMISE.values())) == len(_ALL_MEMBERS), (
    "fixture members must carry pairwise-distinct promises"
)


def member_promise_note_cell(title: str) -> str:
    """That member's promise as it must read back out of a parsed index row.

    `markdown_cell` (`capability.rs:9244-9250`) performs two substitutions and
    this is what survives each of them through `_index_rows_parsed`: the pipe
    escape is undone by the row reader, so the pipe compares as authored, while
    the newline fold is *not* undone -- `<br>` is the cell's real content, and
    the only correct expectation for a promise the author wrote across two
    lines.

    Both substitutions are nonetheless bound by this one comparison, because
    each fails the reader in its own way rather than merely differing: an
    unescaped pipe splits the row into an extra column, an unfolded newline ends
    the row before its last cell. Neither can be absorbed silently.
    """
    return MEMBER_PROMISE[title].replace("\n", "<br>")


assert any("\n" in promise for promise in MEMBER_PROMISE.values()), (
    "one member's promise must span two lines, or `markdown_cell`'s newline "
    "fold never runs and deleting it renders the identical document"
)
assert any("|" in promise for promise in MEMBER_PROMISE.values()), (
    "one member's promise must contain a pipe, or the escape beside the fold "
    "is free in the same way"
)
assert any(
    "\n" in promise and "|" in promise for promise in MEMBER_PROMISE.values()
), (
    "both must reach the *same* cell. Split across two promises, each "
    "substitution is asserted on a cell where the other one does nothing, and "
    "an implementation applying them in the wrong order -- folding a newline "
    "into `<br>` and then escaping nothing, or escaping a pipe the fold later "
    "re-splits -- is never a different document"
)


#: The rest of each member's rendered contract, keyed by title: the exact
#: `Surfaces:`, `EC Dimensions:` and `Gate Inventory:` list items the renderers
#: must produce for it.
#:
#: These are the *input* items round-tripped, which is the contract:
#: `render_surface_field_items` (`capability.rs:9154-9175`) and
#: `render_ec_dimension_field_items` (`capability.rs:9177-9198`) re-emit
#: `kind: commands - summary` and `dimension: \`runner\` - summary`, and
#: `capability_raw_gate_inventory` returns the declared inventory verbatim.
MEMBER_SURFACE_ITEM = {
    member[0]: _member_surface_item(member[0], member[3], member[2])
    for member in _ALL_MEMBERS
}
MEMBER_EC_DIMENSION_ITEM = {
    member[0]: _member_ec_dimension_item(member[0], member[1])
    for member in _ALL_MEMBERS
}
MEMBER_GATE_INVENTORY_ITEM = {
    member[0]: f"tech-design/{member[1]}.md" for member in _ALL_MEMBERS
}
for _field_name, _field_map in (
    ("MEMBER_SURFACE_ITEM", MEMBER_SURFACE_ITEM),
    ("MEMBER_EC_DIMENSION_ITEM", MEMBER_EC_DIMENSION_ITEM),
    ("MEMBER_GATE_INVENTORY_ITEM", MEMBER_GATE_INVENTORY_ITEM),
):
    assert len(set(_field_map.values())) == len(_ALL_MEMBERS), (
        f"{_field_name} must be pairwise distinct across members, or a renderer "
        f"that emitted one capability's value for all of them still passes"
    )
#: The three list-shaped fields, in the order
#: `render_markdown_capability_section_at_level` emits them, and the
#: `item_overrides` key each is addressed by.
LIST_FIELDS = ("Gate Inventory", "Surfaces", "EC Dimensions")
_OVERRIDE_KEYS = {
    "Gate Inventory": "gate_inventory",
    "Surfaces": "surfaces",
    "EC Dimensions": "ec_dimensions",
}
_DECLARED_LIST_ITEMS = {
    "Gate Inventory": MEMBER_GATE_INVENTORY_ITEM,
    "Surfaces": MEMBER_SURFACE_ITEM,
    "EC Dimensions": MEMBER_EC_DIMENSION_ITEM,
}

#: Distinctness of the assembled item is not enough: `kind`, `commands` and
#: `summary` are three separate reads, and varying only the command text leaves
#: the kind read satisfiable by a literal. Asserted on the sub-field itself.
assert len({MEMBER_SURFACE_KIND[member[0]] for member in _ALL_MEMBERS}) == len(
    _ALL_MEMBERS
), "MEMBER_SURFACE_KIND must be pairwise distinct across members"

#: And the command half is a list of its own, one level *inside* the item. The
#: multi-item document raised the item list to two and left this one at one,
#: which is the same arity blind spot one nesting level down.
_MULTI_COMMAND_SURFACES = tuple(
    member[0]
    for member in _ALL_MEMBERS
    if len(_member_surface_commands(member[0], member[3])) > 1
)
assert _MULTI_COMMAND_SURFACES, (
    "some member has to declare a surface with more than one command, or "
    "`.take(1)` on the command vector renders the identical document"
)
for _title in _MULTI_COMMAND_SURFACES:
    _commands = _member_surface_commands(_title, "")
    assert len(set(_commands)) == len(_commands), (
        f"{_title!r} must declare distinct commands, or keeping the first and "
        f"keeping the last render the same item"
    )
    assert not any(" - " in command for command in _commands), (
        f"{_title!r} must not write the summary separator inside a command, or "
        f"the parse cuts the item somewhere other than where it renders it"
    )


def assert_sections_carry_their_own_contract(
    migrated: str,
    titles: tuple[str, ...],
    *,
    item_overrides: dict[str, dict[str, str]] | None = None,
) -> None:
    """Every rendered capability section carries its *own* contract fields.

    `render_markdown_capability_section_at_level` (`capability.rs:9007-9061`)
    re-emits a capability's whole field block, and the product's own doc comment
    says everything constituting the promise is carried through untouched. Of
    that list only `ID:` was actually bound; six further fields were each
    replaceable by a single literal without failing anything:

    - `Promise:` (`capability.rs:9037-9039`)
    - `Type:` (`capability.rs:9026-9028`), because every member declared
      `Service`
    - `Required Verification:` (`capability.rs:9032-9035`), because every member
      declared `smoke`
    - `Surfaces:` (`capability.rs:9043-9048`) and `EC Dimensions:`
      (`capability.rs:9049-9054`), which differed per member but were never read
      back
    - `Gate Inventory:` (`capability.rs:9039-9042`), which was read back for one
      capability only, so "every capability claims `search-core`'s inventory"
      passed
    - `Dependencies:` (`capability.rs:9055-9060`), which no capability declared
      at all, so the whole block was deletable

    Asserted per capability against pairwise-distinct values, so a renderer that
    copies one capability's field into all of them is a different document. The
    surface and EC dimension items are asserted whole rather than by their
    varying half, because each is assembled from separate reads of its kind, its
    command, and its summary, and a guard on the assembled item alone cannot see
    a constant sub-field.

    `Dependencies:` is asserted in both directions -- present with its own value
    for the two members that declare one, absent for the four that do not --
    because a renderer that emitted the block unconditionally would attribute a
    dependency edge to capabilities that never claimed one.

    One of those two declares more than one, out of sorted order and with a
    repeat, and is asserted as the *exact* item list its field renders. Both
    declaring members previously carried exactly one dependency, which left
    `.take(1)` on the render loop rendering the identical document -- the same
    arity gap the multi-item document closes for `Surfaces`, `EC Dimensions` and
    `Gate Inventory`, which this field was left out of. The expectation is
    `_member_rendered_dependencies`, not the declaration: the parse canonicalises
    through a `BTreeSet`, and asserting the sorted, deduplicated form is what
    makes those two steps observable instead of coincidental.

    Read as an item list rather than asserted as a containing block for the
    reason on `_field_list_items`: the deduplication renders one item *fewer*
    than the input declares, so a parse that stopped deduplicating appends the
    repeat after the two items the expectation names -- and the expectation is
    still a substring of what it renders. That mutation survived the containing
    form and fails this one.

    The three list-shaped fields are read the same way, for the same reason, and
    that was a later correction. They were asserted as containing blocks --
    `Surfaces:\\n- <item>\\n` -- which pins the first item and nothing after it.
    Appending a duplicate of the first item after either render loop
    (`capability.rs:9043-9054`) rendered a document every assertion here still
    accepted, on the multi-item document too, because its expectation named the
    two declared items and the third followed them. `Gate Inventory:` escaped
    only because one leg of the derived-inventory document already read it as a
    list. All three are now read as item lists and compared for equality, so a
    field is what it renders rather than what it starts with.

    `item_overrides` lets one input declare differently *shaped* items -- a
    command with no summary -- or a different *number* of them, without exempting
    that capability from the rest of the block. The alternative, skipping the
    capability, would trade the arm this binds for the six fields it stops
    binding on that document.
    """
    for title in titles:
        body = _capability_section_body(migrated, title)
        overrides = (item_overrides or {}).get(title, {})
        dependencies = _member_rendered_dependencies(title)
        if dependencies:
            rendered = _field_list_items(body, title, "Dependencies")
            assert rendered == list(dependencies), (
                f"section {title!r} must render the dependencies "
                f"{list(dependencies)!r}, in that order and nothing else; got "
                f"{rendered!r}. Section was:\n{body}"
            )
        else:
            assert "Dependencies:" not in body, (
                f"section {title!r} declared no dependencies, so rendering the "
                f"field at all invents an edge; section was:\n{body}"
            )
        # The scalar fields. Two of the three are optional at the input and are
        # therefore addressable by an override, in both directions: `None` says
        # the capability declared nothing and the renderer must emit nothing,
        # and a string says the capability declared nothing and the renderer
        # must emit *that* -- which is a different rule and a different defect
        # when it breaks. `Type:` takes the first form and
        # `Required Verification:` the second, because the renderer substitutes
        # `capability_maturity_summary`'s fallback for the latter and simply
        # omits the former.
        for label, key, field, declared in (
            ("promise", None, "Promise", f"\n{MEMBER_PROMISE[title]}"),
            ("type", "type", "Type", f" {_member_type(title)}"),
            (
                "required verification",
                "required_verification",
                "Required Verification",
                f" {_member_required_verification(title)}",
            ),
        ):
            if key is not None and key in overrides:
                substituted = overrides[key]
                if substituted is None:
                    assert f"{field}:" not in body, (
                        f"section {title!r} declared no {field}, so rendering "
                        f"the field states a contract the author did not "
                        f"write; section was:\n{body}"
                    )
                    continue
                expected = f"{field}: {substituted}\n"
                assert expected in body, (
                    f"section {title!r} declared no {field} and must render the "
                    f"substituted {expected!r}; section was:\n{body}"
                )
                continue
            expected = f"{field}:{declared}\n"
            assert expected in body, (
                f"section {title!r} did not carry its own {label} "
                f"{expected!r}; section was:\n{body}"
            )
        for field in LIST_FIELDS:
            expected_items = list(
                overrides.get(
                    _OVERRIDE_KEYS[field], (_DECLARED_LIST_ITEMS[field][title],)
                )
            )
            if not expected_items:
                # An empty override means the capability declared no such field,
                # and the renderer must emit none -- not an empty one. The
                # heading alone reads as a declared-but-empty contract, and each
                # of the three fields is guarded separately on its own vector
                # being non-empty; with every capability declaring one item of
                # each, forcing any of those guards true was unobservable.
                assert f"{field}:" not in body, (
                    f"section {title!r} declared no {field}, so rendering the "
                    f"field -- even empty -- states a contract the author did "
                    f"not write; section was:\n{body}"
                )
                continue
            rendered = _field_list_items(body, title, field)
            assert rendered == expected_items, (
                f"section {title!r} must render the {field} items "
                f"{expected_items!r}, in that order and nothing else; got "
                f"{rendered!r}. Section was:\n{body}"
            )


def _section_readme(
    members: tuple[tuple[Any, ...], ...],
    classes: tuple[str | None, ...],
    brief: str,
    statuses: tuple[str, ...] | None = None,
    preludes: tuple[str | None, ...] | None = None,
    work_root_cells: dict[str, tuple[str, str, str, str, str]] | None = None,
    multi_item_title: str | None = None,
    postludes: tuple[str | None, ...] | None = None,
    withheld_by_title: dict[str, frozenset[str]] | None = None,
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
    for index, (member, declared) in enumerate(zip(members, classes)):
        status = "verified" if statuses is None else statuses[index]
        section = _section(
            member,
            declared,
            "###",
            status=status,
            work_root_cells=work_root_cells,
            multi_item=member[0] == multi_item_title,
            withheld=(withheld_by_title or {}).get(member[0], frozenset()),
        ).replace(
            "Root WI: -", f"Root WI: {SECTION_RELOCATION_WI[member[0]]}"
        )
        prelude = None if preludes is None else preludes[index]
        if prelude is not None:
            heading = f"### {member[0]}\n\n"
            assert section.startswith(heading), section
            section = heading + prelude + "\n\n" + section[len(heading) :]
        postlude = None if postludes is None else postludes[index]
        if postlude is not None:
            # Below the work-root table, which is the last machine table in the
            # block: `markdown_capability_prose_around_machine_tables`
            # (`capability.rs:10966-10988`) reads the prelude from the lines
            # above the first one and the postlude from the lines below the last.
            assert section.endswith("|\n"), section
            section = section + "\n" + postlude + "\n"
        body += section
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


#: Per-work-root table cells for the one relocation shape that varies them.
#:
#: Every other document here writes `| <root> | change | - | implemented |
#: verified | smoke | \`true\` |` for all eight work roots, so five of the seven
#: cells the work-root renderer emits (`capability.rs:9064-9076`) were each
#: replaceable by a constant: only `Work Root` and `WI` differed between rows.
#:
#: `Kind` has only three legal non-empty tokens (`validate_work_root_kind`,
#: `capability.rs:11816-11824`) against eight rows, so it is asserted as "all
#: three appear" rather than pairwise-distinct; the other four columns are
#: pairwise distinct across every row.
#:
#: This variation is confined to its own document on purpose. `Impl` and
#: `Verification` feed `capability_gap_status_from_table`
#: (`capability.rs:11866-11885`), which feeds `capability_impl_summary`
#: (`capability.rs:9273-9295`), which is what the varied-status shape pins its
#: index columns to -- varying these globally would make that leg's expectation
#: a function of two inputs instead of the status alone.
VARIED_WORK_ROOT_CELLS = {
    "query-planner-boolean-eval": (
        "change",
        "implemented",
        "verified",
        "conformance",
        "`lumen-planner-suite`",
    ),
    "bm25-ranking": ("epic", "partial", "passing", "smoke", "`lumen-bm25-suite`"),
    "analyzer-pipeline": (
        "subepic",
        "planned",
        "planned",
        "corpus",
        "`lumen-analyzer-suite`",
    ),
    "operational-endpoint-set": (
        "change",
        "out_of_scope",
        "none",
        "none",
        "`lumen-probe-suite`",
    ),
    "manifest-packaging": (
        "epic",
        "blocked",
        "failing",
        "negative",
        "`lumen-manifest-suite`",
    ),
    "transport-and-identity-hardening": (
        "subepic",
        "implemented",
        "blocked",
        "dogfood",
        "`lumen-identity-suite`",
    ),
    "gate-configuration": (
        "change",
        "partial",
        "verified",
        "smoke",
        "`lumen-gate-config-suite`",
    ),
    "gate-inventory-sync": (
        "epic",
        "planned",
        "passing",
        "conformance",
        "`lumen-gate-sync-suite`",
    ),
}
assert set(VARIED_WORK_ROOT_CELLS) == {
    work_root for member in _ALL_MEMBERS for work_root in member[4]
}, sorted(VARIED_WORK_ROOT_CELLS)
#: Every legal non-empty token each enum-valued work-root column accepts,
#: restated from the validators (`capability.rs:11816-11864`). Four of the five
#: varied columns have fewer legal tokens than there are rows, so "pairwise
#: distinct" is unachievable for them; the stronger property available is that
#: each column exercises its whole vocabulary, which no single constant can.
_WORK_ROOT_COLUMN_VOCABULARY = {
    (0, "Kind"): {"epic", "subepic", "change"},
    (1, "Impl"): {"planned", "partial", "implemented", "blocked", "out_of_scope"},
    (2, "Verification"): {
        "none",
        "planned",
        "failing",
        "passing",
        "verified",
        "blocked",
    },
    (3, "Maturity"): {
        "none",
        "smoke",
        "conformance",
        "corpus",
        "negative",
        "dogfood",
    },
}
for (_cell_column, _cell_label), _vocabulary in _WORK_ROOT_COLUMN_VOCABULARY.items():
    assert {
        cells[_cell_column] for cells in VARIED_WORK_ROOT_CELLS.values()
    } == _vocabulary, (
        f"work-root column {_cell_label} must exercise every legal token, or the "
        f"tokens it omits are indistinguishable from a constant"
    )
assert len({cells[4] for cells in VARIED_WORK_ROOT_CELLS.values()}) == len(
    VARIED_WORK_ROOT_CELLS
), "the Gate / Evidence column has no closed vocabulary and must be pairwise distinct"
#: The rows *within* one capability must differ too. Cross-capability distinctness
#: alone would leave "every row of a capability gets that capability's first row"
#: passing on the four members that declare only one work root.
for _member in _ALL_MEMBERS:
    _member_rows = [VARIED_WORK_ROOT_CELLS[work_root] for work_root in _member[4]]
    assert len(set(_member_rows)) == len(_member_rows), (
        f"{_member[0]!r} must not repeat a work-root row shape"
    )

VARIED_WORK_ROOT_SECTION_README = _section_readme(
    _ALL_MEMBERS,
    (None,) * len(_ALL_MEMBERS),
    "Lumen README-resident capability contract, work-root rows all different.",
    work_root_cells=VARIED_WORK_ROOT_CELLS,
)


#: Relocation input where one capability declares *two* of every list field.
#:
#: Every other document here declares exactly one Surface, one EC Dimension and
#: one Gate Inventory entry per capability, which makes list arity unobservable:
#: an implementation rendering only the first element of each produces the
#: byte-identical document, while silently dropping contract content from any
#: real two-surface capability.
MULTI_ITEM_SECTION_README = _section_readme(
    _ALL_MEMBERS,
    (None,) * len(_ALL_MEMBERS),
    "Lumen README-resident capability contract, one capability declaring two of each.",
    multi_item_title=MULTI_ITEM_TITLE,
)
assert MULTI_ITEM_SECTION_README.count(MULTI_ITEM_SURFACE_ITEM) == 1, (
    "the multi-item document must actually declare the second surface, or its "
    "assertion pins nothing"
)

#: The exact item lists the multi-item capability must render, for the
#: whole-block assertion that runs over every capability of every document.
#: Without them that assertion would expect one item where this document
#: declares two, so the arity this document exists to bind would be asserted in
#: only one place instead of two.
MULTI_ITEM_OVERRIDES = {
    MULTI_ITEM_TITLE: {
        "surfaces": (MEMBER_SURFACE_ITEM[MULTI_ITEM_TITLE], MULTI_ITEM_SURFACE_ITEM),
        "ec_dimensions": (
            MEMBER_EC_DIMENSION_ITEM[MULTI_ITEM_TITLE],
            MULTI_ITEM_EC_DIMENSION_ITEM,
        ),
        "gate_inventory": (
            MEMBER_GATE_INVENTORY_ITEM[MULTI_ITEM_TITLE],
            MULTI_ITEM_GATE_INVENTORY_ITEM,
        ),
    }
}
# Surfaces and Gate Inventory render in declaration order; EC dimensions render
# in `CapabilityEcDimensionKind` order, because `dedupe_ec_dimensions` collapses
# them through a `BTreeMap`. The declaration in `_section` therefore prepends the
# second dimension and appends the other two, so this document distinguishes the
# sort from declaration order rather than agreeing with both.
assert (
    MULTI_ITEM_OVERRIDES[MULTI_ITEM_TITLE]["ec_dimensions"][0]
    != MULTI_ITEM_EC_DIMENSION_ITEM
), (
    "the multi-item member must declare its EC dimensions in an order the sort "
    "does not agree with, or the sort renders the identical document"
)
assert (
    MULTI_ITEM_OVERRIDES[MULTI_ITEM_TITLE]["surfaces"][-1] == MULTI_ITEM_SURFACE_ITEM
    and MULTI_ITEM_OVERRIDES[MULTI_ITEM_TITLE]["gate_inventory"][-1]
    == MULTI_ITEM_GATE_INVENTORY_ITEM
), (
    "the two list fields that do keep declaration order must still expect their "
    "second item last, or the two rules are not being told apart"
)


#: Relocation input carrying both *partial* item shapes: one capability declares
#: a Surface and an EC Dimension as a command with no summary, another declares
#: each as a summary with no command.
#:
#: `render_surface_field_items` (`capability.rs:9153-9172`) and
#: `render_ec_dimension_field_items` (`capability.rs:9176-9196`) are each a
#: four-arm match on whether the item has commands and whether it has a summary.
#: Every other document here declares both halves for every item, so only the
#: two-field arm was ever entered. The command-only arm -- the shape an author
#: writes when the command *is* the description -- could be replaced by
#: `String::new()`, which drops the surface out of the migrated contract
#: entirely while every other assertion here still passes.
#:
#: Adding the command-only shape did not reach the summary-only arm, and a
#: later round confirmed it: replacing `format!("{}: {}", kind, summary)` with
#: `summary.to_string()` in either renderer still rendered this whole fixture
#: byte for byte. That arm drops the `kind`/`dimension` prefix, and for the
#: dimension renderer the prefix is exactly what `parse_ec_dimension_kind`
#: (`capability.rs:11705-11715`) reads back -- an item losing it stops parsing as
#: a dimension at all on the next migration. Both partial shapes therefore live
#: on one document, on two different capabilities, so neither is asserted by a
#: leg that also has the other's answer in front of it.
NO_SUMMARY_TITLE = "Security Hardening"
_NO_SUMMARY_MEMBER = next(
    member for member in _ALL_MEMBERS if member[0] == NO_SUMMARY_TITLE
)
NO_SUMMARY_SURFACE_ITEM = "{}: {}".format(
    _member_surface_kind(NO_SUMMARY_TITLE),
    " + ".join(
        f"`{command}`"
        for command in _member_surface_commands(NO_SUMMARY_TITLE, _NO_SUMMARY_MEMBER[3])
    ),
)
NO_SUMMARY_EC_DIMENSION_ITEM = (
    f"{_member_ec_dimension(NO_SUMMARY_TITLE)}: "
    f"`{_member_ec_runner(NO_SUMMARY_TITLE)}`"
)

#: The other half: an item whose author wrote a description and no command at
#: all. A different capability from the command-only one, so the two arms are
#: never satisfied by the same section.
NO_COMMAND_TITLE = "Standard Operational Endpoints"
assert NO_COMMAND_TITLE != NO_SUMMARY_TITLE, (
    "the two partial shapes have to sit on different capabilities, or one "
    "section's assertion answers for both arms"
)
NO_COMMAND_SURFACE_ITEM = (
    f"{_member_surface_kind(NO_COMMAND_TITLE)}: "
    "the archetype health and readiness probe set."
)
NO_COMMAND_EC_DIMENSION_ITEM = (
    f"{_member_ec_dimension(NO_COMMAND_TITLE)}: "
    "the readiness probe contract, carried without a runner."
)
for _label, _item in (
    ("surface", NO_COMMAND_SURFACE_ITEM),
    ("EC dimension", NO_COMMAND_EC_DIMENSION_ITEM),
):
    assert "`" not in _item, (
        f"the summary-only {_label} must carry no backticked command, or "
        f"`extract_backtick_values` puts it back on the two-field arm"
    )
    assert " - " not in _item, (
        f"the summary-only {_label} must not carry the two-field separator "
        f"either, or the round trip is not the arm under test"
    )

#: The last arm of both renderers: an item that declares *neither* half.
#:
#: A third capability of this same document, because the two partial shapes
#: above answer for a command or a summary and neither answers for their
#: absence. `(true, true)` is the arm an author reaches by naming a surface kind
#: or a dimension and wiring nothing to it yet, and it is the arm that renders
#: the *bare* kind -- so blanking it does not shorten an item, it removes the
#: only thing identifying what the item was. For `EC Dimensions:` the bare
#: dimension name is exactly what `parse_ec_dimension_kind` reads back, so an
#: emptied item stops parsing as a dimension at all.
#:
#: Both round-27 fixes bound one arm each and left this one free: blanking it in
#: either renderer rendered the whole fixture byte for byte.
BARE_ITEM_TITLE = "Kubernetes-Native Deployment"
for _other in (NO_SUMMARY_TITLE, NO_COMMAND_TITLE):
    assert BARE_ITEM_TITLE != _other, (
        "the neither-half shape has to sit on a third capability, or one "
        "section's assertion answers for two arms at once"
    )
#: What the author writes, and what must come back. They differ for the surface
#: and not for the dimension, which is the product's own asymmetry:
#: `parse_capability_surfaces` needs the colon to read `Kubernetes` as a *kind*
#: (without it the piece is a summary under the default kind), while
#: `parse_capability_ec_dimensions` reads a colon-less piece as the dimension
#: name directly.
BARE_SURFACE_ITEM = _member_surface_kind(BARE_ITEM_TITLE)
BARE_SURFACE_DECLARED = f"{BARE_SURFACE_ITEM}:"
BARE_EC_DIMENSION_ITEM = _member_ec_dimension(BARE_ITEM_TITLE)
BARE_EC_DIMENSION_DECLARED = BARE_EC_DIMENSION_ITEM
for _label, _item in (
    ("surface", BARE_SURFACE_ITEM),
    ("EC dimension", BARE_EC_DIMENSION_ITEM),
):
    assert "`" not in _item and " - " not in _item and ":" not in _item, (
        f"the neither-half {_label} must render as the bare kind alone -- a "
        f"backtick puts it on a command arm, a `:` or ` - ` on a summary arm"
    )

#: `candidate`, and only for this capability.
#:
#: `validate_capability_contract` (`capability.rs:10156-10165`) requires a
#: contract-bearing status to carry at least one EC dimension *with content*,
#: and a neither-half dimension has none by construction
#: (`ec_dimension_has_content`, `capability.rs:7149-7153`). `candidate` is not
#: a workaround for that rule: it is the status the rule exists to exempt, and
#: the authoring state a bare dimension describes -- named, not yet wired.
#: Varied on one member rather than the document, so the other five still carry
#: their contract-bearing statuses and the arm is not reached by weakening the
#: whole input.
_PARTIAL_ITEM_STATUSES = tuple(
    "candidate" if member[0] == BARE_ITEM_TITLE else "verified"
    for member in _ALL_MEMBERS
)

PARTIAL_ITEM_OVERRIDES = {
    NO_SUMMARY_TITLE: {
        "surfaces": (NO_SUMMARY_SURFACE_ITEM,),
        "ec_dimensions": (NO_SUMMARY_EC_DIMENSION_ITEM,),
    },
    NO_COMMAND_TITLE: {
        "surfaces": (NO_COMMAND_SURFACE_ITEM,),
        "ec_dimensions": (NO_COMMAND_EC_DIMENSION_ITEM,),
    },
    BARE_ITEM_TITLE: {
        "surfaces": (BARE_SURFACE_ITEM,),
        "ec_dimensions": (BARE_EC_DIMENSION_ITEM,),
    },
}


def _partial_item_readme() -> str:
    document = _section_readme(
        _ALL_MEMBERS,
        (None,) * len(_ALL_MEMBERS),
        "Lumen README-resident capability contract, partial contract items.",
        statuses=_PARTIAL_ITEM_STATUSES,
    )
    for declared, replacement in (
        (MEMBER_SURFACE_ITEM[NO_SUMMARY_TITLE], NO_SUMMARY_SURFACE_ITEM),
        (MEMBER_EC_DIMENSION_ITEM[NO_SUMMARY_TITLE], NO_SUMMARY_EC_DIMENSION_ITEM),
        (MEMBER_SURFACE_ITEM[NO_COMMAND_TITLE], NO_COMMAND_SURFACE_ITEM),
        (MEMBER_EC_DIMENSION_ITEM[NO_COMMAND_TITLE], NO_COMMAND_EC_DIMENSION_ITEM),
        (MEMBER_SURFACE_ITEM[BARE_ITEM_TITLE], BARE_SURFACE_DECLARED),
        (MEMBER_EC_DIMENSION_ITEM[BARE_ITEM_TITLE], BARE_EC_DIMENSION_DECLARED),
    ):
        marker = f"- {declared}\n"
        assert document.count(marker) == 1, (declared, document.count(marker))
        document = document.replace(marker, f"- {replacement}\n", 1)
    assert " - " not in NO_SUMMARY_SURFACE_ITEM, (
        "the command-only item must carry no summary separator, or it is not "
        "the arm this document exists to reach"
    )
    return document


PARTIAL_ITEM_SECTION_README = _partial_item_readme()


def assert_relocation_carries_a_command_only_item(migrated: str) -> None:
    """An item declared as command-only round-trips as command-only.

    Two things have to hold and neither implies the other. The item must still
    be *there* -- the arm that renders it is separately deletable, and deleting
    it makes migration silently drop a declared surface. And it must not have
    acquired a summary the author never wrote, which is what an implementation
    that fell through to the two-field arm with an empty summary produces: a
    trailing ` - ` that reads as an empty description.

    Asserted on the exact item rather than by containment of the command,
    because the command string alone also appears in the capability's work-root
    and gate-inventory cells.
    """
    body = _capability_section_body(migrated, NO_SUMMARY_TITLE)
    for label, expected in (
        ("surface", f"Surfaces:\n- {NO_SUMMARY_SURFACE_ITEM}\n"),
        ("EC dimension", f"EC Dimensions:\n- {NO_SUMMARY_EC_DIMENSION_ITEM}\n"),
    ):
        assert expected in body, (
            f"relocated section {NO_SUMMARY_TITLE!r} lost its command-only "
            f"{label}; expected {expected!r}, section was:\n{body}"
        )


def assert_relocation_carries_a_summary_only_item(migrated: str) -> None:
    """An item declared as summary-only keeps its kind, and gains no command.

    The mirror of the leg above, and the arm it does not reach. Both renderers
    answer a command-less item with `kind: summary`, and the prefix is the whole
    of what that arm adds: dropping it leaves the summary alone on the line,
    which for `EC Dimensions:` is an item the next parse no longer recognises as
    a dimension.

    Asserted on the exact item, so an implementation that fell through to the
    command-bearing arm -- rendering a stray `: ` or an empty pair of backticks
    -- is a different document too.
    """
    body = _capability_section_body(migrated, NO_COMMAND_TITLE)
    for label, expected in (
        ("surface", f"Surfaces:\n- {NO_COMMAND_SURFACE_ITEM}\n"),
        ("EC dimension", f"EC Dimensions:\n- {NO_COMMAND_EC_DIMENSION_ITEM}\n"),
    ):
        assert expected in body, (
            f"relocated section {NO_COMMAND_TITLE!r} lost its summary-only "
            f"{label}; expected {expected!r}, section was:\n{body}"
        )


def assert_relocation_carries_a_bare_item(migrated: str) -> None:
    """An item declared with neither half round-trips as the bare kind.

    The fourth arm, and the one the two legs above cannot reach: they each
    supply one half, so an implementation may answer both while rendering
    nothing at all for an item that supplies neither. Blanking that arm passed
    every assertion this fixture had before -- no other capability declares an
    item with neither half, so the arm was never entered.

    The bare kind is the *whole* of the item, which makes the failure mode
    sharper than the partial legs: there is nothing left to identify what was
    declared. For `EC Dimensions:` an emptied item also stops parsing as a
    dimension on the next read, so the loss compounds rather than showing up as
    a shorter line.

    Asserted as the exact `\\n`-terminated item, so an implementation that fell
    through to a neighbouring arm -- a trailing `: `, an empty pair of
    backticks, a stray ` - ` -- is a different document too. Asserted against
    the section body so the bare kind cannot be satisfied by the same word
    appearing in the capability's `Type:` line or a work-root cell.
    """
    body = _capability_section_body(migrated, BARE_ITEM_TITLE)
    for label, expected in (
        ("surface", f"Surfaces:\n- {BARE_SURFACE_ITEM}\n"),
        ("EC dimension", f"EC Dimensions:\n- {BARE_EC_DIMENSION_ITEM}\n"),
    ):
        assert expected in body, (
            f"relocated section {BARE_ITEM_TITLE!r} lost its bare {label}; "
            f"expected {expected!r}, section was:\n{body}"
        )


def assert_relocation_carries_every_list_item(migrated: str) -> None:
    """A capability's list fields keep *every* item, in order, not just the first.

    The three list renderers (`capability.rs:9039-9054`) each map over their
    whole vector. With one item per list everywhere else, `.take(1)` on any of
    them was indistinguishable from rendering all of them.

    Asserted as the exact item list each field renders, so an implementation that
    renders both but reorders them, or renders a third the author never declared,
    is a different document too. The earlier form asserted the two items as a
    *containing* block, which pins what the field starts with and nothing after
    it: appending a duplicate of the first item after the render loop passed.
    """
    body = _capability_section_body(migrated, MULTI_ITEM_TITLE)
    overrides = MULTI_ITEM_OVERRIDES[MULTI_ITEM_TITLE]
    for field in LIST_FIELDS:
        expected_items = list(overrides[_OVERRIDE_KEYS[field]])
        rendered = _field_list_items(body, MULTI_ITEM_TITLE, field)
        assert rendered == expected_items, (
            f"relocated section {MULTI_ITEM_TITLE!r} must render the {field} "
            f"items {expected_items!r}, in that order and nothing else; got "
            f"{rendered!r}. Section was:\n{body}"
        )


def _normalized_kind_token(value: str) -> str:
    """The product's `normalize_table_token` (`capability.rs:11946-11953`).

    Restated rather than approximated: the spelling comparison below is only
    meaningful if it folds a token the same way the parse does.
    """
    return "".join(
        ch for ch in value.strip().strip("`") if ch.isascii() and ch.isalnum()
    ).lower()


#: Canonical surface kind -> every spelling `normalize_surface_kind`
#: (`capability.rs:11692-11703`) folds into it.
#:
#: Restated from the product rather than derived from it, so an arm that quietly
#: disappears is a failing expectation instead of a narrower one.
SURFACE_KIND_ALIASES = {
    "CLI": ("cli", "command", "commands"),
    "HTTP": ("http", "api", "rest"),
    "SDK": ("sdk",),
    "UI": ("ui", "webui", "web"),
    "Config": ("config", "configuration"),
    "FileFormat": ("fileformat", "file", "format"),
}

#: Canonical EC dimension kind -> every spelling `parse_ec_dimension_kind`
#: (`capability.rs:11705-11715`) folds into it, keyed in
#: `CapabilityEcDimensionKind` declaration order (`capability.rs:654-659`),
#: which is the order `dedupe_ec_dimensions`' `BTreeMap` renders them in.
EC_DIMENSION_ALIASES = {
    "behavior": ("behavior", "behaviour", "functional", "function", "render"),
    "efficiency": ("efficiency", "performance", "perf"),
    "security": ("security", "secure"),
    "stability": ("stability", "resilience", "reliability"),
}
assert set(EC_DIMENSION_ALIASES) == set(MEMBER_EC_DIMENSION.values()), (
    "EC_DIMENSION_ALIASES must be keyed by the same canonical vocabulary the "
    "members declare, or the two maps describe different enums"
)

#: The spellings the other documents already author, and therefore already bind:
#: `HTTP` and `CLI` fold through the arm that names them, and every member's EC
#: dimension is written in its canonical spelling.
_AUTHORED_SURFACE_SPELLINGS = frozenset(
    _normalized_kind_token(MEMBER_SURFACE_KIND[member[0]]) for member in _ALL_MEMBERS
)
_AUTHORED_EC_DIMENSION_SPELLINGS = frozenset(
    MEMBER_EC_DIMENSION[member[0]] for member in _ALL_MEMBERS
)

#: Alias spelling -> the canonical kind it must fold into, for every spelling no
#: other document writes. These are the arms this document exists to reach: with
#: only the canonical spelling ever authored, each `|` alternative in both match
#: tables was deletable without changing any rendered byte.
_SURFACE_ALIASES_TO_BIND = {
    alias: canonical
    for canonical, aliases in SURFACE_KIND_ALIASES.items()
    for alias in aliases
    if alias not in _AUTHORED_SURFACE_SPELLINGS
}
_EC_DIMENSION_ALIASES_TO_BIND = {
    alias: canonical
    for canonical, aliases in EC_DIMENSION_ALIASES.items()
    for alias in aliases
    if alias not in _AUTHORED_EC_DIMENSION_SPELLINGS
}

#: Title -> the alias spellings its `Surfaces:` items are authored with. A
#: capability may declare several, because 13 unbound spellings do not fit one
#: per capability across six members and a second document would cost a second
#: migration run to bind three more arms.
ALIAS_SPELLING_SURFACES = {
    _ALL_MEMBERS[0][0]: ("api", "rest", "sdk"),
    _ALL_MEMBERS[1][0]: ("ui", "command", "commands"),
    _ALL_MEMBERS[2][0]: ("webui", "web", "config"),
    _ALL_MEMBERS[3][0]: ("configuration", "file", "format"),
    _ALL_MEMBERS[4][0]: ("fileformat",),
}

#: Surface-kind spellings `parse_markdown_contract_field_line`
#: (`capability.rs:10920`) also accepts as the *name* of the `Surfaces:` field.
#:
#: An item written as its own `- command: ...` line is read as a second
#: declaration of the field rather than as an item of it: the kind token is
#: consumed as the field name and only the remainder is kept, so the item
#: reaches `normalize_surface_kind` with the `("CLI", piece)` default already
#: substituted for the kind the author wrote. The rendered document is right by
#: coincidence -- the default happens to be the kind those two spellings fold
#: into -- which is exactly why a mutant that deleted both arms survived a
#: document declaring them one per line.
#:
#: Declared behind a spelling that is not a field key, on a `;`-joined line, the
#: token survives to the fold and the arms are entered.
FIELD_KEY_COLLIDING_SURFACE_SPELLINGS = frozenset({"command", "commands"})
_SINGLE_LINE_SURFACE_TITLES = frozenset(
    title
    for title, aliases in ALIAS_SPELLING_SURFACES.items()
    if FIELD_KEY_COLLIDING_SURFACE_SPELLINGS & set(aliases)
)
for _title in _SINGLE_LINE_SURFACE_TITLES:
    assert (
        ALIAS_SPELLING_SURFACES[_title][0]
        not in FIELD_KEY_COLLIDING_SURFACE_SPELLINGS
    ), (
        f"{_title!r} leads its `;`-joined surface line with a spelling that is "
        f"also a field key, which puts the whole line back on the branch that "
        f"eats the kind token"
    )

#: Title -> the alias spellings its `EC Dimensions:` items are authored with.
#:
#: No capability names two aliases of the *same* canonical kind: those two
#: dimensions would be merged by `dedupe_ec_dimensions` into one rendered item,
#: which is a different rule with a different expectation. Within a capability
#: the spellings are deliberately *not* in canonical order, because the rendered
#: order comes from a `BTreeMap` over the enum rather than from the document --
#: authoring them in order would leave a parse that preserved document order
#: rendering the identical list.
ALIAS_SPELLING_EC_DIMENSIONS = {
    _ALL_MEMBERS[0][0]: ("secure", "behaviour", "resilience", "perf"),
    _ALL_MEMBERS[1][0]: ("functional", "performance", "reliability"),
    _ALL_MEMBERS[2][0]: ("function",),
    _ALL_MEMBERS[3][0]: ("render",),
}
assert {
    alias for aliases in ALIAS_SPELLING_SURFACES.values() for alias in aliases
} == set(_SURFACE_ALIASES_TO_BIND), (
    "the alias document must author every surface-kind spelling no other "
    "document authors, or the arms it leaves out stay deletable"
)
assert {
    alias for aliases in ALIAS_SPELLING_EC_DIMENSIONS.values() for alias in aliases
} == set(_EC_DIMENSION_ALIASES_TO_BIND), (
    "the alias document must author every EC dimension spelling no other "
    "document authors, or the arms it leaves out stay deletable"
)
for _title, _aliases in ALIAS_SPELLING_EC_DIMENSIONS.items():
    _kinds = [_EC_DIMENSION_ALIASES_TO_BIND[_alias] for _alias in _aliases]
    assert len(set(_kinds)) == len(_kinds), (
        f"{_title!r} declares two spellings of the same EC dimension kind; "
        f"`dedupe_ec_dimensions` merges those into one item and the expectation "
        f"below would be asserting the merge instead of the alias table"
    )
assert (
    [
        _EC_DIMENSION_ALIASES_TO_BIND[_alias]
        for _alias in ALIAS_SPELLING_EC_DIMENSIONS[_ALL_MEMBERS[0][0]]
    ]
    != sorted(
        (
            _EC_DIMENSION_ALIASES_TO_BIND[_alias]
            for _alias in ALIAS_SPELLING_EC_DIMENSIONS[_ALL_MEMBERS[0][0]]
        ),
        key=tuple(EC_DIMENSION_ALIASES).index,
    )
), (
    "at least one capability must author its dimensions out of canonical order, "
    "or a parse that preserved document order renders the identical list"
)


def _alias_surface_item(cap_id: str, alias: str) -> tuple[str, str]:
    """The `(authored, rendered)` pair for one aliased surface item.

    Only the leading kind token differs. Commands and summary are carried
    through verbatim, so an implementation that canonicalised the whole item --
    or rewrote the summary to name the canonical kind -- is a different
    document.
    """
    body = f"`aw {cap_id} --{alias}` - {cap_id} surface declared as {alias}."
    return f"{alias}: {body}", f"{_SURFACE_ALIASES_TO_BIND[alias]}: {body}"


def _alias_ec_dimension_item(cap_id: str, alias: str) -> tuple[str, str]:
    """The `(authored, rendered)` pair for one aliased EC dimension item."""
    body = f"`{cap_id}-{alias}-gate` - {cap_id} {alias} gate."
    return f"{alias}: {body}", f"{_EC_DIMENSION_ALIASES_TO_BIND[alias]}: {body}"


_ALIAS_CAP_ID = {member[0]: member[1] for member in _ALL_MEMBERS}
_ALIAS_AUTHORED_SURFACES = {}
_ALIAS_RENDERED_SURFACES = {}
for _title, _aliases in ALIAS_SPELLING_SURFACES.items():
    _pairs = [_alias_surface_item(_ALIAS_CAP_ID[_title], _alias) for _alias in _aliases]
    # `dedupe_surfaces` (`capability.rs:11676-11689`) keys on the normalized
    # kind plus the commands plus the summary, so items that fold to the same
    # kind survive only because each carries its own command and summary. It
    # filters in place rather than sorting, so the rendered order is the
    # authored order.
    _ALIAS_AUTHORED_SURFACES[_title] = tuple(pair[0] for pair in _pairs)
    _ALIAS_RENDERED_SURFACES[_title] = tuple(pair[1] for pair in _pairs)

_ALIAS_AUTHORED_EC_DIMENSIONS = {}
_ALIAS_RENDERED_EC_DIMENSIONS = {}
for _title, _aliases in ALIAS_SPELLING_EC_DIMENSIONS.items():
    _pairs = [
        (_alias, _alias_ec_dimension_item(_ALIAS_CAP_ID[_title], _alias))
        for _alias in _aliases
    ]
    _ALIAS_AUTHORED_EC_DIMENSIONS[_title] = tuple(pair[1][0] for pair in _pairs)
    _ALIAS_RENDERED_EC_DIMENSIONS[_title] = tuple(
        pair[1][1]
        for pair in sorted(
            _pairs,
            key=lambda pair: tuple(EC_DIMENSION_ALIASES).index(
                _EC_DIMENSION_ALIASES_TO_BIND[pair[0]]
            ),
        )
    )

#: Per-capability rendered expectations for the alias document, in the
#: `item_overrides` shape `assert_sections_carry_their_own_contract` takes.
#:
#: `Gate Inventory:` is deliberately absent from every entry: it carries no kind
#: token, so it stays on the default single-item expectation and this document
#: keeps binding it alongside the two it varies.
ALIAS_SPELLING_OVERRIDES = {
    title: {
        key: items
        for key, items in (
            ("surfaces", _ALIAS_RENDERED_SURFACES.get(title)),
            ("ec_dimensions", _ALIAS_RENDERED_EC_DIMENSIONS.get(title)),
        )
        if items is not None
    }
    for title in set(ALIAS_SPELLING_SURFACES) | set(ALIAS_SPELLING_EC_DIMENSIONS)
}


def _alias_spelling_readme() -> str:
    """A README whose surface and EC dimension kinds are written as aliases.

    Both kind fields are parsed through a fold table -- `normalize_surface_kind`
    for surfaces, `parse_ec_dimension_kind` for EC dimensions -- and every other
    document in this fixture authors only the spelling the table returns. That
    made each table a near-identity on this fixture's inputs: deleting the
    `"api" | "rest"` alternatives, or the `"behaviour" | "functional"` ones,
    rendered a byte-identical document, because no input ever took those arms.

    The EC dimension table fails *silently* when an arm is missing: an
    unrecognized kind returns `None` and the item is dropped, not carried
    through unfolded. So an alias whose arm is gone does not render a differently
    spelled item -- it renders no item at all, and only an expectation that
    names the exact list catches it.

    Built by rewriting the canonical items in place, the same way the
    partial-item document is: the alias affects the authored kind token only, so
    the rendered expectation stays the canonical item and no new declaration
    machinery is needed.

    Two members keep a passthrough kind (`Identity`, `MCP`), which the fold table
    does not name and returns unchanged, so this document also holds the `_ =>`
    arm against the aliased ones.

    The `command` and `commands` spellings are declared on a `;`-joined line
    rather than one per line, for the reason on
    `FIELD_KEY_COLLIDING_SURFACE_SPELLINGS`: written as their own line they are
    read as the *name* of the field and never reach the fold at all, and the
    default the parse substitutes is the very kind they would have folded into,
    so a document declaring them per line renders correctly with both arms
    deleted. That is the shape this document exists to rule out, so getting it
    wrong here would have been the defect it is written against.
    """
    document = _section_readme(
        _ALL_MEMBERS,
        (None,) * len(_ALL_MEMBERS),
        "Lumen README-resident capability contract, aliased kind spellings.",
    )
    for declared_by_title, authored_by_title, single_line_titles in (
        (MEMBER_SURFACE_ITEM, _ALIAS_AUTHORED_SURFACES, _SINGLE_LINE_SURFACE_TITLES),
        (MEMBER_EC_DIMENSION_ITEM, _ALIAS_AUTHORED_EC_DIMENSIONS, frozenset()),
    ):
        for title, authored in authored_by_title.items():
            marker = f"- {declared_by_title[title]}\n"
            assert document.count(marker) == 1, (title, document.count(marker))
            if title in single_line_titles:
                block = "- " + "; ".join(authored) + "\n"
            else:
                block = "".join(f"- {item}\n" for item in authored)
            document = document.replace(marker, block, 1)
    return document


ALIAS_SPELLING_SECTION_README = _alias_spelling_readme()


#: The two capabilities whose EC dimension is declared as two same-kind halves,
#: and the order each declares them in.
#:
#: `dedupe_ec_dimensions` (`capability.rs:11652-11674`) collapses a capability's
#: dimensions into a `BTreeMap` keyed by kind, and fills the first occurrence's
#: empty fields from the later ones: an empty `runner` takes the later item's
#: runner, an empty `summary` takes the later item's summary. Every document in
#: this fixture declares at most one item per kind, so the map never merged
#: anything and both fills were deletable without changing a rendered byte.
#:
#: The two fills are separately observable only in opposite declaration orders,
#: because the fill always runs on the *first* occurrence. `Search Core` writes
#: the summary half first, so only the runner fill has work to do; `Contract Gate
#: Wiring` writes the command half first, so only the summary fill does. One
#: capability binds one fill and leaves the other free.
SAME_KIND_EC_DIMENSION_ORDER = {
    "Search Core": ("summary", "command"),
    "Contract Gate Wiring": ("command", "summary"),
}
assert len({order for order in SAME_KIND_EC_DIMENSION_ORDER.values()}) == 2, (
    "the two capabilities must declare their halves in opposite orders, or one "
    "of the two fills is never the reason for the merged item"
)
assert len({_member_ec_dimension(title) for title in SAME_KIND_EC_DIMENSION_ORDER}) == 2, (
    "the two capabilities must use different dimension kinds, or their merges "
    "share a map entry in a document that declares them both"
)


def _same_kind_ec_dimension_halves(title: str) -> tuple[str, ...]:
    """The two half-items whose merge must reconstruct the canonical item.

    Neither half is a legal contract item on its own: one carries a summary with
    no runner, the other a runner with no summary. Their merge is asserted as the
    *default* single-item expectation, with no override, because the canonical
    item is exactly what a correct merge produces -- so a merge that dropped
    either half renders a shorter item, and a merge that did not happen at all
    renders two items where one is expected.
    """
    dimension = _member_ec_dimension(title)
    cap_id = _ALIAS_CAP_ID[title]
    halves = {
        "summary": f"{dimension}: {cap_id} {dimension} gate.",
        "command": f"{dimension}: `{_member_ec_runner(title)}`",
    }
    return tuple(halves[half] for half in SAME_KIND_EC_DIMENSION_ORDER[title])


for _same_kind_title in SAME_KIND_EC_DIMENSION_ORDER:
    _same_kind_halves = _same_kind_ec_dimension_halves(_same_kind_title)
    assert len(_same_kind_halves) == 2, _same_kind_title
    for _same_kind_half in _same_kind_halves:
        assert _same_kind_half != MEMBER_EC_DIMENSION_ITEM[_same_kind_title], (
            f"{_same_kind_title!r} must declare halves, not the whole item, or "
            f"the merge has nothing to reconstruct"
        )
        assert _same_kind_half.startswith(
            f"{_member_ec_dimension(_same_kind_title)}:"
        ), (_same_kind_title, _same_kind_half)
    assert "`" not in _same_kind_halves[
        SAME_KIND_EC_DIMENSION_ORDER[_same_kind_title].index("summary")
    ], f"{_same_kind_title!r}'s summary half must carry no runner"


def _same_kind_ec_dimension_readme() -> str:
    """A README in which two capabilities split one dimension across two items.

    Built by rewriting the canonical declaration in place, the way the alias and
    partial-item documents are, so nothing about the rest of the contract moves
    and the merge is the only difference from the baseline relocation input.

    The two capabilities are chosen for their dimension kinds -- `behavior` and
    `stability` -- so the merged entries occupy different keys of the same map,
    and for their opposite declaration orders, so each of the two fills is the
    sole reason for its own merged item.
    """
    document = _section_readme(
        _ALL_MEMBERS,
        (None,) * len(_ALL_MEMBERS),
        "Lumen README-resident capability contract, split EC dimension items.",
    )
    for title, halves in (
        (title, _same_kind_ec_dimension_halves(title))
        for title in SAME_KIND_EC_DIMENSION_ORDER
    ):
        marker = f"- {MEMBER_EC_DIMENSION_ITEM[title]}\n"
        assert document.count(marker) == 1, (title, document.count(marker))
        document = document.replace(
            marker, "".join(f"- {half}\n" for half in halves), 1
        )
    for title in SAME_KIND_EC_DIMENSION_ORDER:
        assert MEMBER_EC_DIMENSION_ITEM[title] not in document, (
            f"{title!r} must declare only halves in this document, or the merged "
            f"item is present in the input and the expectation is not a claim "
            f"about the merge"
        )
    return document


SAME_KIND_EC_DIMENSION_SECTION_README = _same_kind_ec_dimension_readme()


#: Relocation input in which three capabilities declare no work-root table.
#:
#: A capability's work-root table is rendered by four blocks
#: (`capability.rs:9065-9126`), not one, and every document above declares at
#: least one work root for every capability -- which is the first block. The
#: three others synthesize a row when the table would otherwise be empty, and
#: none of them was ever entered here: deleting all three left every rendered
#: byte of every document unchanged.
#:
#: Two capabilities keep a live `Root WI`, so the section parse pushes a
#: synthetic gap (`capability.rs:10724-10731`) and the *gap* block renders the
#: row; a third blanks it, so there is no gap either and the fully synthesized
#: `{title} root` row is the only thing its table carries. Two gap-derived rows
#: rather than one because the verification cell is a fold over `(gap status,
#: capability status)` (`capability.rs:9916-9927`) whose `verified` arm and
#: whose `planned` fallback would otherwise be the same string on every row of
#: the document.
_NO_WORK_ROOT_GAP_MEMBERS = (_ALL_MEMBERS[2], _ALL_MEMBERS[3])
_NO_WORK_ROOT_SYNTHETIC_MEMBER = _ALL_MEMBERS[4]
#: The one capability whose gap-derived row must not read `verified`.
NO_WORK_ROOT_PLANNED_TITLE = _NO_WORK_ROOT_GAP_MEMBERS[1][0]
NO_WORK_ROOT_STATUSES = tuple(
    "candidate" if member[0] == NO_WORK_ROOT_PLANNED_TITLE else "verified"
    for member in _ALL_MEMBERS
)
#: The capability that declares no tracker state either, and so renders `-`.
NO_WORK_ROOT_BLANKED_TITLES = frozenset({_NO_WORK_ROOT_SYNTHETIC_MEMBER[0]})
#: Every capability that declares no work-root table, and therefore no claims.
NO_WORK_ROOT_TITLES = frozenset(
    member[0]
    for member in _NO_WORK_ROOT_GAP_MEMBERS + (_NO_WORK_ROOT_SYNTHETIC_MEMBER,)
)


def _work_root_table(member: tuple[Any, ...]) -> str:
    """The work-root table `_capability` writes for one member, verbatim."""
    rows = "".join(
        f"| {root} | change | - | implemented | verified | smoke | `true` |\n"
        for root in member[4]
    )
    return (
        "| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |\n"
        "|---|---|---:|---|---|---|---|\n"
        f"{rows}"
    )


def _no_work_root_readme() -> str:
    document = _section_readme(
        _ALL_MEMBERS,
        (None,) * len(_ALL_MEMBERS),
        "Lumen README-resident capability contract, some work roots undeclared.",
        statuses=NO_WORK_ROOT_STATUSES,
    )
    for member in _NO_WORK_ROOT_GAP_MEMBERS + (_NO_WORK_ROOT_SYNTHETIC_MEMBER,):
        table = _work_root_table(member)
        assert document.count(table) == 1, (member[0], document.count(table))
        document = document.replace(table, "", 1)
    blanked = f"Root WI: {SECTION_RELOCATION_WI[_NO_WORK_ROOT_SYNTHETIC_MEMBER[0]]}\n"
    assert document.count(blanked) == 1, blanked
    return document.replace(blanked, "Root WI: -\n", 1)


NO_WORK_ROOT_SECTION_README = _no_work_root_readme()

_NO_WORK_ROOT_STATUS_BY_TITLE = dict(
    zip((member[0] for member in _ALL_MEMBERS), NO_WORK_ROOT_STATUSES)
)


def _no_work_root_table(member: tuple[Any, ...]) -> str:
    """The whole work-root table this document must render for one capability."""
    title, cap_id = member[0], member[1]
    inventory = f"tech-design/{cap_id}.md"
    if title in NO_WORK_ROOT_BLANKED_TITLES:
        # No work roots, no claims and no gaps either: the row is synthesized
        # from the capability itself, and its WI is the `-` the fallback
        # produces when there is no gap to read one off.
        rows = f"| {title} root | epic | - | planned | planned | smoke | {inventory} |\n"
    elif title in NO_WORK_ROOT_TITLES:
        verification = (
            "verified"
            if _NO_WORK_ROOT_STATUS_BY_TITLE[title] == "verified"
            else "planned"
        )
        rows = (
            f"| {title} root work | epic | {SECTION_RELOCATION_WI[title]} | "
            f"planned | {verification} | smoke | {inventory} |\n"
        )
    else:
        rows = "".join(
            f"| {root} | change | - | implemented | verified | smoke | `true` |\n"
            for root in member[4]
        )
    return (
        "| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |\n"
        "|---|---|---:|---|---|---|---|\n"
        f"{rows}"
    )


#: The whole rendered table, per capability, for the no-work-root document.
NO_WORK_ROOT_TABLES = {
    member[0]: _no_work_root_table(member) for member in _ALL_MEMBERS
}
assert (
    len(
        {
            NO_WORK_ROOT_TABLES[title].splitlines()[2]
            for title in NO_WORK_ROOT_TITLES
        }
    )
    == 3
), "the three synthesized rows must be pairwise distinct"


def _rendered_work_root_table(migrated: str, title: str) -> str:
    body = _capability_section_body(migrated, title)
    marker = "| Work Root |"
    assert body.count(marker) == 1, (
        f"relocated section {title!r} must render exactly one work-root table, "
        f"section was:\n{body}"
    )
    lines: list[str] = []
    for line in body[body.index(marker) :].splitlines():
        if not line.startswith("|"):
            break
        lines.append(line)
    return "".join(f"{line}\n" for line in lines)


def assert_relocation_synthesizes_an_absent_work_root_table(migrated: str) -> None:
    """A capability that declares no work root still renders a described row.

    The work-root table is written by four blocks (`capability.rs:9065-9126`),
    and every other document here gives every capability at least one work root
    -- which is the first block, and the only one those documents can enter. The
    other three synthesize the table when it would otherwise be empty, and all
    three could be deleted without changing a rendered byte anywhere in this
    case.

    Two of the three are reachable from a section-shaped README. A capability
    with no work-root table but a live `Root WI` gets a synthetic gap
    (`capability.rs:10724-10731`), so the *gap* block describes it, carrying the
    tracker state and a verification cell folded from the capability's status.
    One that declares neither has no gap either, and the last block names it
    from the capability's own title with `-` for a WI. The third block, which
    renders one row per contract claim, is unreachable this way and is bound on
    the YAML leg instead.

    Asserted as the whole table per capability, not as a substring: these rows
    are *added* by independent `if` blocks, so a condition that stopped
    excluding the others would append a second row to a table that already had
    the right one, and any assertion reading a single row would still pass. The
    four capabilities that do declare work roots are asserted here too, for the
    same reason from the other side -- their tables must stay exactly what they
    authored.

    Note that this document's `claim_count` is the same integer as every other
    one's: a synthesized row is a claim when the emitted document is read back,
    so the count cannot distinguish a row that was described from a row that was
    invented. The table text is the only place this behavior is observable.
    """
    for member in _ALL_MEMBERS:
        title = member[0]
        rendered = _rendered_work_root_table(migrated, title)
        expected = NO_WORK_ROOT_TABLES[title]
        assert rendered == expected, (
            f"relocated section {title!r} rendered the wrong work-root table;\n"
            f"expected:\n{expected}\ngot:\n{rendered}"
        )


#: The note `aw capability migrate` writes under a `## Capabilities` heading it
#: had to supply itself.
CANONICAL_CAPABILITIES_NOTE = (
    "Canonical field-style capability contracts below are machine-readable input "
    "for `aw capability`; YAML and legacy tables are migration input only."
)
#: The placeholder the frame repair leaves where a human has to write the brief.
CANONICAL_BRIEF_TODO = (
    "<!-- TODO: Add the human-confirmed project brief before publishing. -->"
)


def _frame_capability(cap_id: str) -> str:
    return (
        f"### {cap_id.title()}\n\n"
        f"ID: {cap_id}\n"
        f"Root WI: #1\n"
        f"Status: candidate\n"
        f"Promise:\n"
        f"Probe promise for {cap_id}.\n"
    )


#: Format-migration inputs whose canonical frame is incomplete, one defect each.
#:
#: `ensure_canonical_readme_scaffold` (`capability.rs:8647-8662`) supplies a
#: missing `# <Project>` title, a missing `## Brief`, and a missing
#: `## Capabilities` heading. Every other document in this case arrives with all
#: three, so all three repairs could be disabled at once without changing a
#: rendered byte -- migration would quietly emit a document with no title and no
#: brief, and the relocation leg's exact-frame assertion, which pins the frame
#: relocation *builds*, says nothing about the frame migration *repairs*.
#:
#: The brief repair has two arms and both are declared here: a title followed by
#: lead prose becomes a `## Brief` carrying that prose, and a title followed by
#: nothing gets the human-confirmation placeholder instead.
FRAME_NO_TITLE_BRIEF = "Probe: the title is missing."
FRAME_NO_TITLE_DOCUMENT = (
    f"## Brief\n\n{FRAME_NO_TITLE_BRIEF}\n\n"
    f"## Capabilities\n\n{_frame_capability('alpha')}"
)
FRAME_LEAD_PROSE = "Lead prose that must become the Brief."
FRAME_LEAD_PROSE_DOCUMENT = (
    f"# Demo\n\n{FRAME_LEAD_PROSE}\n\n"
    f"## Capabilities\n\n{_frame_capability('beta')}"
)
FRAME_NO_BRIEF_DOCUMENT = f"# Demo\n\n## Capabilities\n\n{_frame_capability('gamma')}"
#: The one input that arrives with no `## Capabilities` heading at all.
#:
#: Its authored brief prose does not survive the migration: the strip that
#: removes the capability sections takes the whole prefix with it when there is
#: no `## Capabilities` heading to stop at, and the repair then writes the
#: placeholder over it. That is a product defect, reported as #3234 and
#: deliberately not asserted here in either direction -- asserting the loss
#: would pin the defect in place, and asserting its absence would fail on the
#: unfixed product. What is asserted is the heading repair alone.
FRAME_NO_CAPABILITIES_DOCUMENT = (
    f"# Demo\n\n## Brief\n\nAuthored brief prose.\n\n{_frame_capability('delta')}"
)

_FRAME_TAIL = f"## Capabilities\n\n{CANONICAL_CAPABILITIES_NOTE}\n\n"
#: The exact frame each of those inputs must be repaired into, up to the index.
FRAME_REPAIRS: dict[str, tuple[str, str]] = {
    "no_title": (
        FRAME_NO_TITLE_DOCUMENT,
        f"# Demo\n\n## Brief\n\n{FRAME_NO_TITLE_BRIEF}\n\n{_FRAME_TAIL}",
    ),
    "lead_prose": (
        FRAME_LEAD_PROSE_DOCUMENT,
        f"# Demo\n\n## Brief\n\n{FRAME_LEAD_PROSE}\n\n{_FRAME_TAIL}",
    ),
    "no_brief": (
        FRAME_NO_BRIEF_DOCUMENT,
        f"# Demo\n\n## Brief\n\n{CANONICAL_BRIEF_TODO}\n\n{_FRAME_TAIL}",
    ),
    "no_capabilities_heading": (
        FRAME_NO_CAPABILITIES_DOCUMENT,
        f"# Demo\n\n## Brief\n\n{CANONICAL_BRIEF_TODO}\n\n{_FRAME_TAIL}",
    ),
}
assert not FRAME_NO_TITLE_DOCUMENT.startswith("# "), "the no-title input has a title"
assert "## Brief" not in FRAME_LEAD_PROSE_DOCUMENT
assert "## Brief" not in FRAME_NO_BRIEF_DOCUMENT
assert "## Capabilities" not in FRAME_NO_CAPABILITIES_DOCUMENT
for _frame_label, (_frame_input, _frame_output) in FRAME_REPAIRS.items():
    assert CANONICAL_CAPABILITIES_NOTE not in _frame_input, _frame_label
    assert CANONICAL_BRIEF_TODO not in _frame_input, _frame_label
#: The three repairs are distinguishable: no two inputs share an expected frame
#: except the two that legitimately converge on the placeholder brief.
assert len({expected for _, expected in FRAME_REPAIRS.values()}) == 3


def assert_format_migration_repairs_the_canonical_frame(
    migrated: str, *, expected: str, label: str
) -> None:
    """The frame migration has to supply is asserted as one exact block.

    Asserted as the whole prefix up to `### Capability Index` rather than as
    three `in` checks, for the reason the relocation frame is: containment binds
    that each heading appears somewhere, not that they appear once, in order,
    and with the body each is required to carry. The brief in particular is
    either the author's own prose or the human-confirmation placeholder, and
    those two are the same assertion under containment.
    """
    marker = "### Capability Index"
    assert migrated.count(marker) == 1, (
        f"[{label}] the migrated document must carry exactly one capability "
        f"index, document was:\n{migrated}"
    )
    frame = migrated[: migrated.index(marker)]
    assert frame == expected, (
        f"[{label}] migration repaired the canonical frame wrongly;\n"
        f"expected:\n{expected!r}\ngot:\n{frame!r}"
    )


#: A document whose capability contract has not been written yet, shaped so the
#: renderer actually runs on it.
#:
#: `render_capability_registry` (`capability.rs:8596-8598`) breaks out of the
#: two-root loop when a document declares neither a capability section nor a
#: legacy row, so an empty registry does not acquire a `### Core Features` and a
#: `### Non-Core Features` root with no members. Every other document in this
#: case declares one or the other, so without this leg the guard was entered
#: only with something to render.
#:
#: The obvious input -- a bare `## Capabilities` heading with nothing under it --
#: does not reach the guard at all. `apply_capability_format_migration_tick`
#: (`capability.rs:13010`) returns "already uses canonical Markdown capability
#: format" before rendering anything, because neither
#: `requires_format_migration` nor `requires_feature_class_migration` fires on a
#: document with no contract to migrate. A mutation that deletes the guard
#: survives such an input: the document is byte-identical either way, for a
#: reason that has nothing to do with the guard.
#:
#: The legacy `## Capability Index` heading is what makes the input reach it.
#: `markdown_capability_document_needs_canonicalization`
#: (`capability.rs:10294-10296`) treats a level-2 `Capability Index` as
#: non-canonical on its own, so migration runs, renders, and takes the guarded
#: branch with both `capabilities` and `legacy_rows` empty.
EMPTY_REGISTRY_DOCUMENT = """# Demo

## Brief

A project whose capability contract has not been written yet.

## Capabilities

## Capability Index

## Contributing

See CONTRIBUTING.md.
"""

#: What migration writes for `EMPTY_REGISTRY_DOCUMENT`, byte for byte.
#:
#: Three separate rules are pinned by this one string. The legacy `## Capability
#: Index` heading is demoted to the canonical `### Capability Index`. The index
#: table is emitted with its header row and a single synthesized placeholder row
#: named after the project (`capability.rs:8990-8994`), which is the product's
#: answer to "an index with no capabilities to list". And no feature root
#: appears anywhere -- the guard's branch.
EMPTY_REGISTRY_MIGRATED = (
    "# Demo\n"
    "\n"
    "## Brief\n"
    "\n"
    "A project whose capability contract has not been written yet.\n"
    "\n"
    "## Capabilities\n"
    "\n"
    "### Capability Index\n"
    "\n"
    "| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |\n"
    "|---|---:|---|---|---|---|---|\n"
    "| Demo Capability | - | planned | planned | smoke | not_ready | candidate |\n"
    "\n"
    "\n"
    "## Contributing\n"
    "\n"
    "See CONTRIBUTING.md.\n"
)
assert "Capability Index" in EMPTY_REGISTRY_DOCUMENT, (
    "the empty-registry input must carry the legacy index heading that makes "
    "migration reach the renderer"
)
assert "### Capability Index" not in EMPTY_REGISTRY_DOCUMENT, (
    "the empty-registry input must carry the *legacy* level-2 index heading"
)
assert "Demo Capability" not in EMPTY_REGISTRY_DOCUMENT, (
    "the placeholder row must be the product's invention, not the input's"
)


def assert_an_empty_registry_gains_no_feature_roots(
    envelope: dict[str, Any], migrated: str
) -> None:
    """A registry with no contract in it renders no feature roots.

    Asserted two ways because the interesting failure adds content rather than
    removing it. The document must equal `EMPTY_REGISTRY_MIGRATED` byte for
    byte, and neither canonical feature root may appear anywhere in it. The byte
    equality is the strictly stronger claim; the two named roots are asserted
    beside it so a failure says which root was invented instead of printing a
    whole-document diff.

    The envelope is asserted to report a real migration, which is what keeps the
    leg honest: a `unchanged` run would mean the renderer never executed and the
    byte equality proved nothing about the guard.
    """
    assert envelope.get("status") == "migrated", (
        f"the empty-registry input must actually reach the renderer, "
        f"got status {envelope.get('status')!r}"
    )
    assert envelope.get("changed") is True, envelope.get("changed")
    for root in ("### Core Features", "### Non-Core Features"):
        assert root not in migrated, (
            f"an empty registry must not gain the {root!r} root, "
            f"document was:\n{migrated}"
        )
    assert migrated == EMPTY_REGISTRY_MIGRATED, (
        f"migration must render a contract-less registry exactly, got:\n"
        f"{migrated!r}\nexpected:\n{EMPTY_REGISTRY_MIGRATED!r}"
    )


#: The capabilities whose whole rendered field block is asserted as one string on
#: the multi-item document, and why those two.
#:
#: `MULTI_ITEM_TITLE` declares two of every list field and no dependency;
#: `CANONICAL_BLOCK_DEPENDENT_TITLE` declares one of each *and* a dependency. Two
#: capabilities rather than one because `Dependencies:` is the last field and is
#: emitted conditionally, so a single subject either never renders it -- leaving
#: its position unbound -- or always does, leaving the shape of a section without
#: it unbound.
CANONICAL_BLOCK_DEPENDENT_TITLE = "Lexical Search"
#: Subject -> the `Feature Class` its section renders, on the multi-item
#: document. Nothing there is classified, which is itself an assertion: the line
#: is conditional and a renderer emitting one would be attributing a class
#: nobody wrote.
CANONICAL_BLOCK_SUBJECTS = {
    MULTI_ITEM_TITLE: None,
    CANONICAL_BLOCK_DEPENDENT_TITLE: None,
}
assert _member_dependencies(CANONICAL_BLOCK_DEPENDENT_TITLE), (
    f"{CANONICAL_BLOCK_DEPENDENT_TITLE!r} has to declare a dependency, or the "
    "position of the field that renders last is not bound by this assertion"
)
assert not _member_dependencies(MULTI_ITEM_TITLE), (
    f"{MULTI_ITEM_TITLE!r} has to declare none, or a section that omits the "
    "conditional last field is never compared"
)

#: The same comparison on the one document that declares classes, and why it is
#: needed at all.
#:
#: The multi-item document classifies nothing, so `Feature Class:` was bound
#: only in the direction of absence: moving the whole conditional block
#: (`capability.rs:9030-9032`) to any other position in the renderer left every
#: subject byte-identical, because none of them emits the line. Every other
#: reader of that field finds it with a position-blind `startswith`. The fix for
#: the *last* conditional field therefore missed the conditional field that
#: renders in the middle.
#:
#: Three subjects: one declaring each of the two class values, so a renderer
#: printing a constant class is a different document, and one declaring none, so
#: the absent direction stays bound on this document too.
MIXED_CANONICAL_BLOCK_SUBJECTS = {
    "Search Core": "core",
    "Contract Gate Wiring": "non_core",
    "Standard Operational Endpoints": None,
}
assert {
    value for value in MIXED_CANONICAL_BLOCK_SUBJECTS.values() if value is not None
} == {"core", "non_core"}, (
    "both class values have to be compared in place, or a renderer emitting a "
    "constant class satisfies the block"
)
assert None in MIXED_CANONICAL_BLOCK_SUBJECTS.values(), (
    "one subject has to declare no class, or a renderer emitting the field "
    "unconditionally is never a different document"
)
for _title, _declared in MIXED_CANONICAL_BLOCK_SUBJECTS.items():
    assert (
        MIXED_SECTION_CLASSES[[member[0] for member in _ALL_MEMBERS].index(_title)]
        == _declared
    ), (
        f"{_title!r} is expected to render {_declared!r}, but the mixed-class "
        f"document does not declare that for it"
    )
assert any(_member_dependencies(title) for title in MIXED_CANONICAL_BLOCK_SUBJECTS), (
    "one subject here has to declare a dependency too, or this document only "
    "ever compares blocks whose last field is absent"
)


def _expected_canonical_field_block(
    title: str,
    items: dict[str, tuple[str, ...]],
    feature_class: str | None,
) -> str:
    """`render_markdown_capability_section_at_level`'s whole field block, restated.

    Everything the renderer emits between the heading and the work-root table
    (`capability.rs:9020-9062`), in the order it emits it, ending with the blank
    line it pushes before the table.

    `Feature Class:` is emitted only when the capability carries one
    (`capability.rs:9030-9032`), so the caller says which, and both directions
    are compared in place rather than by a position-blind line search.
    """
    cap_id = next(member[1] for member in _ALL_MEMBERS if member[0] == title)
    block = (
        f"\nID: {cap_id}\n"
        f"Root WI: {SECTION_RELOCATION_WI[title]}\n"
        f"Status: verified\n"
        f"Type: {_member_type(title)}\n"
    )
    if feature_class is not None:
        block += f"Feature Class: {feature_class}\n"
    block += (
        f"Required Verification: {_member_required_verification(title)}\n"
        f"Promise:\n{MEMBER_PROMISE[title]}\n"
    )
    for field in LIST_FIELDS:
        block += f"{field}:\n" + "".join(f"- {item}\n" for item in items[field])
    dependencies = _member_rendered_dependencies(title)
    if dependencies:
        block += "Dependencies:\n" + "".join(f"- {item}\n" for item in dependencies)
    return block + "\n"


def assert_relocation_renders_the_canonical_field_block(
    migrated: str,
    subjects: dict[str, str | None],
) -> None:
    """The canonical section's fields render in the order the product emits them.

    Every other assertion on this renderer reads one field at a time -- a
    containing line for the scalars, an item list for the three list fields --
    and none of them can see the *order* the fields are emitted in. Reversing the
    four conditional field blocks so `Dependencies`, `EC Dimensions`, `Surfaces`
    and `Gate Inventory` render in that order left every one of those assertions
    green, because each still found its own field somewhere in the section.

    The legacy renderer's leg has compared its whole block for exactly this
    reason since the revision that replaced its prefix check
    (`_expected_legacy_section_body`); the canonical renderer, which is the one
    almost every document here exercises, was never given the same treatment.

    Compared as the whole segment before the work-root table rather than as a
    prefix of the section: the table is bounded content of its own, asserted by
    the work-root and status legs, and cutting at it makes this an equality over
    everything the field block contains -- including a field the renderer must
    *not* emit and a blank line it must.

    Run on two documents rather than one. The first revision compared only
    subjects that declare no `Feature Class`, which bound that conditional
    field's absence and left its *position* free: the block can be emitted
    anywhere in the field order and no subject notices. The second document
    supplies subjects that declare one, in both spellings.
    """
    for title, feature_class in subjects.items():
        body = _capability_section_body(migrated, title)
        head, table, _ = body.partition("| Work Root |")
        assert table, (
            f"section {title!r} rendered no work-root table, so there is no "
            f"field block to bound; section was:\n{body}"
        )
        overrides = MULTI_ITEM_OVERRIDES.get(title, {})
        items = {
            field: overrides.get(
                _OVERRIDE_KEYS[field], (_DECLARED_LIST_ITEMS[field][title],)
            )
            for field in LIST_FIELDS
        }
        expected = _expected_canonical_field_block(title, items, feature_class)
        assert head == expected, (
            f"relocated section {title!r} is not the field block the renderer "
            f"emits; expected:\n{expected!r}\ngot:\n{head!r}"
        )


def assert_relocation_carries_every_work_root_cell(migrated: str) -> None:
    """Each work-root row survives relocation cell by cell, not row by row.

    The row is re-rendered from seven separate `markdown_cell` reads. With one
    constant row shape in every other fixture document, five of those reads were
    unbound at once: `Kind`, `Impl`, `Verification`, `Maturity` and
    `Gate / Evidence` could each be replaced by the literal the fixture happened
    to use everywhere.
    """
    for member in _ALL_MEMBERS:
        body = _capability_section_body(migrated, member[0])
        for work_root in member[4]:
            kind, implementation, verification, maturity, gate = VARIED_WORK_ROOT_CELLS[
                work_root
            ]
            row = (
                f"| {work_root} | {kind} | - | "
                f"{implementation} | {verification} | {maturity} | {gate} |"
            )
            assert row in body, (
                f"relocated section {member[0]!r} lost work-root row {row!r}; "
                f"section was:\n{body}"
            )


#: Relocation input where the capabilities do not all share one status.
#:
#: Every other document in this case renders through
#: `render_markdown_capability_section_at_level` with `Status: verified`, which
#: makes three separate rules vacuous at once. The section's own `Status:` field
#: could be hardcoded to `verified` and nothing would change. So could the
#: Capability Index's `Impl` and `Verification` columns, because both are derived
#: from the status (`capability.rs:9273-9306`) and a single status derives a
#: single pair. And the prelude prose branch (`capability.rs:9016-9019`) was
#: never entered at all, because no fixture document carried prose between a
#: capability heading and its first field.
#:
#: `blocked` is the one status that changes `Impl`; `candidate` and `auditing`
#: change `Verification` without changing `Impl`, so the two columns are
#: falsified independently rather than moving together.
#:
#: The tuple is a permutation of the whole status enum, one member per status,
#: rather than a sample of it. Sampling was the defect: with `confirmed` and
#: `retired` undeclared, their two arms of `capability_verification_summary`
#: (`capability.rs:9296-9306`) were unreached, and rewriting either of them to
#: `"verified"` rendered every document here byte for byte -- a retired
#: capability reading as verified in the index is the exact misreport the column
#: exists to prevent. A closed enum walked exhaustively is the only shape that
#: does not have to be revisited when a later round asks which arms were missed.
VARIED_STATUSES = (
    "blocked",
    "verified",
    "candidate",
    "confirmed",
    "auditing",
    "retired",
)
assert len(VARIED_STATUSES) == len(_ALL_MEMBERS)
assert set(VARIED_STATUSES) == CAPABILITY_STATUSES, (
    "the varied-status document must walk the whole status enum; "
    f"missing {sorted(CAPABILITY_STATUSES - set(VARIED_STATUSES))}, "
    f"unknown {sorted(set(VARIED_STATUSES) - CAPABILITY_STATUSES)}"
)
assert len(set(VARIED_STATUSES)) == len(VARIED_STATUSES), (
    "one capability per status, or a repeated status cannot be told apart from "
    "a renderer that answers for whichever capability it saw first"
)
#: The one status the report's totals treat differently from every other, which
#: is a property of this document alone -- every other relocation shape here
#: leaves the status at its member default and has none.
VARIED_STATUS_RETIRED_TITLES = frozenset(
    member[0]
    for member, status in zip(_ALL_MEMBERS, VARIED_STATUSES)
    if status == "retired"
)
assert len(VARIED_STATUS_RETIRED_TITLES) == 1, sorted(VARIED_STATUS_RETIRED_TITLES)
#: Exactly one capability carries prose. One rather than all, so "the prelude is
#: carried" cannot be confused with "some constant prose is emitted everywhere".
VARIED_PRELUDE_TITLE = _ALL_MEMBERS[0][0]
VARIED_PRELUDE = (
    "Ranked retrieval is the promise this project is bought for; the analyzer "
    "pipeline below is subordinate to it."
)
VARIED_STATUS_PRELUDES: tuple[str | None, ...] = tuple(
    VARIED_PRELUDE if member[0] == VARIED_PRELUDE_TITLE else None
    for member in _ALL_MEMBERS
)
#: The other side of the same rule, and the side no document declared.
#:
#: `markdown_capability_prose_around_machine_tables` (`capability.rs:10966-10988`)
#: returns a *pair*: the prose above the first machine table and the prose below
#: the last one. Only the first was ever declared here, so the whole block that
#: re-emits the second (`capability.rs:9124-9129`) could be deleted and every
#: assertion still passed -- migration silently dropping whatever an author wrote
#: under a capability's work-root table. The product parses it and its own
#: colocated `--lib` invariant covers a fenced-code case of it, which is exactly
#: the boundary this project does not get to lean on: a rule observable in the
#: migrated document is this case's to bind.
#:
#: On a different capability from the prelude, so "prose is carried" cannot be
#: satisfied by one section's blob appearing somewhere, and neither piece is the
#: document's last section -- the postlude of a last section would be bounded by
#: the end of the document rather than by the next heading.
VARIED_POSTLUDE_TITLE = _ALL_MEMBERS[4][0]
VARIED_POSTLUDE = (
    "Rollout order is transport first, then identity; both stay behind the "
    "hardening flag until the negative suite is green."
)
assert VARIED_POSTLUDE_TITLE != VARIED_PRELUDE_TITLE, (
    "the two prose sides have to sit on different capabilities, or carrying "
    "one section's prose answers for both"
)
assert VARIED_POSTLUDE_TITLE != _ALL_MEMBERS[-1][0], (
    "the postlude must not sit on the document's last section, whose end is "
    "the end of the document rather than the next heading"
)
#: The `aw ec` efficiency backfill slot, carried through relocation.
#:
#: Not prose. It occupies the same position as the postlude above -- below the
#: work-root table -- but takes a different route through migration:
#: `find_efficiency_backfill_section_span` (`capability.rs:11557+`) *removes* it
#: from the prose it would otherwise be carried as, `parse_efficiency_slot_from_
#: contract` reads its two fields into the capability, and
#: `render_efficiency_backfill_section` (`capability.rs:5324-5332`) writes it
#: back out. Deleting that last step used to render the identical document,
#: because no fixture declared a slot: the block simply had nowhere to go
#: missing from.
#:
#: On a third capability, distinct from both prose sides, so an implementation
#: that carried the block through as ordinary postlude prose -- or that emitted
#: the prose postlude through the efficiency renderer -- is a different document
#: rather than a coincidence of the two landing on the same section.
EFFICIENCY_SLOT_TITLE = _ALL_MEMBERS[3][0]
EFFICIENCY_SLOT_OPERATING_POINT = "p99 < 45ms at 1.2k rps, 3 replicas"
EFFICIENCY_SLOT_CUBE = "cube/kubernetes-native-deployment.json"


#: `####`, one level below the `###` capability sections of this input. At the
#: capability level it would close the section rather than sit inside it.
def _efficiency_slot_block(operating_point: str, cube: str) -> str:
    return (
        "#### Efficiency - GENERATED (backfilled by `aw ec`; do not hand-edit)\n"
        "\n"
        f"Operating point: {operating_point}\n"
        f"Cube: {cube}\n"
    )


EFFICIENCY_SLOT_BLOCK = _efficiency_slot_block(
    EFFICIENCY_SLOT_OPERATING_POINT, EFFICIENCY_SLOT_CUBE
)
#: The other arm of the same merge, and the arm the fixture's own guard used to
#: certify unreachable.
#:
#: `merge_efficiency_backfill_slot` (`capability.rs:11632-11650`) branches on
#: whether the capability already declares an `efficiency` dimension. When it
#: does not, the slot is *pushed* as a new dimension; when it does, the slot is
#: *attached* to the existing one (`dimension.efficiency_backfill = Some(slot);
#: return;`) and the dimension list is left exactly as authored. The push arm is
#: `EFFICIENCY_SLOT_TITLE` above. Nothing reached the attach arm -- and the
#: assert that used to sit under the push carrier said so out loud, requiring
#: the slot to land on a capability declaring some *other* dimension. A fixture
#: invariant that positively guarantees an arm is never entered is not coverage
#: of that arm.
#:
#: What this carrier binds is narrower than "the attach arm", and the difference
#: was established by experiment rather than assumed. Deleting the attach arm on
#: its own renders this document byte for byte: the push arm then appends a
#: second `efficiency` dimension, and `dedupe_ec_dimensions`
#: (`capability.rs:11651-11673`) folds it straight back into the declared one,
#: filling the empty `efficiency_backfill` and discarding the generated runner
#: and summary because the declared ones are non-empty. Deleting that fill on
#: its own renders it byte for byte too, because the attach arm already set the
#: field. The two mechanisms are redundant, so *no* external contract can
#: separate them; what a document can bind is that the slot survives at all on a
#: capability that already declares an `efficiency` dimension, which fails as
#: soon as both are removed. That is the claim asserted here, and the redundancy
#: itself is reported as a product finding rather than papered over.
#:
#: A fourth capability, so the push arm's effect is told apart by which section
#: changed rather than by which of two shapes one section took. Its two fields
#: differ from the push carrier's, so the two blocks cannot be confused for one
#: another and each is still asserted to appear exactly once.
EFFICIENCY_ATTACH_TITLE = _ALL_MEMBERS[1][0]
EFFICIENCY_ATTACH_OPERATING_POINT = "p95 < 12ms at 300 rps, single replica"
EFFICIENCY_ATTACH_CUBE = "cube/lexical-search.json"
EFFICIENCY_ATTACH_BLOCK = _efficiency_slot_block(
    EFFICIENCY_ATTACH_OPERATING_POINT, EFFICIENCY_ATTACH_CUBE
)
assert EFFICIENCY_ATTACH_BLOCK != EFFICIENCY_SLOT_BLOCK, (
    "the two slot carriers must declare different operating points, or "
    "'each block was re-emitted once' cannot be counted per block"
)
for _label, _other in (
    ("prelude", VARIED_PRELUDE_TITLE),
    ("postlude", VARIED_POSTLUDE_TITLE),
    ("efficiency attach slot", EFFICIENCY_ATTACH_TITLE),
):
    assert EFFICIENCY_SLOT_TITLE != _other, (
        f"the efficiency slot must not share a capability with the {_label}, "
        f"or carrying one answers for the other"
    )
for _label, _other in (
    ("prelude", VARIED_PRELUDE_TITLE),
    ("postlude", VARIED_POSTLUDE_TITLE),
):
    assert EFFICIENCY_ATTACH_TITLE != _other, (
        f"the efficiency attach slot must not share a capability with the "
        f"{_label}, or carrying one answers for the other"
    )
#: The merge's own effect on the contract, which is not prose at all:
#: `merge_efficiency_backfill_slot` (`capability.rs:11632-11650`) appends an
#: `efficiency` EC dimension when the capability declares none. This carrier
#: declares `behavior`, so the appended item is observable.
EFFICIENCY_SLOT_MERGED_DIMENSION = "efficiency: aw-generated efficiency backfill slot"
assert _member_ec_dimension(EFFICIENCY_SLOT_TITLE) != "efficiency", (
    "the push carrier has to declare some other dimension, or the merge's "
    "append arm is never entered"
)
assert _member_ec_dimension(EFFICIENCY_ATTACH_TITLE) == "efficiency", (
    "the attach carrier has to be the capability that already declares an "
    "`efficiency` dimension, which is the whole guard on the attach arm"
)

_VARIED_STATUS_POSTLUDE_BY_TITLE = {
    VARIED_POSTLUDE_TITLE: VARIED_POSTLUDE,
    EFFICIENCY_SLOT_TITLE: EFFICIENCY_SLOT_BLOCK.rstrip("\n"),
    EFFICIENCY_ATTACH_TITLE: EFFICIENCY_ATTACH_BLOCK.rstrip("\n"),
}
assert len(_VARIED_STATUS_POSTLUDE_BY_TITLE) == 3, (
    "three distinct capabilities carry something below their work-root table, "
    "or two of the three routes share a section and answer for each other"
)
VARIED_STATUS_POSTLUDES: tuple[str | None, ...] = tuple(
    _VARIED_STATUS_POSTLUDE_BY_TITLE.get(member[0]) for member in _ALL_MEMBERS
)
#: Work-root cells that make every capability's gap read `InProgress`.
#:
#: `capability_gap_status_from_table` (`capability.rs:11862-11881`) resolves
#: `implemented` + `planned` to neither `Closed` nor `Open` nor `Deferred` nor
#: `Blocked`, so it falls through to `InProgress` -- and `InProgress` is the
#: *only* gap state that makes `capability_impl_summary` return `partial`.
#: Every relocation document here wrote `implemented | verified` for every work
#: root, which closes every gap, so the `partial` arm was unreachable from this
#: whole fixture: rewriting it to `"planned"` changed nothing. `partial` is what
#: distinguishes a capability whose work has started from one that has not, so
#: collapsing it into `planned` makes an in-flight capability read as untouched.
#:
#: Applied to the varied-status document rather than to a new one, because the
#: `partial` arm needs both halves at once: a status that is neither `verified`
#: nor `blocked` (or the status arms answer first), and a gap that is open. This
#: is the one document that already varies the status half.
#: The one capability whose gaps are all closed while its status is not
#: `verified`, which is the only way to reach the second disjunct of
#: `capability_impl_summary` (`capability.rs:9277-9280`).
#:
#: That function answers `implemented` for a `verified` capability from the
#: status alone, before the gaps are read. Every capability in this fixture was
#: either `verified` -- answered by the first disjunct -- or in progress, so the
#: second one, "not verified, but every gap is closed or deferred", was
#: unreachable: deleting it rendered every document here byte for byte, and a
#: project that had finished the work but not yet re-verified read as `partial`.
#: It sits on the `retired` member: work that is finished and no longer verified
#: is exactly the state the disjunct describes, and that member is also the one
#: free of the prose and slot shapes above.
_CLOSED_GAP_INDEX = 5
CLOSED_GAP_TITLE = _ALL_MEMBERS[_CLOSED_GAP_INDEX][0]
assert VARIED_STATUSES[_CLOSED_GAP_INDEX] != "verified", (
    "the all-closed capability must not be `verified`, or the status arm "
    "answers before the gaps are consulted and the disjunct stays unreached"
)
assert CLOSED_GAP_TITLE not in _VARIED_STATUS_POSTLUDE_BY_TITLE, (
    "the all-closed capability must not also carry prose or a slot block, or "
    "one shape's assertion answers for the other"
)
_CLOSED_WORK_ROOTS = frozenset(_ALL_MEMBERS[_CLOSED_GAP_INDEX][4])
_IN_PROGRESS_WORK_ROOT_CELLS = {
    work_root: (
        ("change", "implemented", "verified", "smoke", "`true`")
        if work_root in _CLOSED_WORK_ROOTS
        else ("change", "implemented", "planned", "smoke", "`true`")
    )
    for member in _ALL_MEMBERS
    for work_root in member[4]
}
assert len(set(_IN_PROGRESS_WORK_ROOT_CELLS.values())) == 2, (
    "this document has to carry both gap shapes at once, or `partial` and the "
    "all-closed disjunct cannot both be reached from it"
)

#: The capability that declares none of the four optional fields.
#:
#: `render_markdown_capability_section_at_level` guards each of `Type:`,
#: `Surfaces:`, `EC Dimensions:` and `Required Verification:` on its own
#: emptiness check; every capability of every document here declared all four,
#: so forcing any one of those guards to `true` rendered the identical document
#: and all four conditionals were free in the direction of absence. An earlier
#: round bound `Surfaces:` alone and left its three siblings in the same block
#: untouched, which is why they are withheld together now rather than one per
#: round.
#:
#: They land on one capability rather than four because only one capability can
#: legally withhold anything. `validate_capability_contract`
#: (`capability.rs:10133+`) requires a full contract for the five
#: contract-bearing statuses, so `candidate` is the single status under which an
#: author may declare nothing -- and this document declares each status exactly
#: once. The assertions stay per field, so a product that forgot one guard is
#: still caught by that guard's own check.
#:
#: `Required Verification:` is the field whose absence is *not* silent: the
#: renderer substitutes the `smoke` fallback of `capability_maturity_summary`
#: (`capability.rs:9826-9840`), so this capability is also the only carrier for
#: that fallback. Every other member declares a different maturity, so `smoke`
#: here cannot be confused with a constant.
WITHHELD_FIELDS_TITLE = _ALL_MEMBERS[2][0]
WITHHELD_FIELDS = WITHHOLDABLE_FIELDS
#: What the renderer substitutes for the withheld `Required Verification:`.
WITHHELD_REQUIRED_VERIFICATION_FALLBACK = "smoke"
assert VARIED_STATUSES[2] == "candidate", (
    "the field-withholding capability must be the one whose status exempts it "
    "from the contract requirement, or the document is rejected rather than "
    "rendered"
)
assert _member_required_verification(WITHHELD_FIELDS_TITLE) != (
    WITHHELD_REQUIRED_VERIFICATION_FALLBACK
), (
    "the withholding capability's declared maturity must differ from the "
    "fallback, or 'the fallback was substituted' cannot be told apart from "
    "'the declared value was carried'"
)
for _label, _other in (
    ("prelude", VARIED_PRELUDE_TITLE),
    ("postlude", VARIED_POSTLUDE_TITLE),
    ("efficiency push slot", EFFICIENCY_SLOT_TITLE),
    ("efficiency attach slot", EFFICIENCY_ATTACH_TITLE),
    ("all-closed gaps", CLOSED_GAP_TITLE),
):
    assert WITHHELD_FIELDS_TITLE != _other, (
        f"the field-withholding capability must not also carry the {_label}; "
        f"one capability per shape keeps each assertion answering for one rule"
    )

VARIED_STATUS_SECTION_README = _section_readme(
    _ALL_MEMBERS,
    (None,) * len(_ALL_MEMBERS),
    "Lumen README-resident capability contract, statuses not uniform.",
    statuses=VARIED_STATUSES,
    preludes=VARIED_STATUS_PRELUDES,
    postludes=VARIED_STATUS_POSTLUDES,
    work_root_cells=_IN_PROGRESS_WORK_ROOT_CELLS,
    withheld_by_title={WITHHELD_FIELDS_TITLE: WITHHELD_FIELDS},
)
_WITHHELD_DECLARED = VARIED_STATUS_SECTION_README.split(
    f"### {WITHHELD_FIELDS_TITLE}\n", 1
)[1].split("\n### ", 1)[0]
for _field_label in ("Type:", "Surfaces:", "EC Dimensions:", "Required Verification:"):
    assert _field_label not in _WITHHELD_DECLARED, (
        f"the varied-status document must actually withhold {_field_label!r}, "
        f"which it asserts absent from the rendered section"
    )
    assert VARIED_STATUS_SECTION_README.count(_field_label) == len(_ALL_MEMBERS) - 1, (
        f"every other capability of this document must still declare "
        f"{_field_label!r}, or the absence is a property of the document rather "
        f"than of one capability"
    )
assert VARIED_STATUS_SECTION_README.count(VARIED_POSTLUDE) == 1, (
    "the varied-status document must actually declare the postlude, or its "
    "assertion pins nothing"
)
for _slot_label, _slot_block in (
    ("push", EFFICIENCY_SLOT_BLOCK),
    ("attach", EFFICIENCY_ATTACH_BLOCK),
):
    assert VARIED_STATUS_SECTION_README.count(_slot_block.rstrip("\n")) == 1, (
        f"the varied-status document must actually declare the {_slot_label} "
        f"arm's efficiency backfill slot, or its assertion pins nothing"
    )
assert EFFICIENCY_SLOT_MERGED_DIMENSION not in VARIED_STATUS_SECTION_README, (
    "the merged dimension is the product's addition; declaring it in the input "
    "would make 'the merge appended it' indistinguishable from 'the author "
    "wrote it'"
)

#: What the efficiency merge does to the *contract*, as opposed to the block.
#:
#: The one capability of this document whose rendered EC dimension list is not
#: the one it declared: `merge_efficiency_backfill_slot` appends a dimension for
#: the slot. Declared as an override rather than folded into `MEMBER_EC_
#: DIMENSION_ITEM`, because it is the product's addition and not the author's --
#: writing it into the member table would make the two indistinguishable.
#: Its counterpart on the attach carrier is the *absence* of that addition: the
#: attach arm stores the slot on the dimension the author already declared and
#: leaves the list at arity one. Stated as an explicit override equal to the
#: declared list rather than left to the default, because "unchanged" is the
#: whole observable of that arm -- a product that fell through to the push arm
#: would render a second `efficiency` item here, and the assertion below reads
#: the list rather than merely finding the declared item somewhere in it.
VARIED_STATUS_OVERRIDES = {
    EFFICIENCY_SLOT_TITLE: {
        "ec_dimensions": (
            MEMBER_EC_DIMENSION_ITEM[EFFICIENCY_SLOT_TITLE],
            EFFICIENCY_SLOT_MERGED_DIMENSION,
        ),
    },
    EFFICIENCY_ATTACH_TITLE: {
        "ec_dimensions": (MEMBER_EC_DIMENSION_ITEM[EFFICIENCY_ATTACH_TITLE],),
    },
    #: The empty overrides: this capability declared none of the four optional
    #: fields, so the renderer must emit none of them -- asserted as absence
    #: rather than by skipping the capability, which would stop binding its
    #: remaining fields. `required_verification` is the one exception: its
    #: absence is filled by the renderer rather than propagated, so it is
    #: asserted as the substituted fallback instead of as an empty tuple.
    WITHHELD_FIELDS_TITLE: {
        "surfaces": (),
        "type": None,
        "ec_dimensions": (),
        "required_verification": WITHHELD_REQUIRED_VERIFICATION_FALLBACK,
    },
}

#: What each status derives for the two index columns that read it, restated
#: from `capability_impl_summary` and `capability_verification_summary`. Restated
#: rather than imported, because the point is to pin the product's mapping; the
#: fixture asserts below that the restatement is not degenerate.
#:
#: `Verification` is a `match` over the whole status enum, one arm each, so the
#: table below has one row per status and the document declares each exactly
#: once. Two of those arms -- `confirmed` and `retired` -- were undeclared until
#: this round and freely rewritable to `"verified"`.
#:
#: `Impl` reads the status *and* the gaps: `blocked` and `verified` are answered
#: by the status arms before the gaps are consulted, while the other four fall
#: through to them. The gap half is therefore a second key, not a constant, and
#: the same status derives different `Impl` answers on different gap shapes --
#: which is what `CLOSED_GAP_TITLE` supplies for `auditing`.
_STATUS_VERIFICATION_COLUMN = {
    "verified": "verified",
    "blocked": "blocked",
    "candidate": "planned",
    "confirmed": "planned",
    "auditing": "planned",
    "retired": "blocked",
}
assert set(_STATUS_VERIFICATION_COLUMN) == CAPABILITY_STATUSES, (
    "the restatement of `capability_verification_summary` must cover every arm "
    "of the enum it restates"
)
#: `capability_impl_summary` in the same restated form. The status arms answer
#: first; everything else is decided by the gaps, which is why this is keyed by
#: the pair rather than by the status alone.
_STATUS_IMPL_COLUMN = {
    "verified": "implemented",
    "blocked": "blocked",
}
_GAP_IMPL_COLUMN = {"closed": "implemented", "in_progress": "partial"}
VARIED_STATUS_INDEX_COLUMNS = {
    member[0]: (
        _STATUS_IMPL_COLUMN.get(
            status,
            _GAP_IMPL_COLUMN["closed" if member[0] == CLOSED_GAP_TITLE else "in_progress"],
        ),
        _STATUS_VERIFICATION_COLUMN[status],
    )
    for member, status in zip(_ALL_MEMBERS, VARIED_STATUSES)
}
assert len(set(VARIED_STATUS_INDEX_COLUMNS.values())) >= 3, (
    "the varied-status shape must derive at least three distinct (Impl, "
    "Verification) pairs, or a constant column is indistinguishable from a "
    "derived one"
)
assert len({pair[1] for pair in VARIED_STATUS_INDEX_COLUMNS.values()}) > 1
#: `Impl` has four possible answers and this document must exercise three of
#: them, including `partial`. The fourth, `planned`, needs a capability with no
#: work roots at all, which is a different document shape; it is asserted on the
#: derived-inventory fixture below rather than left implied.
assert {pair[0] for pair in VARIED_STATUS_INDEX_COLUMNS.values()} == {
    "blocked",
    "implemented",
    "partial",
}, "the varied-status shape must exercise three distinct Impl derivations"
#: And `implemented` must be derived twice over, once from each disjunct: a
#: `verified` capability whose gaps are open, and a non-`verified` one whose
#: gaps are all closed. With only the first, deleting the second disjunct
#: renders the identical document.
assert VARIED_STATUS_INDEX_COLUMNS[CLOSED_GAP_TITLE][0] == "implemented", (
    "the all-closed capability must derive `implemented` through the gap "
    "disjunct, or that disjunct is still unreached"
)
assert (
    len([title for title, pair in VARIED_STATUS_INDEX_COLUMNS.items() if pair[0] == "implemented"])
    == 2
), "one `implemented` per disjunct, so neither answers for the other"


def assert_relocation_carries_per_capability_status(migrated: str) -> None:
    """Status, prose, and the two index columns derived from status all survive.

    Three rules in one leg because one document falsifies all three and they
    share the same cause -- a fixture in which every capability was `verified`.

    - The section's `Status:` field is the capability's own, not a constant.
    - The Capability Index's `Impl` and `Verification` columns are derived from
      that status. This is the branch reached when the input carries no index
      table of its own, which is every relocation input here.
    - The `Notes` column falls back to the capability's own promise, so it is
      asserted per capability too; a constant there would otherwise pass.
    - The one capability carrying prose keeps it, above its first field.
    """
    for member, status in zip(_ALL_MEMBERS, VARIED_STATUSES):
        body = _capability_section_body(migrated, member[0])
        assert f"Status: {status}\n" in body, (
            f"relocated section {member[0]!r} lost its status {status!r}; "
            f"section was:\n{body}"
        )

    rows = {row[0]: row for row in _index_rows_parsed(migrated)}
    assert set(rows) == set(VARIED_STATUS_INDEX_COLUMNS), sorted(rows)
    for title, (implementation, verification) in VARIED_STATUS_INDEX_COLUMNS.items():
        row = rows[title]
        assert row[2] == implementation, (
            f"index row {title!r} Impl column: expected {implementation!r} for "
            f"its status, got {row[2]!r}"
        )
        assert row[3] == verification, (
            f"index row {title!r} Verification column: expected "
            f"{verification!r} for its status, got {row[3]!r}"
        )
        # The Maturity column's fallback is `capability_maturity_summary`
        # (`capability.rs:8951`, `capability.rs:9826-9840`), which is the
        # capability's own `Required Verification` list joined. Reachable on
        # every relocation shape -- and unasserted on all of them until now,
        # because the varied-index document was the only place Maturity was read
        # back and its own docstring wrongly claimed this branch could not
        # produce a varying value.
        # One capability of this document declares no `Required Verification:`
        # at all, and for it the column is the function's own fallback rather
        # than a carried value. Both directions are asserted here because the
        # fallback was the unreached half: with every capability declaring a
        # maturity, `unwrap_or_else(|| "smoke")` never ran, and rewriting the
        # literal it substitutes changed nothing. No member declares bare
        # `smoke`, so the substituted value cannot be mistaken for a carried one.
        expected_maturity = (
            WITHHELD_REQUIRED_VERIFICATION_FALLBACK
            if title == WITHHELD_FIELDS_TITLE
            else _member_required_verification(title)
        )
        assert row[4] == expected_maturity, (
            f"index row {title!r} Maturity column: expected {expected_maturity!r}, "
            f"got {row[4]!r}"
        )
        # The Production column's fallback is `capability_production_summary`
        # (`capability.rs:9308-9314`). An earlier round recorded that this
        # column "is reachable only on the carried-through branch". That was
        # wrong, and the correction matters more than the assertion: what is
        # unreachable from this fixture is the function's `ready` *arm* (filed as
        # #3214), because `parse_markdown_capability_block` hardcodes
        # `release_scope: false` (`capability.rs:10773`) and the only thing that
        # raises it (`capability.rs:8400-8410`) raises `index_summary` with it,
        # which routes the render past this fallback entirely -- so no
        # markdown-authored capability reaches the arm. The *function* is called
        # for every capability in every
        # index-less document here -- six capabilities across five documents --
        # and its `not_ready` answer was read back by nothing, so replacing the
        # whole body with the constant `"ready"` shipped a table declaring six
        # unwritten contracts production-ready with every assertion still green.
        assert row[5] == "not_ready", (
            f"index row {title!r} Production column: a capability parsed out of "
            f"Markdown is never in release scope, so the derived column must "
            f"read 'not_ready', got {row[5]!r}"
        )
        assert row[6] == member_promise_note_cell(title), (
            f"index row {title!r} Notes column must fall back to that "
            f"capability's own promise, folded into one cell; expected "
            f"{member_promise_note_cell(title)!r}, got {row[6]!r}"
        )

    prelude_body = _capability_section_body(migrated, VARIED_PRELUDE_TITLE)
    assert prelude_body.startswith(f"\n{VARIED_PRELUDE}\n"), (
        f"relocated section {VARIED_PRELUDE_TITLE!r} lost its prose prelude; "
        f"section was:\n{prelude_body}"
    )
    assert migrated.count(VARIED_PRELUDE) == 1, (
        "the prelude belongs to exactly one capability; emitting it more than "
        "once would mean prose is being copied rather than carried"
    )

    # And the other side of the same pair. The parse returns prelude *and*
    # postlude; only the prelude was ever declared, which left the block that
    # re-emits the postlude deletable.
    postlude_body = _capability_section_body(migrated, VARIED_POSTLUDE_TITLE)
    assert postlude_body.endswith(f"\n\n{VARIED_POSTLUDE}\n"), (
        f"relocated section {VARIED_POSTLUDE_TITLE!r} lost the prose written "
        f"under its work-root table; section was:\n{postlude_body}"
    )
    assert migrated.count(VARIED_POSTLUDE) == 1, (
        "the postlude belongs to exactly one capability too"
    )
    assert VARIED_PRELUDE not in postlude_body, (
        "the two prose sides must stay on their own capabilities, or one of "
        "them is being copied rather than carried"
    )
    assert VARIED_POSTLUDE not in prelude_body, (
        "the two prose sides must stay on their own capabilities, or one of "
        "them is being copied rather than carried"
    )

    # And the third thing that can occupy that same slot, which is not prose at
    # all. `find_efficiency_backfill_section_span` lifts the block out of the
    # prose before either side above is read, so the two prose assertions hold
    # whether or not it is ever re-emitted: deleting the renderer that writes it
    # back rendered this document byte for byte.
    #
    # Read against the whole document rather than the section body, because the
    # block is itself a heading at the capability level -- it *bounds* the
    # section it belongs to rather than sitting inside it, and a reader that
    # looked for it in the body would find it missing on a correct product.
    #
    # Both merge arms are read here. They differ only in what the merge does to
    # the capability's dimension list, so each is asserted against its own
    # block: the push carrier's list gains the generated item, the attach
    # carrier's stays exactly as authored.
    for slot_label, slot_title, slot_block in (
        ("push", EFFICIENCY_SLOT_TITLE, EFFICIENCY_SLOT_BLOCK),
        ("attach", EFFICIENCY_ATTACH_TITLE, EFFICIENCY_ATTACH_BLOCK),
    ):
        assert migrated.count(slot_block) == 1, (
            f"the {slot_label} arm's efficiency backfill slot must be "
            f"re-emitted exactly once; found {migrated.count(slot_block)} "
            f"copies of:\n{slot_block}"
        )
        # Bounded by the *next capability*, not by the next heading.
        # `_capability_section_body` stops at the next heading of the rendering
        # level or shallower, and the block renders at `####` -- so it falls
        # inside the section on an unclassified document and outside it on a
        # classified one. Which of those this document is is not what this
        # assertion is about.
        slot_window = _capability_window(migrated, slot_title)
        assert slot_window.rstrip("\n").endswith(slot_block.rstrip("\n")), (
            f"the {slot_label} arm's efficiency backfill slot must be "
            f"re-emitted below {slot_title!r}'s work-root table, in the "
            f"position it was authored; that capability ended with:\n"
            f"{slot_window[-500:]}"
        )
        for other in (VARIED_PRELUDE, VARIED_POSTLUDE):
            assert other not in slot_window, (
                "the efficiency slot's capability carries neither prose side, "
                "so a renderer that emitted prose through the efficiency block "
                "-- or the block through the prose renderer -- is a different "
                "document"
            )

    # The merge's effect on the contract, asserted from both sides so that
    # neither arm can answer for the other. The generated dimension belongs to
    # the push carrier and to nothing else: a product that fell through to the
    # push arm for a capability already declaring `efficiency` would emit a
    # second copy here, and one that took the attach arm for the push carrier
    # would emit none.
    assert migrated.count(EFFICIENCY_SLOT_MERGED_DIMENSION) == 1, (
        f"the efficiency merge appends its generated dimension to the one "
        f"capability that declared no `efficiency` dimension of its own; found "
        f"{migrated.count(EFFICIENCY_SLOT_MERGED_DIMENSION)} copies"
    )
    attach_window = _capability_window(migrated, EFFICIENCY_ATTACH_TITLE)
    assert EFFICIENCY_SLOT_MERGED_DIMENSION not in attach_window, (
        f"{EFFICIENCY_ATTACH_TITLE!r} already declares an `efficiency` "
        f"dimension, so the merge must attach its slot to that dimension "
        f"rather than append a second one; that capability rendered as:\n"
        f"{attach_window}"
    )


#: Per-capability Capability Index cells, pairwise distinct in every column.
#:
#: `_index_rows` emits the same five trailing cells for every member, so a
#: renderer that dropped `capability.index_summary` and printed one constant row
#: per capability was indistinguishable from one that carried each row through.
#: Five of the seven columns were unbound at once.
#:
#: These values are deliberately not all drawn from the token vocabularies the
#: rest of the document uses. `parse_capability_index_summaries`
#: (`capability.rs:9316-9385`) stores each cell as free text and the renderer
#: prints it back, so the contract is round-trip, not normalisation; asserting it
#: with values no enum could supply is what separates "carried" from "recomputed
#: and coincidentally equal".
VARIED_INDEX_CELLS = {
    "Search Core": ("implemented", "verified", "conformance", "ready", "domain core"),
    "Lexical Search": ("partial", "planned", "load", "pilot", "analyzer rollout"),
    "Standard Operational Endpoints": (
        "planned",
        "blocked",
        "smoke",
        "not_ready",
        "awaiting probe contract",
    ),
    "Kubernetes-Native Deployment": (
        "blocked",
        "deferred",
        "chaos",
        "staged",
        "manifest packaging gap",
    ),
    "Security Hardening": (
        "prototype",
        "contract-only",
        "soak",
        "ready-with-caveats",
        "identity hardening in review",
    ),
    "Contract Gate Wiring": (
        "staged",
        "smoke-only",
        "functional",
        "blocked",
        "gate inventory sync pending",
    ),
}
assert set(VARIED_INDEX_CELLS) == {member[0] for member in _ALL_MEMBERS}
for _column in range(5):
    assert len({cells[_column] for cells in VARIED_INDEX_CELLS.values()}) == len(
        _ALL_MEMBERS
    ), f"Capability Index column {_column} must be pairwise distinct across members"


def _varied_index_document() -> str:
    """`UNCLASSIFIED_DOCUMENT`'s shape, with the index rows made distinguishable.

    Format migration, not relocation: this input already carries a `Capability
    Index`, so every capability gets an `index_summary` and the renderer takes
    the carried-through branch rather than the derived-from-status fallback.

    An earlier revision claimed `Production` was reachable *only* on this
    branch. That is false and was refuted by a surviving mutant: the fallback
    calls `capability_production_summary` for every capability in every
    index-less document, and its answer -- `not_ready`, because
    `parse_markdown_capability_block` hardcodes `release_scope: false`
    (`capability.rs:10773`) -- is rendered into all six rows of five relocation
    documents. What is genuinely unreachable is the *`ready` arm* of that
    function, which is a narrower statement about one arm rather than about the
    column, and is disclosed as an unreached branch (filed as #3214) rather than
    dressed up as coverage. `Maturity` was never in question either: its fallback is the
    capability's own `Required Verification`, which this fixture varies per
    member. `assert_relocation_carries_per_capability_status` binds both derived
    columns; this document binds only the carried-through form.
    """
    rows = "\n".join(
        "| {title} | - | {impl} | {verification} | {maturity} | {production} | {notes} |".format(
            title=member[0],
            impl=VARIED_INDEX_CELLS[member[0]][0],
            verification=VARIED_INDEX_CELLS[member[0]][1],
            maturity=VARIED_INDEX_CELLS[member[0]][2],
            production=VARIED_INDEX_CELLS[member[0]][3],
            notes=VARIED_INDEX_CELLS[member[0]][4],
        )
        for member in _ALL_MEMBERS
    )
    body = "".join(_section(member, None, "###") for member in _ALL_MEMBERS)
    return f"""# Lumen

## Brief

Lumen reference fixture, pre-migration: the index rows differ per capability.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
{rows}

{body}"""


VARIED_INDEX_DOCUMENT = _varied_index_document()


def assert_migration_carries_every_index_column(migrated: str) -> None:
    """Each capability's own five index cells survive the rewrite.

    Not "an index table is emitted" and not "the row count is right" -- the
    columns other than `Capability` and `Root WI` were rewritable to constants
    without failing anything.
    """
    rows = {row[0]: row for row in _index_rows_parsed(migrated)}
    assert set(rows) == set(VARIED_INDEX_CELLS), sorted(rows)
    columns = ("Impl", "Verification", "Maturity", "Production", "Notes")
    for title, expected in VARIED_INDEX_CELLS.items():
        actual = tuple(rows[title][2:7])
        assert actual == expected, (
            f"index row {title!r} lost its own cells: expected "
            f"{dict(zip(columns, expected))}, got {dict(zip(columns, actual))}"
        )


def _level_2_index_document() -> str:
    """`VARIED_INDEX_DOCUMENT` with its index heading written at level 2.

    `parse_capability_index_summaries` (`capability.rs:9316-9385`) accepts a
    `Capability Index` heading at level 2 *or* level 3, and is run over the whole
    document body (`capability.rs:8399`), so both are reachable. Every
    index-carrying fixture here wrote `###`, which left the level-2 arm free:
    restricting the guard to level 3 makes an author's `## Capability Index`
    parse as no index at all, so every carried cell is silently replaced by the
    derived fallback and the document comes back saying different things about
    every capability than it said going in.
    """
    document = VARIED_INDEX_DOCUMENT
    heading = "### Capability Index\n"
    assert document.count(heading) == 1, document
    return document.replace(heading, "## Capability Index\n", 1)


LEVEL_2_INDEX_DOCUMENT = _level_2_index_document()


def _no_notes_column_document() -> str:
    """`VARIED_INDEX_DOCUMENT` with the `Notes` column removed from the index.

    The `Notes` cell is the one index column with a fallback of its own: when
    the summary carries no note, the renderer prints the capability's *promise*
    instead (`capability.rs:8972-8977`). That fallback was unreachable from every
    fixture here, and not for the obvious reason -- writing a blank cell does not
    reach it either, because `table_cell` (`capability.rs:12134-12140`) turns an
    empty cell into `-`, which is not empty and renders straight back as `-`.
    The only input that reaches it is an index table with no `Notes` column at
    all, where `notes_idx` is `None` and the field defaults to the empty string.

    That is a real authoring shape -- a hand-written index carrying only the
    readiness columns -- and the promise fallback is what keeps such a row from
    coming out blank. Deleting the emptiness filter renders `-` for all six.
    """
    document = VARIED_INDEX_DOCUMENT
    header = (
        "| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |\n"
        "|---|---:|---|---|---|---|---|\n"
    )
    assert document.count(header) == 1, document
    document = document.replace(
        header,
        "| Capability | Root WI | Impl | Verification | Maturity | Production |\n"
        "|---|---:|---|---|---|\n",
        1,
    )
    for title, cells in VARIED_INDEX_CELLS.items():
        row = f"| {title} | - | " + " | ".join(cells[:4]) + f" | {cells[4]} |"
        assert document.count(row) == 1, (title, row)
        document = document.replace(
            row, f"| {title} | - | " + " | ".join(cells[:4]) + " |", 1
        )
    for _title, cells in VARIED_INDEX_CELLS.items():
        assert cells[4] not in document, (
            f"the note {cells[4]!r} must not survive anywhere in the input, or "
            f"the promise fallback is not what the assertion observes"
        )
    return document


NO_NOTES_COLUMN_DOCUMENT = _no_notes_column_document()


def assert_migration_falls_back_to_the_promise_for_notes(migrated: str) -> None:
    """With no `Notes` column declared, each row's note is its own promise.

    Asserted per capability against pairwise-distinct promises, so a renderer
    that printed one capability's promise into every row is a different
    document, and asserted alongside the four columns the same rows *did*
    declare -- otherwise "the notes are the promises" is also satisfied by an
    implementation that discarded the index summary entirely and derived all six
    columns, which is the opposite failure.
    """
    rows = {row[0]: row for row in _index_rows_parsed(migrated)}
    assert set(rows) == set(VARIED_INDEX_CELLS), sorted(rows)
    for title, cells in VARIED_INDEX_CELLS.items():
        carried = tuple(rows[title][2:6])
        assert carried == cells[:4], (
            f"index row {title!r} lost the columns it did declare: expected "
            f"{cells[:4]}, got {carried}"
        )
        # `_index_rows_parsed` unescapes `\|` back to `|`, so the pipe compares
        # against the promise as written. The newline fold is not undone: a
        # two-line promise is one cell containing `<br>`, which is what
        # `member_promise_note_cell` restates.
        expected_note = member_promise_note_cell(title)
        assert rows[title][6] == expected_note, (
            f"index row {title!r} declared no note, so it must fall back to its "
            f"own promise; expected {expected_note!r}, got {rows[title][6]!r}"
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


def _capability_window(migrated: str, title: str) -> str:
    """One capability's text, bounded by the *next capability* rather than by
    the next heading.

    `_capability_section_body` is the right reader for a capability's fields:
    it stops at the next heading of the rendering level or shallower, which is
    what keeps one section from claiming its siblings' fields. That bound is
    wrong for anything the product renders as a heading *inside* a capability --
    the efficiency backfill block, which renders at `####` and therefore lands
    inside the body on an unclassified document and outside it on a classified
    one. This reader answers "what belongs to this capability" independently of
    the level either it or its contents rendered at.
    """
    titles = _rendered_capability_titles(migrated)
    assert title in titles, (title, titles)
    for level in ("#### ", "### "):
        marker = f"{level}{title}\n"
        start = migrated.find(marker)
        if start != -1:
            break
    rest = migrated[start + len(marker) :]
    following = titles[titles.index(title) + 1 :]
    ends = [
        at
        for other in following[:1]
        for at in (rest.find(f"\n### {other}\n"), rest.find(f"\n#### {other}\n"))
        if at != -1
    ]
    return rest if not ends else rest[: min(ends) + 1]


def _rendered_capability_titles(migrated: str) -> list[str]:
    """Every capability section title, at either heading level, in order.

    The feature roots are headings at the same level as an unclassified
    capability section, so they are excluded by name rather than by level.

    The efficiency backfill block is excluded the same way and for the same
    reason: `render_efficiency_backfill_section` emits it at `####`, exactly the
    level a classified capability section renders at, so a level-based reader
    counts it as a seventh capability. It is excluded by prefix rather than by
    exact name because its heading carries a parenthetical the product owns.
    """
    roots = {"Capability Index", "Core Features", "Non-Core Features"}
    titles = []
    for raw in migrated.splitlines():
        line = raw.strip()
        for level in ("#### ", "### "):
            if line.startswith(level):
                title = line[len(level) :].strip()
                if title not in roots and not title.startswith("Efficiency - "):
                    titles.append(title)
                break
    return titles


def assert_relocation_preserves_section_tracker_state(
    migrated: str,
    *,
    expected_order: tuple[str, ...],
    blanked_titles: frozenset[str] = frozenset(),
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
    # `blanked_titles` are the capabilities that declared no tracker state, so
    # `-` is what preserving it means for them. Passed per document rather than
    # skipped, so the rule stays quantified over every capability.
    assert blanked_titles <= set(expected_order), sorted(
        blanked_titles - set(expected_order)
    )

    def _expected_wi(title: str) -> str:
        return "-" if title in blanked_titles else SECTION_RELOCATION_WI[title]

    for row in rows:
        expected = _expected_wi(row[0])
        assert row[1] == expected, (
            f"relocated index row {row[0]!r} lost its tracker state; "
            f"expected {expected!r}, got {row[1]!r}"
        )
    for title in SECTION_RELOCATION_WI:
        body = _capability_section_body(migrated, title)
        wi = _expected_wi(title)
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


#: The closed set of spellings `is_empty_table_value` (`capability.rs:11960-11963`)
#: treats as "this cell says nothing": the empty string, `-`, and the two
#: spellings of `n/a`. Cycled across the fixture's capabilities below so each is
#: exercised by a real document rather than by the one spelling every author in
#: this fixture happened to write.
EMPTY_TABLE_VALUE_SPELLINGS = ("-", "n/a", "N/A")


def _work_root_wi_readme() -> str:
    """The section README with every `Root WI` blank and the work roots numbered.

    `root_wi_for_capability` is two branches: the declared `Root WI:` field, and
    -- only when that is absent or `-` -- the first non-empty `active_wi` among
    the capability's work roots. Every other relocation input here declares its
    own `Root WI`, so the second branch is unreachable from them and an
    implementation that deleted it entirely renders identically.

    The blank is written three ways, cycling `EMPTY_TABLE_VALUE_SPELLINGS`. The
    predicate that decides "this field is absent" is one closed match, and with
    every capability writing `-` the two `n/a` arms were free: dropping them
    makes `Root WI: n/a` read as a *declared* WI, so the fallback stops firing
    and the literal string `n/a` ships into the contract as tracker state. A
    fixture that writes one spelling cannot see that, and `n/a` in a
    hand-written README is at least as likely as `-`.
    """
    document = UNCLASSIFIED_SECTION_README
    for index, wi in enumerate(SECTION_RELOCATION_WI.values()):
        blank = EMPTY_TABLE_VALUE_SPELLINGS[index % len(EMPTY_TABLE_VALUE_SPELLINGS)]
        assert document.count(f"Root WI: {wi}\n") == 1, wi
        document = document.replace(f"Root WI: {wi}\n", f"Root WI: {blank}\n", 1)
    assert "Root WI: #" not in document, document
    for spelling in EMPTY_TABLE_VALUE_SPELLINGS:
        assert f"Root WI: {spelling}\n" in document, (
            f"every empty-value spelling must be exercised by a real capability; "
            f"{spelling!r} is missing"
        )
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
    # The input wrote its blanks three ways, and none of them may survive as a
    # declared value. Without this, an implementation that stopped recognizing
    # `n/a` as empty would keep the fallback firing for the `-` capabilities --
    # satisfying every assertion above for four of six -- while shipping the
    # literal string `n/a` as the other two capabilities' tracker state.
    for spelling in EMPTY_TABLE_VALUE_SPELLINGS:
        assert f"Root WI: {spelling}\n" not in migrated, (
            f"a relocated section still renders the empty spelling {spelling!r} "
            f"as its Root WI instead of falling back to its first work root"
        )
    # The work-root table's own `WI` cell, which is a different assignment from
    # the field the fallback feeds (`capability.rs:9074` versus `:9023`).
    # Asserting the fallback alone left `markdown_cell(&row.wi)` rewritable to
    # `-`: the field still rendered, because the fallback had already read the
    # row before the table was printed. The shadowed WI is asserted *present*
    # here and *absent* as a `Root WI:` above -- the row it came from must
    # survive even though it lost the election.
    for member in _ALL_MEMBERS:
        body = _capability_section_body(migrated, member[0])
        for work_root in member[4]:
            row = f"| {work_root} | change | {WORK_ROOT_WI[work_root]} |"
            assert row in body, (
                f"relocated section {member[0]!r} lost its work-root row for "
                f"{work_root!r}; expected {row!r}, section was:\n{body}"
            )


def assert_relocation_renders_every_capability_section(
    migrated: str,
    report: dict[str, Any],
    *,
    expected_order: tuple[str, ...],
    retired_titles: frozenset[str] = frozenset(),
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
    # `capability_count` is not `capabilities.len()`. `capability_report`
    # (`capability.rs:6267-6288`) filters `status != Retired` out of the
    # capability, verified, and claim totals while still *reporting* the
    # retired capability in the list -- a retired capability stays visible but
    # stops counting against the project's percentage. Until this round no
    # fixture document declared a `retired` capability, so the three filters
    # were identities and deleting all three left every count unchanged, which
    # is what let a project keep dragging retired work through its denominator.
    #
    # Both totals are asserted, not just the capability one, because they are
    # three separate filters over the same list: dropping the filter on
    # `claim_count` alone leaves the capability count right and the claim
    # percentage wrong.
    retired = frozenset(retired_titles)
    assert retired <= set(expected_order), sorted(retired - set(expected_order))
    counted = [member for member in _ALL_MEMBERS if member[0] not in retired]
    assert report["capability_count"] == len(counted), (
        f"`capability_count` counts the capabilities that are not retired: "
        f"expected {len(counted)} of {len(expected_ids)}, got "
        f"{report['capability_count']}"
    )
    # Quantified over every document including the no-work-root one, where the
    # count holds for a different reason: the row the renderer synthesizes for a
    # capability that declared no work root is a claim when the emitted document
    # is read back, so the total is the same integer the authored tables would
    # have produced.
    assert report["claim_count"] == sum(len(member[4]) for member in counted), (
        f"a retired capability's claims leave the claim total with its "
        f"capability: expected "
        f"{sum(len(member[4]) for member in counted)}, got "
        f"{report['claim_count']}"
    )
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


def assert_section_relocation_empties_the_readme(
    readme: str, titles: tuple[str, ...]
) -> None:
    """The README a section-shaped contract was relocated *out of* gives it up.

    `apply_readme_capability_relocation_tick` (`capability.rs:13088-13090`)
    rewrites the source README to a forwarding pointer through
    `render_readme_capability_migration_residue`. Only the legacy-table shape
    ever re-read the README afterwards; on all five section-shaped relocations
    the residue write was unobservable, so an implementation that left the
    README's whole capability contract in place -- producing two divergent copies
    of it, which is exactly what relocation exists to prevent -- passed.

    Asserted in three directions: the pointer arrives, the sections leave, and
    everything that was never part of the capability contract stays. The third
    is what separates "moved the contract out" from "truncated the README": the
    residue is built by splicing a pointer into the surviving prefix and suffix
    (`render_readme_capability_migration_residue`), and discarding the prefix
    outright still satisfies both of the other two directions.
    """
    assert README_CONTRACT_POINTER in readme, readme
    assert "CAPABILITIES.md" in readme, readme
    assert readme.count(f"\n{README_CONTRACT_POINTER}\n") == 1, (
        f"the residue must carry exactly one contract pointer, got "
        f"{readme.count(chr(10) + README_CONTRACT_POINTER + chr(10))}; "
        f"README was:\n{readme}"
    )
    for landmark in ("# Lumen", "## Brief", "## Contributing", "See CONTRIBUTING.md."):
        assert landmark in readme, (
            f"relocation must keep {landmark!r}, which was never part of the "
            f"capability contract it moved; README was:\n{readme}"
        )
    for title in titles:
        assert f"### {title}" not in readme, (
            f"relocation must remove {title!r}'s section from the README it "
            f"moved the contract out of; README was:\n{readme}"
        )
    for member in _ALL_MEMBERS:
        assert f"ID: {member[1]}" not in readme, (
            f"relocation left {member[1]!r}'s contract fields in the README; "
            f"README was:\n{readme}"
        )


#: The author's own contract pointer, worded differently from the product's, so
#: "the README ends up with a pointer" cannot be satisfied by either of the two
#: branches indifferently.
AUTHORED_CONTRACT_POINTER_BODY = (
    "The machine-readable contract lives in [CAPABILITIES.md](CAPABILITIES.md); "
    "this section is maintained by hand."
)


def _existing_pointer_readme() -> str:
    """A section-shaped README that already carries a `## Capability Contract`.

    `render_readme_capability_migration_residue` (`capability.rs:13147-13149`)
    opens with an early return for exactly this input: a README that already
    declares the pointer heading gets its sections stripped and nothing
    appended. Every other relocation input here arrives without the heading, so
    that branch was never entered, and disabling it renders a README with *two*
    `## Capability Contract` sections -- the author's and the product's, saying
    different things.

    This is the shape a second `aw capability migrate` run sees, so the branch is
    what makes relocation idempotent rather than accumulating.
    """
    document = UNCLASSIFIED_SECTION_README
    marker = "## Contributing\n"
    assert document.count(marker) == 1, document
    return document.replace(
        marker,
        f"## Capability Contract\n\n{AUTHORED_CONTRACT_POINTER_BODY}\n\n{marker}",
        1,
    )


EXISTING_POINTER_SECTION_README = _existing_pointer_readme()


def assert_relocation_keeps_an_authored_contract_pointer(readme: str) -> None:
    """A README that already pointed at its contract keeps its own wording.

    The single-pointer count is asserted for every relocation shape; this is the
    other half, and it is the half that says *which* pointer survived. A residue
    that dropped the author's section and substituted the product's boilerplate
    would still carry exactly one pointer, and would still have silently
    rewritten prose the author maintains by hand.
    """
    assert AUTHORED_CONTRACT_POINTER_BODY in readme, (
        f"relocation overwrote the pointer the README already carried; README "
        f"was:\n{readme}"
    )
    generated = "Machine-readable capability contract for "
    assert generated not in readme, (
        f"relocation appended its own pointer to a README that already had one; "
        f"README was:\n{readme}"
    )


#: The capabilities whose `Gate Inventory:` this document does not declare, and
#: what the product must derive for each.
#:
#: `capability_gate_inventory` (`capability.rs:9842-9863`) returns the declared
#: inventory when there is one and otherwise derives one by collecting each
#: claim's fixtures and then each of the capability's gate commands, joined with
#: `<br>`; `markdown_field_list_items` (`capability.rs:9139-9151`) then
#: substitutes a single `-` when that comes back empty. Every capability in every
#: other document here declares an inventory, so neither the derivation nor the
#: placeholder was reachable: collapsing the derivation to `"-"` and deleting the
#: placeholder both rendered identically.
#:
#: The members below are separated deliberately, because they reach their field
#: through different code and fail in different directions.
#:
#: `DERIVED_INVENTORY_TITLE` declares no inventory at all and has a work root
#: whose `Gate / Evidence` cell is a backticked command, which becomes a gate and
#: comes back as the inventory -- so an author who wired gates but wrote no
#: inventory still gets one.
#:
#: `MULTI_GATE_INVENTORY_TITLE` is the same shape at arity two and drawing from
#: both halves of the derivation at once, which is what makes the `<br>` join and
#: the order the two halves are collected in falsifiable at all; see its own
#: comment.
#:
#: `FIXTURE_INVENTORY_TITLE` reaches the derivation through the claim-fixture
#: half rather than the capability-gate half; see its own comment.
#:
#: `EMPTY_INVENTORY_TITLE` declares an inventory whose every entry is one of the
#: empty-table-value spellings. `capability_raw_gate_inventory` accepts it -- the
#: joined `-<br>n/a` is not itself an empty table value -- so the raw branch wins
#: and derivation never runs; `markdown_field_list_items` then filters both
#: entries away and has to substitute the placeholder. Its work root carries a
#: gate of its own, so the derivation *would* have produced something had it run:
#: asserting that gate is absent from the rendered inventory is what separates
#: "the placeholder was substituted" from "the field was silently re-derived".
#:
#: The remaining path -- a derivation with genuinely nothing to collect -- is
#: reached by `EMPTY_DERIVATION_SECTION_README` below rather than by this
#: document, because the document that reaches it is one `aw capability report`
#: rejects while this one must stay accepted.
DERIVED_INVENTORY_TITLE = "Standard Operational Endpoints"
DERIVED_INVENTORY_GATE = "`aw health --project demo`"
EMPTY_INVENTORY_TITLE = "Kubernetes-Native Deployment"
EMPTY_INVENTORY_GATE = "`aw conf check --project demo`"
#: The declared entries for `EMPTY_INVENTORY_TITLE`: two *different* empty-table
#: spellings, so an implementation that only recognises the literal `-` leaves
#: `n/a` behind and renders a one-item inventory instead of the placeholder.
EMPTY_INVENTORY_DECLARED_ITEMS = ("-", "n/a")
assert all(item in EMPTY_TABLE_VALUE_SPELLINGS for item in EMPTY_INVENTORY_DECLARED_ITEMS), (
    "the declared entries have to be spellings the product treats as empty, or "
    "the placeholder is not what is under test"
)
assert len(set(EMPTY_INVENTORY_DECLARED_ITEMS)) == len(EMPTY_INVENTORY_DECLARED_ITEMS), (
    "two identical spellings would not separate 'filters every empty spelling' "
    "from 'filters the one spelling it knows'"
)
#: `Impl` for the capability whose only work root is `planned | planned`: that
#: gap is `Open`, which is neither all-closed nor in-progress, so
#: `capability_impl_summary` (`capability.rs:9273-9295`) falls through to its
#: last arm. The varied-status document exercises the other three; this is the
#: fourth.
#:
#: Reaching it needs the status too. That function short-circuits to
#: `implemented` for a `verified` capability whatever its gaps say, which is what
#: every other section in this document is, so the status here is `confirmed`:
#: still a status the checker requires a full contract for, so nothing is
#: relaxed, but not one that answers the question before the gaps are consulted.
EMPTY_INVENTORY_STATUS = "confirmed"
EMPTY_INVENTORY_IMPL = "planned"

#: The third capability whose declared inventory is stripped, and the only one
#: that reaches the derivation with *two* work roots.
#:
#: `capability_gate_inventory` (`capability.rs:9842-9865`) accumulates one `refs`
#: list across all of a capability's claims and joins it with `<br>`. Every
#: capability that reaches the derivation elsewhere in this fixture has exactly
#: one work root, so the join runs at arity 1 -- where `refs.join("<br>")`,
#: `refs.into_iter().next().unwrap_or_default()` and `refs.pop().unwrap_or_default()`
#: are byte-identical. Keeping the first element and keeping the last are
#: *different* mutations and a two-item list separates them only if the two items
#: are distinct and their order is asserted, so the two cells below are distinct
#: and the rendered order is asserted exactly.
#:
#: It is also the only capability that reaches the derivation through *both*
#: halves at once. The function collects every claim's fixtures first and appends
#: the capability's gates afterwards, so which half a ref came from decides where
#: it lands. With one capability deriving gates only and another fixtures only,
#: the two halves never meet in one list and swapping the two `refs.extend`
#: blocks renders the identical document -- the arity was fixed and the fixture
#: half was woken up, but their *composition* stayed unbound. So this capability
#: declares cells of both kinds: a backticked piece parses as a gate, a bare one
#: as a fixture.
#:
#: One of each was not enough either. Composing the halves at one ref apiece left
#: three narrower truncations rendering the identical document: `.take(1)` on the
#: gate half, `.take(1)` on a claim's fixtures, and visiting the claims in
#: reverse. A `Gate / Evidence` cell splits on `<br>`
#: (`capability.rs:11921-11925`), so one work root can carry several pieces of
#: either kind, and the two halves are widened without changing any capability's
#: work-root count -- which the per-class claim arithmetic elsewhere is pinned
#: to. The declaration below therefore spreads *two* gates across the two work
#: roots, spreads *three* fixtures across them, and gives the first work root two
#: fixtures of its own, so each of those three truncations drops a different ref.
#:
#: The order is chosen deliberately. The first piece declared is a gate and the
#: first item rendered is a fixture, so the fixtures-then-gates order and the
#: declaration order disagree; a rendering that simply followed declaration order
#: -- which is what swapping the two `refs.extend` blocks produces -- is a
#: different document. The two fixtures on the first root come before the fixture
#: on the second, so reversing the claim walk is a different document too.
MULTI_GATE_INVENTORY_TITLE = "Lexical Search"
#: The `Gate / Evidence` pieces each of the two work roots declares, in
#: declaration order. Joined with `<br>` into one cell per root.
MULTI_GATE_INVENTORY_PIECES = (
    (
        "`aw ec verify --project demo`",
        "evidence/lexical-search-planner.md",
        "evidence/lexical-search-analyzer.md",
    ),
    (
        "evidence/lexical-search-recall.md",
        # Two commands inside *one* piece. `split_gate_evidence_pieces`
        # (`capability.rs:11921-11927`) splits a cell only on `<br>` and on
        # newlines, so this stays a single piece, and
        # `capability_claim_evidence_from_table` then loops
        # `extract_backtick_values` over it -- a second, inner list whose arity
        # was 1 in every piece this fixture declared. Truncating that loop to
        # `.take(1)` rendered the whole document byte for byte.
        #
        # This is the shape this repository's own contract carries, not a
        # contrivance: `apps/agentic-workflow/CAPABILITIES.md` contains no `<br>`
        # at all and separates a work root's gate commands with `; ` inside one
        # cell, which is one piece by the rule above.
        "`aw ec check --project demo` ; `aw ec verify --project demo --stage td`",
        # A third gate in a *separate* piece, so the cross-piece accumulation
        # (`gates` persisting across iterations of the outer loop) stays bound
        # too. Collapsing the two levels into one would leave whichever level
        # the fix did not reach free -- which is exactly how the inner one
        # survived the round that added the second gate.
        "`aw ec verify --project demo --stage cb`",
    ),
)
MULTI_GATE_INVENTORY_CELLS = tuple(
    "<br>".join(pieces) for pieces in MULTI_GATE_INVENTORY_PIECES
)


def _gate_commands(piece: str) -> tuple[str, ...]:
    """The backticked commands one cell piece declares, in order.

    `extract_backtick_values` (`capability.rs:11929-11944`) restated: a piece
    contributes one gate per backticked span, not one gate per piece.
    """
    commands: list[str] = []
    rest = piece
    while "`" in rest:
        _, _, after = rest.partition("`")
        value, sep, rest = after.partition("`")
        if not sep:
            break
        if value.strip():
            commands.append(value.strip())
    return tuple(commands)


def _is_gate_piece(piece: str) -> bool:
    """A piece carrying any backticked command parses as gates; a bare one as a fixture.

    Keyed on "carries a backticked value" rather than "starts and ends with a
    backtick", which is the product's own test (`commands.is_empty()`,
    `capability.rs:11896`). The two agreed only while every gate piece was
    exactly one command; a piece holding two commands separated by `; ` is a
    gate piece that does not end where it starts.
    """
    return bool(_gate_commands(piece))


_MULTI_GATE_DECLARED = tuple(
    piece for pieces in MULTI_GATE_INVENTORY_PIECES for piece in pieces
)
#: The same refs in the order the derivation must render them: every claim's
#: fixtures in claim order first, then the capability's gates.
#:
#: The gate half is per *command*, not per piece: the inventory is rebuilt from
#: `capability.evidence.verification` as one `` `cmd` `` item each
#: (`capability.rs:9852-9858`), so a piece declaring two commands owes two items.
MULTI_GATE_INVENTORY_ITEMS = tuple(
    [piece for piece in _MULTI_GATE_DECLARED if not _is_gate_piece(piece)]
    + [
        f"`{command}`"
        for piece in _MULTI_GATE_DECLARED
        for command in _gate_commands(piece)
    ]
)
assert len(set(_MULTI_GATE_DECLARED)) == len(_MULTI_GATE_DECLARED), (
    "two identical pieces would not separate keep-first from keep-last"
)
assert MULTI_GATE_INVENTORY_ITEMS != _MULTI_GATE_DECLARED, (
    "the rendered order has to differ from the declared order, or "
    "'fixtures then gates' and 'declaration order' render the same document"
)
assert sum(
    1 for pieces in MULTI_GATE_INVENTORY_PIECES if any(map(_is_gate_piece, pieces))
) == 2, (
    "both work roots have to contribute a gate, or truncating the gate half to "
    "its first ref renders the identical document"
)
assert sum(
    1
    for pieces in MULTI_GATE_INVENTORY_PIECES
    if any(not _is_gate_piece(piece) for piece in pieces)
) == 2, (
    "both work roots have to contribute a fixture, or the order the claims are "
    "walked in is unobservable"
)
assert any(
    sum(1 for piece in pieces if not _is_gate_piece(piece)) > 1
    for pieces in MULTI_GATE_INVENTORY_PIECES
), (
    "one work root has to contribute more than one fixture, or truncating a "
    "single claim's fixture list renders the identical document"
)
assert any(
    sum(len(_gate_commands(piece)) for piece in pieces) > 1
    for pieces in MULTI_GATE_INVENTORY_PIECES
), (
    "one work root has to contribute more than one *gate*, or the id rule for "
    "a claim's second gate (`capability.rs:11903-11907`) is never reached"
)
assert any(
    len(_gate_commands(piece)) > 1
    for pieces in MULTI_GATE_INVENTORY_PIECES
    for piece in pieces
), (
    "one *piece* has to declare more than one command, or the inner loop over "
    "`extract_backtick_values` (`capability.rs:11902`) is composed at arity 1 "
    "and truncating it to the first command renders the identical document. "
    "Two gates in two pieces do not reach it: that binds the outer loop only"
)
assert any(
    len(_gate_commands(piece)) > 1
    for pieces in MULTI_GATE_INVENTORY_PIECES
    for piece in pieces
) and any(
    sum(1 for piece in pieces if _is_gate_piece(piece)) > 1
    for pieces in MULTI_GATE_INVENTORY_PIECES
), (
    "one work root has to declare gates at both levels at once -- two commands "
    "in one piece *and* more than one gate-bearing piece -- or the level the "
    "fixture did not reach stays free while the numbering looks bound"
)


#: What `capability_claim_evidence_from_table` (`capability.rs:11887-11919`) has
#: to build for the multi-gate capability, keyed by work root -- which is also
#: the claim id and the `proves` value.
#:
#: A claim's *first* gate is `<gap>-gate` and every later one is
#: `<gap>-gate-<n>`. Until one work root declared two gates, that branch was
#: unreachable from this whole fixture: collapsing the id rule to a bare
#: `<gap>-gate` rendered every document byte for byte. It is not cosmetic -- two
#: gates sharing an id is a hard error the checker bails on
#: (`capability.rs:10231-10238`), so the collapse turns a legitimate two-gate
#: work root, which this repository's own `CAPABILITIES.md` carries, into a
#: refused contract.
#:
#: The numbering is counted over *commands*, not over pieces, because the
#: product counts `gates.len()` -- a single accumulator shared by both loops.
#: Enumerating pieces instead agreed with it only while no piece held two
#: commands, and that agreement is what let the inner loop stay unbound.
#:
#: Derived from the declared pieces by the product's own rule rather than written
#: out, so the expectation cannot be edited into agreement with a changed
#: fixture.
MULTI_GATE_CLAIM_EVIDENCE = {
    work_root: {
        "gates": tuple(
            (
                f"{work_root}-gate" if position == 0
                else f"{work_root}-gate-{position + 1}",
                command,
            )
            for position, command in enumerate(
                command for piece in pieces for command in _gate_commands(piece)
            )
        ),
        "fixtures": tuple(
            piece for piece in pieces if not _is_gate_piece(piece)
        ),
    }
    for work_root, pieces in zip(
        next(
            member[4]
            for member in _ALL_MEMBERS
            if member[0] == MULTI_GATE_INVENTORY_TITLE
        ),
        MULTI_GATE_INVENTORY_PIECES,
    )
}


def assert_derived_claims_carry_their_own_gates(report: dict[str, Any]) -> None:
    """Each derived claim's gates and fixtures are its own work root's.

    The migrated document renders the *union* of a capability's refs as one
    `Gate Inventory:` field, so nothing in it can see which claim a ref came
    from, what id the gate was given, or which work root it proves. All three
    are read off the report instead, per claim, as exact ordered lists.

    The gate id is the reason this leg exists. Every work root in this fixture
    carried at most one gate, which left the numbering branch unreachable and a
    real two-gate work root -- the shape
    `apps/agentic-workflow/CAPABILITIES.md:147` carries -- rendering two gates
    with the same id, which the checker refuses outright.
    """
    capability = next(
        item
        for item in report["capabilities"]
        if item["title"] == MULTI_GATE_INVENTORY_TITLE
    )
    claims = {claim["id"]: claim for claim in capability["claims"]}
    assert set(claims) == set(MULTI_GATE_CLAIM_EVIDENCE), (
        f"{MULTI_GATE_INVENTORY_TITLE!r} must derive one claim per work root; "
        f"got {sorted(claims)}, expected {sorted(MULTI_GATE_CLAIM_EVIDENCE)}"
    )
    for work_root, expected in MULTI_GATE_CLAIM_EVIDENCE.items():
        claim = claims[work_root]
        rendered_gates = tuple(
            (gate["id"], gate["command"]) for gate in claim["gates"]
        )
        assert rendered_gates == expected["gates"], (
            f"claim {work_root!r} must carry the gates {expected['gates']!r}, "
            f"in that order and nothing else; got {rendered_gates!r}"
        )
        assert all(gate["proves"] == work_root for gate in claim["gates"]), (
            f"every gate of claim {work_root!r} proves that work root; got "
            f"{[gate['proves'] for gate in claim['gates']]!r}"
        )
        rendered_fixtures = tuple(claim["fixtures"])
        assert rendered_fixtures == expected["fixtures"], (
            f"claim {work_root!r} must carry the fixtures "
            f"{expected['fixtures']!r}, in that order and nothing else; got "
            f"{rendered_fixtures!r}"
        )

#: The fourth capability whose declared inventory is stripped, and the only one
#: whose work-root `Gate / Evidence` cell is *not* backticked.
#:
#: The derivation reads two lists off each claim -- `claim.fixtures` and
#: `claim.gates` -- and a backticked cell parses as a gate. Every work root in
#: this document was backticked, so deleting the `claim.fixtures` half of the
#: derivation changed nothing: half the function was dead against the fixture
#: while the assertion above read as if it bound the whole of it. A non-backticked
#: cell parses as a fixture instead, which is the form this repository's own
#: `apps/agentic-workflow/CAPABILITIES.md` carries, so the input is not exotic.
FIXTURE_INVENTORY_TITLE = "Security Hardening"
FIXTURE_INVENTORY_EVIDENCE = "evidence/security-hardening-baseline.md"
assert "`" not in FIXTURE_INVENTORY_EVIDENCE, (
    "a backticked cell parses as a gate, which is the half of the derivation "
    "that was already bound; this one has to reach `claim.fixtures`"
)

#: Every title whose declared `Gate Inventory` this document strips, so the
#: rendered field can only be what the derivation produced.
DERIVED_INVENTORY_STRIPPED_TITLES = (
    DERIVED_INVENTORY_TITLE,
    MULTI_GATE_INVENTORY_TITLE,
    FIXTURE_INVENTORY_TITLE,
)
_MULTI_GATE_WORK_ROOTS = next(
    member[4] for member in _ALL_MEMBERS if member[0] == MULTI_GATE_INVENTORY_TITLE
)
assert len(_MULTI_GATE_WORK_ROOTS) == len(MULTI_GATE_INVENTORY_CELLS) == 2, (
    "the arity-1 blind spot is closed by a capability with exactly two work "
    "roots, each carrying pieces of both kinds"
)


def _derived_inventory_work_root_cells() -> dict[str, tuple[str, str, str, str, str]]:
    """The `Gate / Evidence` cell each work root carries in this one document.

    Written as a loop rather than a comprehension because the multi-gate member
    needs a *different* cell per work root, keyed on the root's position, and the
    two must not collapse to one value.
    """
    cells: dict[str, tuple[str, str, str, str, str]] = {}
    for member in _ALL_MEMBERS:
        title = member[0]
        for position, work_root in enumerate(member[4]):
            if title == EMPTY_INVENTORY_TITLE:
                gate = EMPTY_INVENTORY_GATE
                readiness = ("planned", "planned")
            elif title == DERIVED_INVENTORY_TITLE:
                gate = DERIVED_INVENTORY_GATE
                readiness = ("implemented", "verified")
            elif title == MULTI_GATE_INVENTORY_TITLE:
                gate = MULTI_GATE_INVENTORY_CELLS[position]
                readiness = ("implemented", "verified")
            elif title == FIXTURE_INVENTORY_TITLE:
                gate = FIXTURE_INVENTORY_EVIDENCE
                readiness = ("implemented", "verified")
            else:
                gate = "`true`"
                readiness = ("implemented", "verified")
            cells[work_root] = ("change", readiness[0], readiness[1], "smoke", gate)
    return cells


_DERIVED_INVENTORY_WORK_ROOT_CELLS = _derived_inventory_work_root_cells()

#: What each of these four capabilities renders as its `Gate Inventory`, for the
#: whole-field-block assertion that runs over every capability in this document.
#: `assert_relocation_derives_a_missing_gate_inventory` asserts the same lists
#: again with the derivation's own reasoning attached; here they are what keeps
#: the four derived fields from being exempted from the block assertion.
DERIVED_INVENTORY_ITEM_OVERRIDES = {
    DERIVED_INVENTORY_TITLE: {"gate_inventory": (DERIVED_INVENTORY_GATE,)},
    EMPTY_INVENTORY_TITLE: {"gate_inventory": ("-",)},
    MULTI_GATE_INVENTORY_TITLE: {"gate_inventory": MULTI_GATE_INVENTORY_ITEMS},
    FIXTURE_INVENTORY_TITLE: {"gate_inventory": (FIXTURE_INVENTORY_EVIDENCE,)},
}


def _derived_inventory_readme() -> str:
    """Drop one capability's declared inventory and empty another's.

    The rewrite addresses the block by the same rule `_capability` wrote it, and
    asserts the rewrite matched, so a change to the section builder surfaces here
    instead of silently leaving both capabilities declaring an inventory.
    """
    document = _section_readme(
        _ALL_MEMBERS,
        (None,) * len(_ALL_MEMBERS),
        "Lumen README-resident capability contract, one capability declaring no "
        "gate inventory and one declaring an empty inventory.",
        statuses=tuple(
            EMPTY_INVENTORY_STATUS if member[0] == EMPTY_INVENTORY_TITLE else "verified"
            for member in _ALL_MEMBERS
        ),
        work_root_cells=_DERIVED_INVENTORY_WORK_ROOT_CELLS,
    )
    replacements = {
        **{title: "" for title in DERIVED_INVENTORY_STRIPPED_TITLES},
        EMPTY_INVENTORY_TITLE: "Gate Inventory:\n"
        + "".join(f"- {item}\n" for item in EMPTY_INVENTORY_DECLARED_ITEMS),
    }
    for title, replacement in replacements.items():
        cap_id = next(member[1] for member in _ALL_MEMBERS if member[0] == title)
        block = f"Gate Inventory:\n- {MEMBER_GATE_INVENTORY_ITEM[title]}\n"
        assert block == f"Gate Inventory:\n- tech-design/{cap_id}.md\n", title
        assert document.count(block) == 1, (title, document.count(block))
        document = document.replace(block, replacement, 1)
    return document


DERIVED_INVENTORY_SECTION_README = _derived_inventory_readme()
for _gate in (
    DERIVED_INVENTORY_GATE,
    EMPTY_INVENTORY_GATE,
    FIXTURE_INVENTORY_EVIDENCE,
    *_MULTI_GATE_DECLARED,
):
    assert _gate not in UNCLASSIFIED_SECTION_README, (
        f"{_gate} must not be a string the fixture already writes elsewhere, or "
        "'it was derived' is indistinguishable from 'it was copied'"
    )
    assert DERIVED_INVENTORY_SECTION_README.count(_gate) == 1, (
        f"{_gate} has to appear exactly once in the input -- in its own work-root "
        "row -- or the rendered inventory cannot be attributed to it"
    )
del _gate
for _title in DERIVED_INVENTORY_STRIPPED_TITLES:
    assert (
        f"- {MEMBER_GATE_INVENTORY_ITEM[_title]}\n" not in DERIVED_INVENTORY_SECTION_README
    ), (
        f"{_title!r} still declares its own gate inventory, so whatever renders "
        "for it is the declared value carried through, not a derivation"
    )
del _title


#: Relocation input whose one capability declares no gate inventory *and* whose
#: only work root carries an empty `Gate / Evidence` cell, so
#: `capability_gate_inventory` (`capability.rs:9842-9863`) collects no fixture
#: and no gate and returns through its `refs.is_empty()` arm.
#:
#: Held as its own document because `aw capability report` rejects what
#: `aw capability migrate` writes here -- the emitted claim has neither a gate nor
#: a fixture, which the checker requires -- and every other relocation shape in
#: this case asserts an empty document-blocker set. That disagreement between the
#: two verbs is the point of the fixture and is filed as #3215; it is asserted
#: rather than disclosed, because a document `migrate` accepts is a document this
#: case can drive whether or not `report` likes the result.
EMPTY_DERIVATION_TITLE = "Search Core"
EMPTY_DERIVATION_CAPABILITY = next(
    member[1] for member in _ALL_MEMBERS if member[0] == EMPTY_DERIVATION_TITLE
)
EMPTY_DERIVATION_CLAIM = next(
    member[4][0] for member in _ALL_MEMBERS if member[0] == EMPTY_DERIVATION_TITLE
)
EMPTY_DERIVATION_BLOCKER = (
    f"claim `{EMPTY_DERIVATION_CLAIM}` in capability `{EMPTY_DERIVATION_CAPABILITY}` "
    "requires at least one gate or fixture/inventory reference"
)


def _empty_derivation_readme() -> str:
    """Strip one capability's declared inventory and empty its work-root cell.

    Both halves are needed and neither alone reaches the arm: with the inventory
    declared, `capability_raw_gate_inventory` returns before the derivation runs;
    with the cell populated, the derivation collects it.
    """
    cells = {
        work_root: (
            ("change", "implemented", "verified", "smoke", "-")
            if member[0] == EMPTY_DERIVATION_TITLE
            else ("change", "implemented", "verified", "smoke", "`true`")
        )
        for member in _ALL_MEMBERS
        for work_root in member[4]
    }
    document = _section_readme(
        _ALL_MEMBERS,
        (None,) * len(_ALL_MEMBERS),
        "Lumen README-resident capability contract, one capability declaring no "
        "gate inventory and gating nothing.",
        work_root_cells=cells,
    )
    block = f"Gate Inventory:\n- tech-design/{EMPTY_DERIVATION_CAPABILITY}.md\n"
    assert document.count(block) == 1, (EMPTY_DERIVATION_TITLE, document.count(block))
    return document.replace(block, "", 1)


EMPTY_DERIVATION_SECTION_README = _empty_derivation_readme()


def assert_relocation_renders_an_underivable_gate_inventory(
    migrated: str, report: dict[str, Any]
) -> None:
    """Nothing to derive renders the placeholder, and the checker then rejects it.

    This is the last unentered path through the gate-inventory field:
    `capability_raw_gate_inventory` returns `None`, and the derivation behind it
    collects neither a claim fixture nor a capability gate.

    What is bound here is that the arm is *entered* and yields an empty table
    value -- a marker returned in its place renders as the item `- MARKER`.
    Deleting the arm outright is not observable and is not claimed to be:
    `refs.join("<br>")` over an empty list is the empty string, which
    `markdown_field_list_items` (`capability.rs:9139-9151`) filters away and
    replaces with the same `-`. The two spellings of "no inventory" collapse
    downstream, so the assertion says what it can see rather than borrowing the
    stronger claim from the sibling arms.

    The blocker set is asserted alongside, because it is the reason this document
    is held apart from every other relocation shape here: `aw capability migrate`
    writes a capability whose only claim references neither a gate nor a fixture,
    and `aw capability report` -- reading the document the same run just produced
    -- rejects exactly that claim. Asserted as the whole ordered document-blocker
    set, so the disagreement is pinned to one claim rather than to "something was
    reported". Filed as #3215.
    """
    body = _capability_section_body(migrated, EMPTY_DERIVATION_TITLE)
    items = _gate_inventory_items(body, EMPTY_DERIVATION_TITLE)
    assert items == ["-"], (
        f"{EMPTY_DERIVATION_TITLE!r} declared no gate inventory and gates "
        f"nothing, so the field must render the single placeholder; got "
        f"{items!r}. Section was:\n{body}"
    )
    assert document_blockers(report) == [EMPTY_DERIVATION_BLOCKER], report["blockers"]


def _field_list_items(body: str, title: str, field: str) -> list[str]:
    """One list-shaped field of a rendered section, as its item list.

    Read as a list rather than as a substring so the assertions below can pin
    what the field *is* -- its length and the order of its entries -- instead of
    what it contains. A containment check cannot see an item dropped from the
    end, nor one appended after the ones it names: the expected block is a
    substring of a longer rendered block either way. Both were live here. The
    gate inventory carried the first shape, and the dependency block carried the
    second -- a parse that stopped deduplicating rendered the repeated dependency
    a second time, after the two the assertion named, and the containment check
    did not see it.

    The field is bounded by the first line that is not an item, because the
    renderer emits every field of the section into one block and the field that
    follows this one differs by shape of capability.
    """
    marker = f"{field}:\n"
    assert marker in body, (
        f"section {title!r} rendered no {field!r} at all; section was:\n{body}"
    )
    items = []
    for line in body.split(marker, 1)[1].splitlines():
        if not line.startswith("- "):
            break
        items.append(line[2:])
    return items


def _gate_inventory_items(body: str, title: str) -> list[str]:
    """The `Gate Inventory` field of one rendered section, as its item list."""
    return _field_list_items(body, title, "Gate Inventory")


def assert_relocation_derives_a_missing_gate_inventory(migrated: str) -> None:
    """A capability that declared no gate inventory gets the one its claims imply.

    Four capabilities are asserted on the same document, because they fail in
    different directions and each alone reads as another's success. Each is
    asserted as the *exact* item list its field renders, not as a substring of
    it: a containment check on the first item cannot see a second item dropped,
    and the derivation's whole output is one joined list.

    `DERIVED_INVENTORY_TITLE` declared nothing and must come back carrying its
    work-root command -- not `-`, which is what a collapsed derivation renders.

    `MULTI_GATE_INVENTORY_TITLE` declared nothing and has *two* work roots, so
    its derived field is the only two-item list here. Every other capability that
    reaches the derivation has one work root, where joining the list, keeping its
    first element and keeping its last are byte-identical; two distinct refs
    asserted in exact order separate all three.

    It is also the only capability whose two refs come from *different* halves of
    the derivation -- a gate from the first work root, a fixture from the second
    -- so the order the two halves are collected in is asserted rather than
    assumed. Everywhere else each capability draws from one half only, so the
    fixture half and the gate half never share a list and swapping the two
    collections renders the identical document. The declaration order is the
    reverse of the rendered order on purpose: fixtures come first however they
    were declared, so an implementation that emitted refs in work-root order
    fails here too.

    `FIXTURE_INVENTORY_TITLE` declared nothing and its work-root cell is not
    backticked, so it parses as a claim *fixture* rather than a claim gate. The
    derivation reads both lists; with every cell in this document backticked, the
    fixture half of it was dead and deleting it changed nothing.

    `EMPTY_INVENTORY_TITLE` declared only empty-table spellings and must come
    back carrying `-` -- not an empty list, which is what deleting the
    placeholder renders and which is not a document the checker accepts. That arm
    also pins which branch produced the `-`: the capability has a work-root gate
    of its own, so a renderer that ignored the declared inventory and derived one
    anyway would render that command here. Asserting it is absent from the field
    while still present in the section separates "the placeholder was
    substituted" from "the author's empty inventory was quietly replaced".

    The two capabilities that keep their declared inventory are left alone in the
    same document, so "derive one" cannot be satisfied by an implementation that
    derives one for everybody and discards what the author wrote.
    """
    for title, expected_items in (
        (DERIVED_INVENTORY_TITLE, [DERIVED_INVENTORY_GATE]),
        (MULTI_GATE_INVENTORY_TITLE, list(MULTI_GATE_INVENTORY_ITEMS)),
        (FIXTURE_INVENTORY_TITLE, [FIXTURE_INVENTORY_EVIDENCE]),
        (EMPTY_INVENTORY_TITLE, ["-"]),
    ):
        body = _capability_section_body(migrated, title)
        items = _gate_inventory_items(body, title)
        assert items == expected_items, (
            f"section {title!r} must render the gate inventory "
            f"{expected_items!r}, in that order and nothing else; got {items!r}. "
            f"Section was:\n{body}"
        )
    empty = _capability_section_body(migrated, EMPTY_INVENTORY_TITLE)
    assert EMPTY_INVENTORY_GATE in empty, (
        f"{EMPTY_INVENTORY_GATE} has to survive somewhere in "
        f"{EMPTY_INVENTORY_TITLE!r} -- its work-root row -- or the absence of it "
        f"from the inventory above proves nothing; section was:\n{empty}"
    )
    rows = {row[0]: row for row in _index_rows_parsed(migrated)}
    assert rows[EMPTY_INVENTORY_TITLE][2] == EMPTY_INVENTORY_IMPL, (
        f"{EMPTY_INVENTORY_TITLE!r}'s only work root is open, so Impl must "
        f"derive {EMPTY_INVENTORY_IMPL!r}, got {rows[EMPTY_INVENTORY_TITLE][2]!r}"
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
    capability_type = _member_type_for_id(cap_id)
    marker = f"ID: {cap_id}\nType: {capability_type}\nFeature Class: core\n"
    assert document.count(marker) == 1, (
        f"expected exactly one declared-core section for {cap_id}; "
        f"found {document.count(marker)}"
    )
    return document.replace(marker, f"ID: {cap_id}\nType: {capability_type}\n")


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


# ---------------------------------------------------------------------------
# Work-root row reading: surface identity and gap status
# ---------------------------------------------------------------------------

#: A capability whose `Surfaces:` list declares the same surface twice, and
#: four near-misses. `dedupe_surfaces` keys on
#: `normalize_table_token(kind) : commands.join(",") : summary`, so each of the
#: three key fields gets one pair that differs in it alone. Without the pairs, a
#: key that dropped the summary or the commands still folds the exact duplicate
#: and nothing observable changes.
#:
#: The fifth subject is the kind term, added in round 32. Round 31 shipped this
#: block with three subjects and claimed all three key fields; a reviewer proved
#: that no pair anywhere in the case differed in kind alone, so
#: `normalize_table_token(&surface.kind)` could be dropped from the key with
#: every expectation still green. `Unknown Kind Case` does not cover it -- that
#: pair *folds*, so it binds the case-folding inside the kind term rather than
#: the presence of the term.
SURFACE_DEDUPE_SUBJECTS = (
    # (title, cap_id, authored items, expected rendered items)
    (
        "Exact Duplicate",
        "exact-duplicate",
        ("CLI: `aw one` -- the same item", "CLI: `aw one` -- the same item"),
        ("- CLI: `aw one` - - the same item",),
    ),
    (
        "Summary Differs",
        "summary-differs",
        ("CLI: `aw two` -- first summary", "CLI: `aw two` -- second summary"),
        (
            "- CLI: `aw two` - - first summary",
            "- CLI: `aw two` - - second summary",
        ),
    ),
    (
        "Command Differs",
        "command-differs",
        ("CLI: `aw three` -- shared summary", "CLI: `aw four` -- shared summary"),
        (
            "- CLI: `aw three` - - shared summary",
            "- CLI: `aw four` - - shared summary",
        ),
    ),
    # An unrecognized kind, spelled two ways. `normalize_surface_kind` has no
    # alias for `Probe`, so its `_ => value.trim()` fallback keeps the authored
    # spelling verbatim in the render -- while the dedupe key's own
    # `normalize_table_token` case-folds, so the two spellings are one surface.
    # The rendered kind is the *first* spelling, which is what pins the fallback
    # as pass-through rather than canonicalizing.
    (
        "Unknown Kind Case",
        "unknown-kind-case",
        ("Probe: `aw five` -- shared summary", "probe: `aw five` -- shared summary"),
        ("- Probe: `aw five` - - shared summary",),
    ),
    # The kind term on its own: same command, same summary, two *recognized*
    # kinds that `normalize_surface_kind` folds to different canonical spellings.
    # Nothing but the kind term of the dedupe key keeps these two apart, so a key
    # built as `commands.join(",") : summary` folds them into one item and this
    # is the only expectation in the case that notices.
    (
        "Kind Differs",
        "kind-differs",
        ("CLI: `aw six` -- shared summary", "HTTP: `aw six` -- shared summary"),
        (
            "- CLI: `aw six` - - shared summary",
            "- HTTP: `aw six` - - shared summary",
        ),
    ),
)

#: Deliberately not all-1 and not all-2: a dedupe that folded everything, and
#: one that folded nothing, are each refuted by the count alone.
SURFACE_DEDUPE_COUNTS = tuple(len(subject[3]) for subject in SURFACE_DEDUPE_SUBJECTS)
assert set(SURFACE_DEDUPE_COUNTS) == {1, 2}, SURFACE_DEDUPE_COUNTS


def _work_root_row(work_root: str) -> str:
    return f"| {work_root} | change | implemented | verified | smoke | `true` |"


def surface_dedupe_document() -> str:
    """A README whose capabilities exercise every field of the dedupe key."""
    sections = []
    for title, cap_id, authored, _expected in SURFACE_DEDUPE_SUBJECTS:
        items = "\n".join(f"- {item}" for item in authored)
        sections.append(
            f"""### {title}

ID: {cap_id}
Root WI: -
Status: implemented
Type: Service
Feature Class: core
Promise: Promise for {cap_id}.
Current State: Partially implemented.
Required Verification: smoke
Surfaces:
{items}
EC Dimensions:
- behavior: `true` -- behaviour of {cap_id}

| Work Root | Kind | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---|---|---|---|
{_work_root_row(f"{title} root")}

"""
        )
    return (
        "# Demo\n\n## Brief\n\nSurface identity under migration.\n\n"
        "## Capabilities\n\n" + "".join(sections)
    )


SURFACE_DEDUPE_DOCUMENT = surface_dedupe_document()


def _rendered_surface_items(migrated: str, title: str) -> list[str]:
    body = _capability_section_body(migrated, title)
    items: list[str] = []
    collecting = False
    for line in body.splitlines():
        if line.startswith("Surfaces:"):
            collecting = True
            continue
        if collecting:
            if line.startswith("- "):
                items.append(line)
            elif line.strip():
                break
    return items


def assert_surface_identity_is_the_whole_declared_item(migrated: str) -> None:
    """Two declared surfaces are one surface only when kind, commands, and
    summary all agree.

    `dedupe_surfaces` builds its key from three fields. A key that dropped any
    one of them still collapses an exact duplicate, which is the only shape the
    rest of this case ever declares -- so the fold looked bound while two of its
    three fields were free. Each subject here differs from its partner in
    exactly one field, so dropping that field from the key merges a pair the
    document declares as two, and the rendered item count moves.

    The unrecognized-kind subject binds a second thing: the key case-folds
    (`Probe` and `probe` are one surface) while the *render* does not (the item
    reads `Probe`, the authored spelling). Those are two different normalizers
    over the same token, and asserting only the count would leave the render
    free to canonicalize the kind to anything it liked.
    """
    for title, _cap_id, _authored, expected in SURFACE_DEDUPE_SUBJECTS:
        rendered = _rendered_surface_items(migrated, title)
        assert rendered == list(expected), (title, rendered, list(expected))


#: One work-root row per gap status, each in its own capability so the
#: capability-level fold in `capability_impl_summary` attributes to that row
#: alone. `capability report` names the gap status directly, which is a sharper
#: observable than the folded Index cell -- both are asserted.
#:
#: (title, cap_id, Impl cell, Verification cell, expected gap status, expected
#:  rendered Index Impl cell)
GAP_STATUS_SUBJECTS = (
    ("Blocked Row", "blocked-row", "implemented", "blocked", "blocked", "planned"),
    # The same arm through its *other* disjunct. Round 31 entered the blocked
    # arm only from the verification side, leaving `implementation == "blocked"`
    # deletable; reviewer finding F103. The two blocked subjects are the reason
    # the status set below is checked for coverage rather than for uniqueness.
    ("Impl Blocked Row", "impl-blocked-row", "blocked", "planned", "blocked", "planned"),
    ("Deferred Row", "deferred-row", "out_of_scope", "planned", "deferred", "implemented"),
    ("Open Row", "open-row", "planned", "none", "open", "planned"),
    ("In Progress Row", "in-progress-row", "partial", "planned", "in_progress", "partial"),
)

#: Every arm of the match is entered, and no two subjects present the same cell
#: pair -- so a subject cannot be satisfied by the arm another subject is there
#: to bind. The Index cells are deliberately *not* distinct (both blocked rows
#: and the open row all fold to `planned`), which is why the gap status is
#: asserted by name too.
assert {subject[4] for subject in GAP_STATUS_SUBJECTS} == {
    "blocked",
    "deferred",
    "open",
    "in_progress",
}, GAP_STATUS_SUBJECTS
assert len({(subject[2], subject[3]) for subject in GAP_STATUS_SUBJECTS}) == len(
    GAP_STATUS_SUBJECTS
), GAP_STATUS_SUBJECTS


def gap_status_document() -> str:
    sections = []
    for title, cap_id, impl, verification, _status, _index in GAP_STATUS_SUBJECTS:
        sections.append(
            f"""### {title}

ID: {cap_id}
Root WI: -
Status: implemented
Type: Service
Feature Class: core
Promise: Promise for {cap_id}.
Current State: Partially implemented.
Required Verification: smoke
Surfaces:
- CLI: `aw {cap_id}` -- surface of {cap_id}
EC Dimensions:
- behavior: `true` -- behaviour of {cap_id}

| Work Root | Kind | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---|---|---|---|
| {title} root | change | {impl} | {verification} | smoke | `true` |

"""
        )
    return (
        "# Demo\n\n## Brief\n\nWork-root status folding.\n\n"
        "## Capabilities\n\n" + "".join(sections)
    )


GAP_STATUS_DOCUMENT = gap_status_document()


def assert_work_root_cells_fold_into_gap_status(
    report: dict[str, Any], migrated: str
) -> None:
    """Each `(Impl, Verification)` cell pair reads as one gap status.

    `capability_gap_status_from_table` is a five-arm match and the rest of this
    case only ever declares rows that reach `Closed`. Every other arm is entered
    below: the blocked arm through *both* of its disjuncts (one row blocked on
    the verification side, one on the implementation side), the `out_of_scope`
    guard, the `none` spelling of an open row, and the catch-all in-progress
    fallthrough. Two rows reaching the same status through different disjuncts
    is the point of the pair -- either disjunct alone leaves the other free.

    Asserted twice over. The gap status is named directly by `capability
    report`, which is exact. The Index `Impl` cell is the rendered consequence
    through `capability_impl_summary`, which is lossy -- `blocked` and `open`
    both fold to `planned` -- but it is what an adopter reads, and binding only
    the internal name would leave the fold free to route any status anywhere.
    """
    statuses = {}
    for item in report["capabilities"]:
        for gap in item.get("gaps") or []:
            statuses[gap["id"]] = gap.get("status")

    expected_statuses = {
        _slugify(f"{title} root"): status
        for title, _cap_id, _impl, _verification, status, _index in GAP_STATUS_SUBJECTS
    }
    for gap_id, expected in expected_statuses.items():
        assert statuses.get(gap_id) == expected, (gap_id, statuses.get(gap_id), expected)

    index_impl = {
        row[0]: row[2]
        for row in _index_rows_parsed(migrated)
    }
    for title, _cap_id, _impl, _verification, _status, expected_cell in GAP_STATUS_SUBJECTS:
        assert index_impl.get(title) == expected_cell, (
            title,
            index_impl.get(title),
            expected_cell,
        )


#: The YAML reading form is the only one that produces a capability with gaps
#: and *no* work roots, which is the sole route into `gap_status_to_impl` and
#: `gap_status_to_verification` -- the table route pushes a work-root row
#: alongside every gap it derives, so the `work_roots.is_empty()` guard above
#: those two functions never opens for it.
YAML_GAP_STATUSES = ("open", "in_progress", "blocked", "closed", "deferred")

#: gap status -> rendered `Impl` cell, under a capability that is not verified.
YAML_GAP_IMPL = {
    "open": "planned",
    "in_progress": "partial",
    "blocked": "blocked",
    "closed": "implemented",
    "deferred": "out_of_scope",
}

#: gap status -> rendered `Verification` cell, under a capability that is not
#: verified.
YAML_GAP_VERIFICATION = {
    "open": "planned",
    "in_progress": "planned",
    "blocked": "blocked",
    "closed": "passing",
    "deferred": "blocked",
}

assert set(YAML_GAP_IMPL) == set(YAML_GAP_STATUSES)
assert set(YAML_GAP_VERIFICATION) == set(YAML_GAP_STATUSES)
#: Five distinct impl cells: the impl fold is a bijection over the statuses, so
#: any two arms swapped is observable.
assert len(set(YAML_GAP_IMPL.values())) == len(YAML_GAP_STATUSES)

#: The verification fold is *not* a bijection, and its arms are ordered. These
#: two subjects pin the order, which arm-by-arm assertions cannot: under a
#: verified capability a closed gap reads `verified` rather than `passing`
#: (the capability-status arm precedes the closed arm), while a blocked gap
#: still reads `blocked` (the blocked arm precedes the capability-status arm).
YAML_VERIFIED_GAP_VERIFICATION = {"closed": "verified", "blocked": "blocked"}
assert YAML_VERIFIED_GAP_VERIFICATION["closed"] != YAML_GAP_VERIFICATION["closed"]
assert YAML_VERIFIED_GAP_VERIFICATION["blocked"] == YAML_GAP_VERIFICATION["blocked"]

YAML_GAP_CANDIDATE = ("Every Gap Status", "every-gap-status")
YAML_GAP_VERIFIED = ("Verified Capability", "verified-capability")


def _yaml_gap_capability(
    title: str, cap_id: str, status: str, gap_statuses: tuple[str, ...]
) -> str:
    gaps = "\n".join(
        f"""  - id: {cap_id}-{gap}
    status: {gap}
    summary: "{gap} gap of {cap_id}\""""
        for gap in gap_statuses
    )
    return f"""## Capability: {title}
<!-- type: capability lang: yaml -->

```yaml
id: {cap_id}
status: {status}
feature_class: core
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
gaps:
{gaps}
```

"""


YAML_GAP_DOCUMENT = (
    "# Demo\n\n## Brief\n\nYAML gap-status rendering.\n\n"
    "## Capabilities\n\n### Core Features\n\n### Non-Core Features\n\n"
    + _yaml_gap_capability(*YAML_GAP_CANDIDATE, "candidate", YAML_GAP_STATUSES)
    + _yaml_gap_capability(
        *YAML_GAP_VERIFIED, "verified", tuple(YAML_VERIFIED_GAP_VERIFICATION)
    )
)


def _rendered_work_root_rows(migrated: str, title: str) -> list[list[str]]:
    body = _capability_section_body(migrated, title)
    rows = []
    for line in body.splitlines():
        if not line.startswith("|"):
            continue
        cells = _split_escaped_row(line)
        if not cells or cells[0] in {"Work Root"} or set(cells[0]) <= {"-", ":"}:
            continue
        rows.append(cells)
    return rows


def assert_gap_status_renders_its_own_work_root_row(migrated: str) -> None:
    """A capability with gaps and no work roots renders one row per gap.

    This is the YAML reading form's own rendering path, and the table form
    cannot reach it: every gap the table route derives arrives with a work-root
    row beside it, so the `work_roots.is_empty()` guard is closed. Both folds
    below were therefore free in their entirety.

    All five statuses are declared. The impl fold is asserted as a bijection --
    five statuses, five distinct cells -- so no two arms can be swapped. The
    verification fold is not a bijection and its arms are *ordered*, so a second
    capability declares the two gaps whose rendering depends on that order: under
    a verified capability a closed gap reads `verified` and a blocked gap still
    reads `blocked`. Together those pin the blocked arm ahead of the
    capability-status arm and the capability-status arm ahead of the closed one.

    The row `Kind` is asserted as `epic`, which is what distinguishes a
    gap-derived row from the `change` rows the table route emits: without it a
    renderer that fell back to the table path would satisfy every cell above.
    """
    for (title, cap_id), status_map, verification_map in (
        (YAML_GAP_CANDIDATE, YAML_GAP_STATUSES, YAML_GAP_VERIFICATION),
        (
            YAML_GAP_VERIFIED,
            tuple(YAML_VERIFIED_GAP_VERIFICATION),
            YAML_VERIFIED_GAP_VERIFICATION,
        ),
    ):
        rows = _rendered_work_root_rows(migrated, title)
        expected = [
            [
                f"{gap} gap of {cap_id}",
                "epic",
                "-",
                YAML_GAP_IMPL[gap],
                verification_map[gap],
                "smoke",
                "-",
            ]
            for gap in status_map
        ]
        assert rows == expected, (title, rows, expected)


# --- Contract-field vocabularies -------------------------------------------
#
# A `Surfaces:` item and an `EC Dimensions:` item are both `key: value` clauses,
# and both are read by a hand-written parser rather than a serde enum. Three
# separate vocabularies decide what those parsers do, and every other document
# in this case writes only the handful of spellings its own subject needs, so
# the rest of each vocabulary was free.

#: Every spelling `is_surface_contract_key` accepts, in the order the document
#: writes them. Membership decides one thing only: whether an inline `;` opens a
#: new surface item. It is deliberately *not* the same set as the one
#: `normalize_surface_kind` folds -- the last seven here are carried by the
#: separator test and not folded by the renderer, so they reach the report
#: verbatim. Binding them together is the point: two vocabularies that look like
#: one are exactly the shape in which a spelling gets added to one and forgotten
#: in the other.
SURFACE_KEY_SPELLINGS = (
    "cli",
    "command",
    "commands",
    "http",
    "api",
    "rest",
    "sdk",
    "ui",
    "webui",
    "web",
    "config",
    "configuration",
    "fileformat",
    "file",
    "format",
    "agent",
    "agents",
    "browser",
    "browsere2e",
    "webe2e",
    "webappe2e",
    "e2e",
)

#: The kind each spelling above reaches the report as. Fifteen fold onto six
#: canonical labels; the last seven are carried verbatim. Each spelling is
#: paired to its kind through the command in the same item rather than by
#: position in a list, so a renderer that emitted the right *multiset* of kinds
#: against the wrong spellings fails.
SURFACE_KIND_BY_SPELLING = {
    "cli": "CLI",
    "command": "CLI",
    "commands": "CLI",
    "http": "HTTP",
    "api": "HTTP",
    "rest": "HTTP",
    "sdk": "SDK",
    "ui": "UI",
    "webui": "UI",
    "web": "UI",
    "config": "Config",
    "configuration": "Config",
    "fileformat": "FileFormat",
    "file": "FileFormat",
    "format": "FileFormat",
    "agent": "agent",
    "agents": "agents",
    "browser": "browser",
    "browsere2e": "browsere2e",
    "webe2e": "webe2e",
    "webappe2e": "webappe2e",
    "e2e": "e2e",
}

#: Words that look exactly like a surface key -- lowercase, followed by `: `,
#: after a `; ` -- and are not in the vocabulary. They must not open an item.
#: Without them the separator test is satisfied by an implementation that splits
#: on every `; ` it finds and never consults the vocabulary at all.
NON_SURFACE_KEY_WORDS = ("note", "nonsense", "gate", "runner")


def _contract_field_section(
    title: str,
    cap_id: str,
    surfaces: str,
    dimensions: str,
    rows: tuple[str, ...],
) -> str:
    return f"""#### {title}

ID: {cap_id}
Root WI: -
Status: confirmed
Type: Service
Feature Class: core
Required Verification: smoke
Promise:
Promise for {cap_id}.
Current State: Implemented.
Gate Inventory:
- `true`
Surfaces:
{surfaces}
EC Dimensions:
{dimensions}

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
{chr(10).join(rows)}

"""


def _spelling_walk_surface_field() -> str:
    """One `Surfaces:` line carrying every key spelling and every control word.

    They share a line rather than taking one each because the rule under test is
    what an inline `;` does, which a one-item-per-line document cannot reach.
    """
    clauses = [f"{SURFACE_KEY_SPELLINGS[0]}: `aw s0` - item 0"]
    for index, word in enumerate(
        SURFACE_KEY_SPELLINGS[1:] + NON_SURFACE_KEY_WORDS, start=1
    ):
        clauses.append(f"{word}: `aw s{index}` - item {index}")
    return "- " + "; ".join(clauses)


#: The index at which the control words begin -- the last key spelling opens the
#: item that swallows all four of them.
_TRAILING_SPELLING_INDEX = len(SURFACE_KEY_SPELLINGS) - 1

_CONTRACT_FIELD_SUBJECTS = (
    (
        "Spelling Walk",
        "spelling-walk",
        _spelling_walk_surface_field(),
        "- behavior: `true` - behaviour of spelling-walk",
        None,
    ),
    (
        "Semi Keeps",
        "semi-keeps",
        "- CLI: `aw semi-keeps` - runs a step; note: this clause stays attached",
        "- behavior: `true` - behaviour of semi-keeps",
        None,
    ),
    (
        "Dimension Split",
        "dimension-split",
        "- CLI: `aw dimension-split` - surface of dimension-split",
        "- behavior: `true` - behaviour of dimension-split;"
        " security: `true` - security of it",
        None,
    ),
    (
        "Dimension Unknown",
        "dimension-unknown",
        "- CLI: `aw dimension-unknown` - surface of dimension-unknown",
        # Declared `security`, not `behavior`, and that is the whole point of
        # the subject: `dedupe_ec_dimensions` merges items by kind and keeps the
        # first non-empty field of each, so an unknown kind falling back to
        # `Behavior` beside a declared `behavior` item is absorbed into it and
        # leaves the report byte-identical to the drop. Against a declared
        # `security` the fallback surfaces as a second item.
        "- security: `true` - security of dimension-unknown\n"
        "- nonsense: `true` - dropped?",
        None,
    ),
    (
        "Bad Cells",
        "bad-cells",
        "- CLI: `aw bad-cells` - surface of bad-cells",
        "- behavior: `true` - behaviour of bad-cells",
        (
            "| Bad kind of bad-cells | bug | - | implemented | verified"
            " | smoke | `true` |",
            "| Bad impl of bad-cells | change | - | mystery | verified"
            " | smoke | `true` |",
            "| Bad verif of bad-cells | change | - | implemented | unknown"
            " | smoke | `true` |",
            "| Bad maturity of bad-cells | change | - | implemented | verified"
            " | fuzzy | `true` |",
        ),
    ),
)


def _contract_field_document() -> str:
    index = "".join(
        f"| {title} | - | implemented | planned | smoke | not_ready |"
        f" Promise for {cap_id}. |\n"
        for title, cap_id, *_ in _CONTRACT_FIELD_SUBJECTS
    )
    sections = "".join(
        _contract_field_section(
            title,
            cap_id,
            surfaces,
            dimensions,
            rows or (f"| {title} root | change | - | implemented | verified"
                     " | smoke | `true` |",),
        )
        for title, cap_id, surfaces, dimensions, rows in _CONTRACT_FIELD_SUBJECTS
    )
    return f"""# Demo

## Brief

Machine-readable capability contract for Demo.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
{index}
### Core Features

{sections}### Non-Core Features
"""


#: Reported, not migrated. Migration lifts a backticked token out of a surface
#: item's summary into that item's command list without removing it from the
#: summary, so the trailing item below comes back reporting nine commands where
#: five were declared. It settles there -- the duplication is one-shot, not
#: unbounded -- but asserting the post-migration command list would hold the
#: defect in place, so this document is asserted on the reading route instead.
#: Filed as #3273.
CONTRACT_FIELD_DOCUMENT = _contract_field_document()


def _capability_by_id(report: dict[str, Any], cap_id: str) -> dict[str, Any]:
    for capability in report.get("capabilities") or []:
        if capability.get("id") == cap_id:
            return capability
    raise AssertionError((cap_id, report))


def assert_surface_keys_are_two_independent_vocabularies(
    report: dict[str, Any],
) -> None:
    """Every key spelling opens its own item; no other word does.

    One line declares twenty-two key spellings and four look-alikes. Each clause
    carries a distinct command, so the spelling that opened an item is
    recoverable from the item itself and the spelling-to-kind pairing is bound
    rather than just the multiset of kinds -- the failure mode where a renderer
    emits the right labels against the wrong keys.

    The two vocabularies are asserted to disagree. The last seven spellings open
    an item and are then rendered verbatim, because the separator test carries
    them and the kind fold does not. Asserting the labels alone would be
    satisfied by one vocabulary doing both jobs, which is the shape in which a
    spelling gets added to one and forgotten in the other.

    The four control words are the other half: they sit in the trailing item's
    summary, and their backticked commands are harvested into that item. That
    makes the negative observable -- an implementation that split on every `; `
    would produce twenty-six items, and one that consulted no vocabulary at all
    would produce one.
    """
    surfaces = _capability_by_id(report, "spelling-walk")["surfaces"]
    assert len(surfaces) == len(SURFACE_KEY_SPELLINGS), surfaces

    by_command = {}
    for item in surfaces:
        commands = item["commands"]
        assert commands, item
        by_command[commands[0]] = item
    assert len(by_command) == len(surfaces), surfaces

    for index, spelling in enumerate(SURFACE_KEY_SPELLINGS):
        item = by_command.get(f"aw s{index}")
        assert item is not None, (spelling, index, surfaces)
        assert item["kind"] == SURFACE_KIND_BY_SPELLING[spelling], (spelling, item)
        if index < _TRAILING_SPELLING_INDEX:
            assert item["commands"] == [f"aw s{index}"], item
            assert item["summary"] == f"item {index}", item

    # The fold is many-to-one and the carry-through is not, and both have to be
    # visible. Fifteen spellings collapse onto six labels; seven survive as
    # themselves. A renderer that folded all twenty-two, or none, fails here.
    folded = {
        spelling
        for spelling in SURFACE_KEY_SPELLINGS
        if SURFACE_KIND_BY_SPELLING[spelling] != spelling
    }
    assert len(folded) == 15, sorted(folded)
    assert len({SURFACE_KIND_BY_SPELLING[s] for s in folded}) == 6, sorted(folded)
    carried = set(SURFACE_KEY_SPELLINGS) - folded
    assert carried == {
        "agent",
        "agents",
        "browser",
        "browsere2e",
        "webe2e",
        "webappe2e",
        "e2e",
    }, sorted(carried)

    trailing = by_command[f"aw s{_TRAILING_SPELLING_INDEX}"]
    expected_commands = [f"aw s{_TRAILING_SPELLING_INDEX}"] + [
        f"aw s{_TRAILING_SPELLING_INDEX + offset}"
        for offset in range(1, len(NON_SURFACE_KEY_WORDS) + 1)
    ]
    assert trailing["commands"] == expected_commands, trailing
    expected_summary = f"item {_TRAILING_SPELLING_INDEX}"
    for offset, word in enumerate(NON_SURFACE_KEY_WORDS, start=1):
        index = _TRAILING_SPELLING_INDEX + offset
        expected_summary += f"; {word}: `aw s{index}` - item {index}"
    assert trailing["summary"] == expected_summary, trailing


def assert_a_semicolon_without_a_key_stays_in_the_summary(
    report: dict[str, Any],
) -> None:
    """The negative in isolation, on a summary that reads like prose.

    The spelling walk above proves the vocabulary is consulted, but its control
    words sit at the tail of a twenty-six-clause line where a truncation would
    look the same. This capability declares one item whose summary contains a
    single `; note: ` and nothing else, so the clause's survival is the whole
    observation.
    """
    surfaces = _capability_by_id(report, "semi-keeps")["surfaces"]
    assert surfaces == [
        {
            "kind": "CLI",
            "commands": ["aw semi-keeps"],
            "summary": "runs a step; note: this clause stays attached",
        }
    ], surfaces


def assert_an_inline_semicolon_splits_ec_dimensions(report: dict[str, Any]) -> None:
    """EC dimensions split on `;` too, and the split is not the surface rule.

    Every other document here writes one dimension per line, which leaves the
    inline separator unreached. The two dimensions are declared on one line and
    must arrive as two, each keeping its own summary -- a splitter that dropped
    the tail, or one that kept the line whole, fails.
    """
    dimensions = _capability_by_id(report, "dimension-split")["ec_dimensions"]
    assert dimensions == [
        {
            "dimension": "behavior",
            "runner": "true",
            "summary": "behaviour of dimension-split",
            "required_for_production": True,
        },
        {
            "dimension": "security",
            "runner": "true",
            "summary": "security of it",
            "required_for_production": True,
        },
    ], dimensions


def assert_an_unrecognized_ec_dimension_is_dropped(report: dict[str, Any]) -> None:
    """A dimension whose kind is outside the enum does not reach the report.

    The drop is silent: no blocker, no diagnostic, and the capability still
    reports as if the author had written one dimension. That is asserted here
    because it was unbound, not because it is obviously right -- an author who
    misspells a dimension kind gets a contract quietly narrower than the one
    they wrote. Filed as a defect rather than pinned as intent.

    The declared dimension is `security` rather than `behavior` so that the drop
    is distinguishable from the near miss beside it. `parse_ec_dimension_kind`
    returning `Behavior` for an unrecognized kind, instead of `None`, produces a
    byte-identical report when a `behavior` item is already declared, because
    `dedupe_ec_dimensions` merges by kind and keeps the first non-empty field of
    each. Against a declared `security` the fallback surfaces as a second item.

    Migration is not asserted on this document. Measured separately, `capability
    migrate` re-renders the dropped line into `CAPABILITIES.md`, so the document
    keeps a clause no consumer reads -- the same shape as the generated
    Efficiency section of #3272. Binding that here would hold it in place.

    Filed as #3274.
    """
    dimensions = _capability_by_id(report, "dimension-unknown")["ec_dimensions"]
    assert dimensions == [
        {
            "dimension": "security",
            "runner": "true",
            "summary": "security of dimension-unknown",
            "required_for_production": True,
        },
        # Synthesized, not declared, and carrying no runner: a capability that
        # declares no behavior dimension is given one anyway. This is the only
        # capability in the case that declares dimensions without declaring
        # `behavior`, so it is the only input on which the synthesis is
        # observable at all -- everywhere else the declared item occupies the
        # slot and the fallback is indistinguishable from doing nothing.
        {
            "dimension": "behavior",
            "summary": "declared by behavior surfaces or verification contract",
            "required_for_production": True,
        },
    ], dimensions


#: One blocker per out-of-vocabulary cell, in the order the rows are written.
#: The messages are pinned whole because each names the vocabulary it rejected
#: against, and those four enumerations are the assertion's content -- a message
#: pinned only by its prefix leaves the expected-value list free.
BAD_CELL_BLOCKERS = (
    "capability `Bad Cells` work root `Bad kind of bad-cells` has invalid Kind"
    " `bug`; expected epic, subepic, or change — fix the Kind cell"
    " (`aw capability check --project <project>` for remediation guidance)",
    "capability `Bad Cells` work root `Bad impl of bad-cells` has invalid Impl"
    " `mystery`; expected planned, partial, implemented, blocked, or out_of_scope"
    " — fix the Impl cell"
    " (`aw capability check --project <project>` for remediation guidance)",
    "capability `Bad Cells` work root `Bad verif of bad-cells` has invalid"
    " Verification `unknown`; expected none, planned, failing, passing, verified,"
    " or blocked — fix the Verification cell"
    " (`aw capability check --project <project>` for remediation guidance)",
    "capability `Bad Cells` work root `Bad maturity of bad-cells` has invalid"
    " Maturity `fuzzy`; expected none, smoke, conformance, corpus, negative, or"
    " dogfood — fix the Maturity cell"
    " (`aw capability check --project <project>` for remediation guidance)",
)


def assert_each_out_of_vocabulary_cell_raises_its_own_blocker(
    report: dict[str, Any],
) -> None:
    """Four bad cells in four columns raise four blockers, in document order.

    Every work-root cell every other document here writes is in vocabulary, so
    all four column validators ran only on inputs that could not fail them. Each
    row breaks exactly one column, which is what attributes a blocker to a
    validator: a single row breaking all four would be satisfied by one validator
    firing four times.

    The capability still parses and still reports. A reader that abandoned the
    section on the first bad cell would raise one blocker and drop the
    capability, and the count assertion below is what separates that from
    validating every row.
    """
    blockers = [b for b in report.get("blockers") or [] if "`Bad Cells`" in b]
    assert tuple(blockers) == BAD_CELL_BLOCKERS, blockers
    # Not `>= 4`: the fixture's own environment contributes unrelated blockers
    # (no Python EC or TD manifest), and this leg owns only the ones it wrote.
    assert report["capability_count"] == len(_CONTRACT_FIELD_SUBJECTS), report
    assert report["status"] == "blocked", report
    bad_cells = _capability_by_id(report, "bad-cells")
    assert bad_cells["surfaces"] == [
        {
            "kind": "CLI",
            "commands": ["aw bad-cells"],
            "summary": "surface of bad-cells",
        }
    ], bad_cells


# --- Machine tables and the efficiency contract fields ----------------------
#
# A capability may declare its surfaces and EC dimensions as a Markdown *table*
# instead of as contract fields. Every other document in this case uses the
# field form, so `parse_markdown_surface_table` and
# `parse_markdown_ec_dimension_table` were entirely undriven -- both alias sets,
# both defaults, and the row-drop guard.

#: A capability whose `Status` is `confirmed` is required to declare at least
#: one surface and at least one EC dimension. Each subject below declares its
#: own kind as a table and the other as a contract field, so the requirement is
#: met without a second table competing for the same parser.
_MACHINE_TABLE_SUBJECTS = (
    (
        "Table Surfaces",
        "table-surfaces",
        """| Surface | Commands | Owns | Verification |
|---|---|---|---|
| CLI | `aw alpha` | owns the alpha path | `aw alpha --check` |
|  | `aw gamma` | kind column empty, defaults | - |""",
        "EC Dimensions:\n- behavior: `true` - behaviour of table-surfaces\n",
    ),
    (
        "Alias Surfaces",
        "alias-surfaces",
        """| Kind | CLI | Purpose | Gate |
|---|---|---|---|
| HTTP | `aw beta` | purpose alias | `aw beta --gate` |""",
        "EC Dimensions:\n- behavior: `true` - behaviour of alias-surfaces\n",
    ),
    (
        "Two Columns",
        "two-columns",
        """| Surface | Commands |
|---|---|
| CLI | `aw kept` |
| HTTP | - |
| SDK |  |""",
        "EC Dimensions:\n- behavior: `true` - behaviour of two-columns\n",
    ),
    (
        "Three Columns",
        "three-columns",
        """| Surface | Commands | Summary |
|---|---|---|
| CLI | `aw kept` | kept |
| HTTP | - |  |
| SDK |  |  |""",
        "EC Dimensions:\n- behavior: `true` - behaviour of three-columns\n",
    ),
    (
        "Dimension Table",
        "dimension-table",
        # The out-of-enum row is declared *first* on purpose. `dedupe_ec_dimensions`
        # merges by kind and keeps the first non-empty field, so an unknown kind
        # falling back to `Behavior` behind a declared `behavior` row is absorbed
        # and reads identically to being dropped -- the mutant that keeps the row
        # survived in exactly that arrangement. Declared first, the fallback would
        # win the summary, so the drop is observable.
        """| Dimension | Runner | Summary |
|---|---|---|
| nonsense | `aw nonsense` | dropped, not folded into the row below |
| behavior | `true` | behaviour by table |""",
        "Surfaces:\n- CLI: `aw dimension-table` - surface of dimension-table\n",
    ),
    (
        "Dimension Aliases",
        "dimension-aliases",
        """| Category | Tool | Evidence |
|---|---|---|
| security | `aw sec` | security by alias |""",
        "Surfaces:\n- CLI: `aw dimension-aliases` - surface of dimension-aliases\n",
    ),
)


def _machine_table_section(title: str, cap_id: str, table: str, field: str) -> str:
    return f"""#### {title}

ID: {cap_id}
Root WI: -
Status: confirmed
Type: Service
Feature Class: core
Required Verification: smoke
Promise:
Promise for {cap_id}.
Current State: Implemented.
Gate Inventory:
- `true`
{field}
{table}

| Work Root | Kind | WI | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---:|---|---|---|---|
| {title} root | change | - | implemented | verified | smoke | `true` |

"""


def _machine_table_document() -> str:
    index = "".join(
        f"| {title} | - | implemented | planned | smoke | not_ready |"
        f" Promise for {cap_id}. |\n"
        for title, cap_id, *_ in _MACHINE_TABLE_SUBJECTS
    )
    sections = "".join(
        _machine_table_section(*subject) for subject in _MACHINE_TABLE_SUBJECTS
    )
    return f"""# Demo

## Brief

Machine-readable capability contract for Demo.

## Capabilities

### Capability Index

| Capability | Root WI | Impl | Verification | Maturity | Production | Notes |
|---|---:|---|---|---|---|---|
{index}
### Core Features

{sections}### Non-Core Features
"""


MACHINE_TABLE_DOCUMENT = _machine_table_document()

#: What each table-declared capability reports. Written out per subject rather
#: than derived, because the derivation would be the implementation.
MACHINE_TABLE_SURFACES = {
    "table-surfaces": [
        {
            "kind": "CLI",
            "commands": ["aw alpha"],
            "summary": "owns the alpha path",
            "verification": "`aw alpha --check`",
        },
        # The kind cell is blank and defaults to `CLI` -- the one default in this
        # parser that no other input can reach, because a table that declares a
        # kind column and fills it renders the identical item whatever the
        # default is.
        {
            "kind": "CLI",
            "commands": ["aw gamma"],
            "summary": "kind column empty, defaults",
            "verification": "-",
        },
    ],
    "alias-surfaces": [
        {
            "kind": "HTTP",
            "commands": ["aw beta"],
            "summary": "purpose alias",
            "verification": "`aw beta --gate`",
        }
    ],
    # No summary and no verification column, so the drop guard can fire: the two
    # rows with no command are gone.
    "two-columns": [{"kind": "CLI", "commands": ["aw kept"]}],
    # The identical rows plus a third column, and now nothing drops. `table_cell`
    # returns the literal `-` for a blank cell, so `summary.trim().is_empty()` is
    # false and the guard cannot fire. The two capabilities differ only in
    # whether the table has a third column, which is what makes the guard's
    # column-shape dependence observable at all.
    "three-columns": [
        {"kind": "CLI", "commands": ["aw kept"], "summary": "kept"},
        {"kind": "HTTP", "summary": "-"},
        {"kind": "SDK", "summary": "-"},
    ],
}

MACHINE_TABLE_DIMENSIONS = {
    # The table route stores the runner cell verbatim; the contract-field route
    # strips the backticks. Both spellings reach the report, and the divergence
    # is why a table-declared runner renders double-backticked on migration.
    "dimension-table": [
        {
            "dimension": "behavior",
            "runner": "`true`",
            "summary": "behaviour by table",
            "required_for_production": True,
        }
    ],
    "dimension-aliases": [
        {
            "dimension": "security",
            "runner": "`aw sec`",
            "summary": "security by alias",
            "required_for_production": True,
        },
        {
            "dimension": "behavior",
            "summary": "declared by behavior surfaces or verification contract",
            "required_for_production": True,
        },
    ],
}


def assert_machine_tables_declare_surfaces_and_dimensions(
    report: dict[str, Any],
) -> None:
    """Both table parsers, both alias sets, both defaults, and the drop guard.

    The four surface subjects cover the column vocabulary in two halves --
    `Surface`/`Commands`/`Owns`/`Verification` and the aliases
    `Kind`/`CLI`/`Purpose`/`Gate` -- so no single alias carries a column on its
    own. The blank kind cell reaches the `CLI` default, which a filled column
    cannot.

    The drop guard is the sharp one. It fires only when the command cell is
    empty *and* the summary and verification are blank, and `table_cell` returns
    the literal `-` for a missing or blank cell, so a table that declares a third
    column can never satisfy it. `Two Columns` and `Three Columns` carry the same
    rows and disagree about which survive, which is not expressible in one
    document.

    The dimension table's runner is asserted with its backticks intact, against
    the field route's stripped spelling elsewhere in this case. The two routes
    genuinely disagree, and asserting one of them alone would leave a reader --
    or a refactor -- free to unify them in either direction.

    A row whose dimension kind is outside the enum is dropped here too, and the
    `Dimension Table` subject declares one -- ahead of its in-vocabulary row, so
    that a fallback to `Behavior` would win the merged summary rather than being
    absorbed behind it. That is the same silent drop as the contract-field route
    (#3274), reached through the other parser.
    """
    for cap_id, expected in MACHINE_TABLE_SURFACES.items():
        surfaces = _capability_by_id(report, cap_id)["surfaces"]
        assert surfaces == expected, (cap_id, surfaces, expected)
    for cap_id, expected in MACHINE_TABLE_DIMENSIONS.items():
        dimensions = _capability_by_id(report, cap_id)["ec_dimensions"]
        assert dimensions == expected, (cap_id, dimensions, expected)
    assert report["capability_count"] == len(_MACHINE_TABLE_SUBJECTS), report


def assert_migration_leaves_a_machine_table_alone(migrated: str) -> None:
    """Migration does not convert a machine table into contract fields.

    The table is re-emitted verbatim and no `Surfaces:` field appears beside it,
    so the surfaces are re-parsed from the table on every read. That is stable
    and lossless -- it is asserted here because it was unbound, and because the
    obvious alternative, rendering the parsed items into a `Surfaces:` field,
    would silently drop each item's `Verification` cell: `render_surface_field_items`
    has nowhere to put it.
    """
    body = _capability_section_body(migrated, "Table Surfaces")
    assert "Surfaces:" not in body, body
    for line in _MACHINE_TABLE_SUBJECTS[0][2].splitlines():
        assert line in body, (line, body)


# --- The efficiency contract fields ----------------------------------------

_EFFICIENCY_SUBJECTS = (
    ("Both Halves", "both-halves", "3M docs", "apps/demo/both.json", "efficiency"),
    ("Point Only", "point-only", "9M docs", None, "behavior"),
    ("Cube Only", "cube-only", None, "apps/demo/cube.json", "behavior"),
)


def _efficiency_section(title, cap_id, point, cube, dimension):
    fields = ""
    if point is not None:
        fields += f"Efficiency Operating Point: {point}\n"
    if cube is not None:
        fields += f"Efficiency Cube: {cube}\n"
    runner = "`aw bench`" if dimension == "efficiency" else "`true`"
    return f"""### {title}

ID: {cap_id}
Root WI: -
Status: implemented
Type: Service
Feature Class: core
Promise: Promise for {cap_id}.
Current State: Partially implemented.
Required Verification: smoke
{fields}Surfaces:
- CLI: `aw {cap_id}` - surface of {cap_id}
EC Dimensions:
- {dimension}: {runner} - {dimension} of {cap_id}

| Work Root | Kind | Impl | Verification | Maturity | Gate / Evidence |
|---|---|---|---|---|---|
| {title} root | change | implemented | verified | smoke | `true` |

"""


EFFICIENCY_DOCUMENT = (
    "# Demo\n\n## Brief\n\nEfficiency contract fields.\n\n## Capabilities\n\n"
    + "".join(_efficiency_section(*s) for s in _EFFICIENCY_SUBJECTS)
)

#: The generated section's heading, asserted whole: it carries the instruction
#: not to hand-edit, which is the only thing telling an author the block is
#: machine-owned.
EFFICIENCY_SECTION_HEADING = (
    "#### Efficiency - GENERATED (backfilled by `aw ec`; do not hand-edit)"
)


def assert_efficiency_fields_render_their_generated_section(migrated: str) -> None:
    """Both halves, each half alone, and the two spellings of the missing half.

    `Efficiency Operating Point:` and `Efficiency Cube:` are contract fields no
    other document here writes, so the slot parser, the generated section, and
    the merge that attaches the slot to a dimension were all undriven.

    Three capabilities are needed rather than one. A capability declaring both
    halves cannot show what a missing half renders as, and the two missing-half
    spellings are separate literals -- `Cube: -` and `Operating point: -` -- that
    a renderer could easily produce for only one of them. Each half is declared
    alone by exactly one capability.

    The merge is exercised in both directions and **claimed in only one**, which
    is a correction to what this docstring said when the leg was written. `Both
    Halves` declares an `efficiency` dimension of its own and the slot attaches to
    it; the other two declare `behavior` only and the merge pushes a generated
    `efficiency` dimension carrying the product's marker summary. The push arm is
    observable. The attach arm is not: a merge that *always* pushed leaves `Both
    Halves` with two efficiency dimensions only until `dedupe_ec_dimensions` runs,
    and that merge keeps the first non-empty field of each -- so the authored
    runner and summary win and the report is byte-identical. Measured, not
    reasoned: the mutant survives. This is the same redundancy the production
    document's own label already records for this pair, arrived at from the other
    side.

    What is *not* asserted here is the round trip. The generated section is
    emitted after the capability's work-root table, so on re-read it lies outside
    the block `find_efficiency_backfill_section_span` is given and is never seen
    again -- the slot is text-only from tick 1 onward, and
    `validate_efficiency_backfill_slots` is unreachable on any canonical
    document. Filed as #3272. Pinning the render is safe because fixing the round
    trip does not change the bytes migration emits.
    """
    # Read positionally rather than through `_capability_section_body`, which
    # cannot see these blocks at all: the generated section is emitted after the
    # capability's work-root table, so its own `####` heading closes the block it
    # belongs to. That is the whole of #3272, and it is why the section is read
    # here as a sequence of blocks in document order instead of as a field of the
    # capability that produced it.
    lines = migrated.splitlines()
    blocks = []
    for index, line in enumerate(lines):
        if line.strip() != EFFICIENCY_SECTION_HEADING:
            continue
        values = {}
        for follower in lines[index + 1 :]:
            if follower.startswith("#"):
                break
            if ":" in follower:
                key, _, value = follower.partition(":")
                values[key.strip()] = value.strip()
        blocks.append(values)
    expected = [
        {
            "Operating point": point if point is not None else "-",
            "Cube": cube if cube is not None else "-",
        }
        for _title, _cap_id, point, cube, _dimension in _EFFICIENCY_SUBJECTS
    ]
    assert blocks == expected, (blocks, expected)


