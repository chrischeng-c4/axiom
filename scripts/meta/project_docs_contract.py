#!/usr/bin/env python3
"""Validate one project product-document set.

The README validator remains the source of truth for the product-front-door
contract. This script adds deterministic STATUS and ROADMAP checks plus the
cross-document rules. It also validates conventional protocol, generated-client,
indexing, querying, GKE, client-integration, and linked migration guides when a
project adopts them. The ``prompt`` command emits one clean-reader task bound to
the exact bytes of the complete adopted set.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
from dataclasses import dataclass, field
from pathlib import Path


SCRIPT_DIR = Path(__file__).resolve().parent
if str(SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPT_DIR))

import readme_contract  # noqa: E402


STATUS_H2 = ("Scope", "State definitions", "Support matrix", "Evidence policy")
STATUS_DEFINITION_HEADER = ("State", "Meaning")
STATUS_HEADER = ("Surface", "ID", "State", "Supported scope", "Limits", "Evidence")
STATUS_STATES = ("Supported", "Limited", "Not supported")
ROADMAP_H2 = ("Purpose", "Near-term outcomes", "Later outcomes", "Non-goals")
OUTCOME_FIELDS = ("ID", "Outcome", "Boundary", "Completion evidence", "Tracking")
NON_GOAL_FIELDS = ("ID", "Reason")
DOC_NAMES = ("README.md", "STATUS.md", "ROADMAP.md")
PROTOCOL_H2 = (
    "Purpose",
    "Contract map",
    "Use the protocol",
    "Current boundaries",
    "Supporting documents",
)
CLIENTS_H2 = (
    "Contract",
    "Generate",
    "Language matrix",
    "Connect",
    "Current boundaries",
    "Verification",
    "Supporting documents",
)
INDEXING_H2 = (
    "Purpose",
    "Contract map",
    "Data ownership",
    "Schema contract",
    "Write contract",
    "Durability",
    "Rebuild and activation",
    "Current boundaries",
    "Supporting documents",
)
QUERYING_H2 = (
    "Purpose",
    "Data ownership",
    "Contract map",
    "Search model",
    "Result controls",
    "Facets and metrics",
    "Limits and failures",
    "Compatibility and migration",
    "Current boundaries",
    "Supporting documents",
)
GKE_H2 = (
    "Purpose",
    "Contract map",
    "Support tiers",
    "Runtime size and topology",
    "Kubernetes-native contract",
    "GKE Standard Regional profile",
    "Storage, placement, and disruption",
    "Identity and networking",
    "Verification",
    "Current boundaries",
    "Supporting documents",
)
CLIENT_INTEGRATION_H2 = (
    "Purpose",
    "Contract map",
    "Responsibility boundary",
    "Connection profiles",
    "Generated client behavior",
    "Kubernetes workload template",
    "Source integration",
    "Failure handling",
    "Verification",
    "Current boundaries",
    "Supporting documents",
)
MIGRATION_H2 = (
    "Purpose",
    "Compatibility window",
    "Schema migration",
    "Request migration",
    "Response migration",
    "Managed activation",
    "Migration tools",
    "Verification",
    "Supporting documents",
)
PROTOCOL_TABLE_HEADER = ("Fact", "Canonical source", "Discovery")
CONTRACT_MAP_TABLE_HEADER = PROTOCOL_TABLE_HEADER
CLIENTS_TABLE_HEADER = (
    "Language",
    "Generated form",
    "Transport",
    "Auth input",
    "Current limits",
)
MIGRATION_TABLE_HEADER = ("Surface", "0.4.x", "0.5.0", "Required action")
SUPPORTING_DOC_SPECS = {
    "docs/protocol.md": ("protocol", PROTOCOL_H2, "Contract map", PROTOCOL_TABLE_HEADER),
    "clients/README.md": ("clients", CLIENTS_H2, "Language matrix", CLIENTS_TABLE_HEADER),
    "docs/indexing.md": ("indexing", INDEXING_H2, "Contract map", CONTRACT_MAP_TABLE_HEADER),
    "docs/querying.md": ("querying", QUERYING_H2, "Contract map", CONTRACT_MAP_TABLE_HEADER),
    "docs/gke.md": ("gke", GKE_H2, "Contract map", CONTRACT_MAP_TABLE_HEADER),
    "docs/client-integration.md": (
        "client-integration",
        CLIENT_INTEGRATION_H2,
        "Contract map",
        CONTRACT_MAP_TABLE_HEADER,
    ),
}
MIGRATION_DOC_SPEC = (
    "migration",
    MIGRATION_H2,
    "Compatibility window",
    MIGRATION_TABLE_HEADER,
)
FIELD_RE = re.compile(r"^- ([A-Za-z][A-Za-z ]*):\s*(.*)$")
BANNED_META_FIELD_RE = re.compile(
    r"^\s*(?:[-*]\s+)?(?:Status|Maturity|Production|Progress|Percent complete|"
    r"Owner|Due|Target date|ETA)\s*:",
    re.MULTILINE | re.IGNORECASE,
)
PERCENT_PROGRESS_RE = re.compile(
    r"(?:\b\d{1,3}\s*%\s*(?:complete|done)|(?:progress|complete)\s*[:=]?\s*\d{1,3}\s*%)",
    re.IGNORECASE,
)

# These are exact positive claims that would reverse Lumen's maintained
# current/target boundary. Keep this list narrow. Correct statements such as
# "Fleet is not HA" or "generated clients do not yet rotate tokens" must not
# match. The semantic clean reader remains responsible for paraphrases.
LUMEN_FORBIDDEN_ASSERTIONS = (
    re.compile(r"\bFleet (?:is|equals|enables|provides) (?:HA|autoscaling)\b", re.IGNORECASE),
    re.compile(r"\b(?:one|1) Pod (?:is|provides) HA\b", re.IGNORECASE),
    re.compile(r"\b2 voters? (?:is|are|provides?) production HA\b", re.IGNORECASE),
    re.compile(r"\bZonal acceptance proves regional HA\b", re.IGNORECASE),
    re.compile(r"\bPDB (?:limits|gates|protects) StatefulSet rolling updates?\b", re.IGNORECASE),
    re.compile(r"\bGCE machine types? belong(?:s)? in the Kubernetes-native CRD\b", re.IGNORECASE),
    re.compile(
        r"\bOperator creates? (?:the )?(?:Kubernetes cluster|namespace|KSA|client Deployment)\b",
        re.IGNORECASE,
    ),
    re.compile(r"\bBinding a KSA automatically adds? (?:an )?Authorization header\b", re.IGNORECASE),
    re.compile(r"\bGenerated clients support token rotation and safe retry today\b", re.IGNORECASE),
    re.compile(r"\bClient-side validation replaces TokenReview\b", re.IGNORECASE),
    re.compile(r"\bThe current runtime Kustomize template is Managed\b", re.IGNORECASE),
    re.compile(r"\bLumen (?:executes|runs|generates) embedding models?\b", re.IGNORECASE),
    re.compile(r"\bGenerated SDK packages? (?:are|is) published\b", re.IGNORECASE),
)


@dataclass(frozen=True)
class DocFinding:
    rule: str
    line: int
    message: str

    def as_dict(self) -> dict[str, object]:
        return {"rule": self.rule, "line": self.line, "message": self.message}


@dataclass
class StatusSurface:
    name: str
    surface_id: str
    state: str
    scope: str
    limits: str
    evidence: str
    gates: list[str] = field(default_factory=list)
    roadmap_id: str | None = None

    def as_dict(self) -> dict[str, object]:
        return {
            "name": self.name,
            "id": self.surface_id,
            "state": self.state,
            "scope": self.scope,
            "limits": self.limits,
            "evidence": self.evidence,
            "gates": self.gates,
            "roadmap_id": self.roadmap_id,
        }


@dataclass
class RoadmapItem:
    name: str
    item_id: str
    horizon: str
    fields: dict[str, str]

    def as_dict(self) -> dict[str, object]:
        return {
            "name": self.name,
            "id": self.item_id,
            "horizon": self.horizon,
            "fields": self.fields,
        }


@dataclass
class StatusReport:
    path: str
    sha256: str
    surfaces: list[StatusSurface]
    warnings: list[str]
    findings: list[DocFinding]

    @property
    def ok(self) -> bool:
        return not self.findings

    def as_dict(self) -> dict[str, object]:
        return {
            "path": self.path,
            "sha256": self.sha256,
            "ok": self.ok,
            "surfaces": [surface.as_dict() for surface in self.surfaces],
            "warnings": self.warnings,
            "findings": [finding.as_dict() for finding in self.findings],
        }


@dataclass
class RoadmapReport:
    path: str
    sha256: str
    items: list[RoadmapItem]
    findings: list[DocFinding]

    @property
    def ok(self) -> bool:
        return not self.findings

    def as_dict(self) -> dict[str, object]:
        return {
            "path": self.path,
            "sha256": self.sha256,
            "ok": self.ok,
            "items": [item.as_dict() for item in self.items],
            "findings": [finding.as_dict() for finding in self.findings],
        }


@dataclass
class SupportingDocReport:
    kind: str
    path: str
    sha256: str
    findings: list[DocFinding]

    @property
    def ok(self) -> bool:
        return not self.findings

    def as_dict(self) -> dict[str, object]:
        return {
            "kind": self.kind,
            "path": self.path,
            "sha256": self.sha256,
            "ok": self.ok,
            "findings": [finding.as_dict() for finding in self.findings],
        }


@dataclass
class ProjectReport:
    path: str
    readme: readme_contract.Report | None
    status: StatusReport | None
    roadmap: RoadmapReport | None
    supporting_docs: list[SupportingDocReport]
    findings: list[DocFinding]

    @property
    def ok(self) -> bool:
        return (
            not self.findings
            and self.readme is not None
            and self.readme.ok
            and self.status is not None
            and self.status.ok
            and self.roadmap is not None
            and self.roadmap.ok
            and all(document.ok for document in self.supporting_docs)
        )

    def as_dict(self) -> dict[str, object]:
        return {
            "path": self.path,
            "ok": self.ok,
            "readme": self.readme.as_dict() if self.readme else None,
            "status": self.status.as_dict() if self.status else None,
            "roadmap": self.roadmap.as_dict() if self.roadmap else None,
            "supporting_docs": [document.as_dict() for document in self.supporting_docs],
            "findings": [finding.as_dict() for finding in self.findings],
        }


def resolve_project(repo: Path, raw: str) -> Path:
    repo = repo.resolve()
    path = Path(raw)
    if not path.is_absolute():
        path = repo / path
    path = path.resolve()
    if path.is_file():
        if (
            path.parent.name == "docs"
            and (
                path.name
                in {
                    "protocol.md",
                    "indexing.md",
                    "querying.md",
                    "gke.md",
                    "client-integration.md",
                }
                or is_migration_guide(path.name)
            )
        ):
            path = path.parent.parent
        elif path.name == "README.md" and path.parent.name == "clients":
            path = path.parent.parent
        elif path.name in DOC_NAMES:
            path = path.parent
        else:
            readme_contract.usage_error(
                "target file must be a core project document, a conventional "
                f"supporting guide, or clients/README.md: {path}"
            )
    try:
        path.relative_to(repo)
    except ValueError:
        readme_contract.usage_error(f"target is outside the checkout: {raw}")
    if not path.is_dir():
        readme_contract.usage_error(f"project directory does not exist: {path}")
    return path


def is_migration_guide(name: str) -> bool:
    return name.startswith("migration-") and name.endswith(".md")


def headings(lines: list[str], level: int) -> list[tuple[int, str]]:
    prefix = "#" * level + " "
    return [
        (index, line[len(prefix) :].strip())
        for index, line in enumerate(lines)
        if line.startswith(prefix)
    ]


def required_heading_positions(
    lines: list[str], required: tuple[str, ...], doc: str, rule: str
) -> tuple[dict[str, int], list[DocFinding]]:
    h2 = headings(lines, 2)
    positions: dict[str, list[int]] = {}
    for index, name in h2:
        positions.setdefault(name, []).append(index)
    findings: list[DocFinding] = []
    for name in required:
        count = len(positions.get(name, []))
        if count != 1:
            findings.append(
                DocFinding(rule, 1, f"{doc} heading '## {name}' appears {count} times")
            )
    resolved = {name: found[0] for name, found in positions.items() if len(found) == 1}
    if all(name in resolved for name in required):
        actual = [name for _index, name in h2]
        if actual != list(required):
            findings.append(
                DocFinding(
                    rule,
                    1,
                    f"{doc} H2 headings must be exactly: " + " | ".join(required),
                )
            )
    return resolved, findings


def table_after(
    lines: list[str], heading_at: int, heading_level: int = 2
) -> list[tuple[int, list[str]]]:
    end = readme_contract.section_end(lines, heading_at, heading_level)
    rows: list[tuple[int, list[str]]] = []
    started = False
    for index in range(heading_at + 1, end):
        if lines[index].lstrip().startswith("|"):
            rows.append((index, readme_contract.parse_table_row(lines[index])))
            started = True
        elif started and lines[index].strip():
            break
    return rows


def strip_inline(text: str) -> str:
    text = re.sub(r"\[([^\]]+)\]\([^)]+\)", r"\1", text)
    return re.sub(r"`([^`]+)`", r"\1", text)


def github_slug(text: str) -> str:
    value = strip_inline(text).lower().strip()
    value = re.sub(r"[^a-z0-9 _-]", "", value)
    value = re.sub(r"[ _]+", "-", value)
    return re.sub(r"-+", "-", value).strip("-")


def markdown_links(lines: list[str]) -> list[tuple[int, str]]:
    links: list[tuple[int, str]] = []
    in_fence = False
    fence = ""
    for index, line in enumerate(lines):
        marker = line.lstrip()[:3]
        if marker in {"```", "~~~"}:
            if not in_fence:
                in_fence = True
                fence = marker
            elif marker == fence:
                in_fence = False
                fence = ""
            continue
        if in_fence:
            continue
        prose = readme_contract.CODE_SPAN_RE.sub(
            lambda match: " " * len(match.group(0)), line
        )
        links.extend((index, target) for target in readme_contract.LINK_RE.findall(prose))
    return links


def validate_relative_links(repo: Path, doc: Path, lines: list[str], rule: str) -> list[DocFinding]:
    findings: list[DocFinding] = []
    repo = repo.resolve()
    for index, raw_target in markdown_links(lines):
        if not raw_target or raw_target.startswith(("#", "/")):
            continue
        if readme_contract.SCHEME_RE.match(raw_target):
            continue
        if any(char in raw_target for char in "<>{}"):
            continue
        path_part, _separator, anchor = raw_target.partition("#")
        path_part = path_part.split("?", 1)[0]
        target = (doc.parent / path_part).resolve() if path_part else doc
        try:
            target.relative_to(repo)
        except ValueError:
            findings.append(
                DocFinding(rule, index + 1, f"relative link leaves the checkout: {raw_target}")
            )
            continue
        if not target.is_file():
            findings.append(
                DocFinding(rule, index + 1, f"relative link target does not exist: {raw_target}")
            )
            continue
        if anchor and target.suffix.lower() == ".md":
            target_lines = readme_contract.without_fenced_content(
                target.read_text(encoding="utf-8").splitlines()
            )
            anchors = {github_slug(name) for level in (1, 2, 3, 4) for _at, name in headings(target_lines, level)}
            if anchor not in anchors:
                findings.append(
                    DocFinding(rule, index + 1, f"Markdown anchor does not exist: {raw_target}")
                )
    return findings


def parse_id(cell: str) -> str:
    match = re.fullmatch(r"`([^`]+)`", cell)
    return match.group(1) if match else ""


def roadmap_link_ids(evidence: str) -> list[str]:
    ids: list[str] = []
    for target in readme_contract.LINK_RE.findall(evidence):
        path_part, separator, anchor = target.partition("#")
        if separator and Path(path_part).name == "ROADMAP.md" and anchor:
            ids.append(anchor)
    return ids


def validate_status(
    repo: Path,
    status: Path,
    package_index: dict[str, set[str]],
) -> StatusReport:
    raw = status.read_bytes()
    lines = raw.decode("utf-8").splitlines()
    prose = readme_contract.without_fenced_content(lines)
    findings: list[DocFinding] = []
    warnings: list[str] = []
    surfaces: list[StatusSurface] = []

    h1 = headings(prose, 1)
    if len(h1) != 1:
        findings.append(DocFinding("S1", 1, f"STATUS must contain exactly one H1; found {len(h1)}"))
    if headings(prose, 3):
        findings.append(DocFinding("S1", 1, "STATUS uses the flat Support matrix; H3 sections are not allowed"))
    positions, heading_findings = required_heading_positions(prose, STATUS_H2, "STATUS", "S1")
    findings.extend(heading_findings)

    text = "\n".join(prose)
    if BANNED_META_FIELD_RE.search(text) or PERCENT_PROGRESS_RE.search(text):
        findings.append(
            DocFinding("S2", 1, "STATUS must not contain self-graded, owner, schedule, or percent-complete fields")
        )

    if "State definitions" in positions:
        rows = table_after(prose, positions["State definitions"])
        if (
            len(rows) != 5
            or tuple(rows[0][1]) != STATUS_DEFINITION_HEADER
            or not readme_contract.is_separator_row(rows[1][1])
            or [row[1][0] for row in rows[2:] if len(row[1]) == 2] != list(STATUS_STATES)
            or any(len(cells) != 2 or not cells[1] for _at, cells in rows[2:])
        ):
            findings.append(
                DocFinding(
                    "S3",
                    positions["State definitions"] + 1,
                    "State definitions must define Supported, Limited, and Not supported in that order",
                )
            )

    for section in ("Scope", "Evidence policy"):
        if section in positions:
            start = positions[section]
            end = readme_contract.section_end(prose, start, 2)
            if not any(line.strip() for line in prose[start + 1 : end]):
                findings.append(DocFinding("S3", start + 1, f"{section} cannot be empty"))

    if "Support matrix" in positions:
        rows = table_after(prose, positions["Support matrix"])
        if (
            len(rows) < 3
            or tuple(rows[0][1]) != STATUS_HEADER
            or not readme_contract.is_separator_row(rows[1][1])
        ):
            findings.append(
                DocFinding(
                    "S4",
                    positions["Support matrix"] + 1,
                    "Support matrix columns must be: " + " | ".join(STATUS_HEADER),
                )
            )
            rows = []

        seen_names: set[str] = set()
        seen_ids: set[str] = set()
        for row_at, cells in rows[2:]:
            if len(cells) != 6:
                findings.append(DocFinding("S4", row_at + 1, "Support matrix row must have six cells"))
                continue
            name, raw_id, state, scope, limits, evidence = cells
            surface_id = parse_id(raw_id)
            if not name or not scope or not limits or not evidence:
                findings.append(DocFinding("S4", row_at + 1, "Support matrix cells cannot be empty"))
            if not readme_contract.ID_RE.fullmatch(surface_id):
                findings.append(DocFinding("S4", row_at + 1, f"invalid support ID: {raw_id}"))
            if name in seen_names or surface_id in seen_ids:
                findings.append(DocFinding("S4", row_at + 1, "Support names and IDs must be unique"))
            if state not in STATUS_STATES:
                findings.append(DocFinding("S4", row_at + 1, f"invalid support state: {state}"))
            seen_names.add(name)
            seen_ids.add(surface_id)

            gates: list[str] = []
            roadmap_id: str | None = None
            if state in {"Supported", "Limited"}:
                gates = [match.group(0).strip("`") for match in readme_contract.CODE_SPAN_RE.finditer(evidence)]
                if not gates:
                    findings.append(
                        DocFinding("S5", row_at + 1, f"{state} row must name an executable backticked gate")
                    )
                for command in gates:
                    gate_findings = readme_contract.validate_gate(
                        repo,
                        status,
                        command,
                        row_at + 1,
                        package_index,
                        warnings,
                    )
                    findings.extend(
                        DocFinding("S5", finding.line, finding.message)
                        for finding in gate_findings
                    )
                if state == "Limited":
                    roadmap_ids = roadmap_link_ids(limits)
                    roadmap_id = roadmap_ids[0] if len(roadmap_ids) == 1 else None
                    if len(roadmap_ids) != 1:
                        findings.append(
                            DocFinding(
                                "S6",
                                row_at + 1,
                                "Limited row must link its boundary to exactly one ROADMAP outcome or non-goal",
                            )
                        )
            elif state == "Not supported":
                roadmap_ids = roadmap_link_ids(evidence)
                roadmap_id = roadmap_ids[0] if len(roadmap_ids) == 1 else None
                if len(roadmap_ids) != 1:
                    findings.append(
                        DocFinding(
                            "S6",
                            row_at + 1,
                            "Not supported row must link to exactly one ROADMAP outcome or non-goal",
                        )
                    )
                if readme_contract.CODE_SPAN_RE.search(evidence):
                    findings.append(
                        DocFinding("S6", row_at + 1, "Not supported evidence must not claim a current gate")
                    )

            surfaces.append(
                StatusSurface(name, surface_id, state, scope, limits, evidence, gates, roadmap_id)
            )

    findings.extend(validate_relative_links(repo, status, lines, "S7"))
    findings.sort(key=lambda finding: (finding.line, finding.rule, finding.message))
    return StatusReport(
        path=str(status.relative_to(repo)),
        sha256=hashlib.sha256(raw).hexdigest(),
        surfaces=surfaces,
        warnings=warnings,
        findings=findings,
    )


def detail_fields(lines: list[str], start: int, end: int) -> tuple[dict[str, str], list[str]]:
    fields: dict[str, str] = {}
    order: list[str] = []
    current = ""
    for line in lines[start + 1 : end]:
        match = FIELD_RE.match(line)
        if match:
            current = match.group(1)
            order.append(current)
            fields[current] = match.group(2).strip()
            continue
        if current and line.startswith("  ") and line.strip():
            fields[current] = f"{fields[current]} {line.strip()}".strip()
        elif line.strip():
            current = ""
    return fields, order


def validate_roadmap(repo: Path, roadmap: Path) -> RoadmapReport:
    raw = roadmap.read_bytes()
    lines = raw.decode("utf-8").splitlines()
    prose = readme_contract.without_fenced_content(lines)
    findings: list[DocFinding] = []
    items: list[RoadmapItem] = []

    h1 = headings(prose, 1)
    if len(h1) != 1:
        findings.append(DocFinding("M1", 1, f"ROADMAP must contain exactly one H1; found {len(h1)}"))
    if headings(prose, 4):
        findings.append(DocFinding("M1", 1, "ROADMAP items are flat H3 sections; H4 hierarchy is not allowed"))
    positions, heading_findings = required_heading_positions(prose, ROADMAP_H2, "ROADMAP", "M1")
    findings.extend(heading_findings)

    text = "\n".join(prose)
    if BANNED_META_FIELD_RE.search(text) or PERCENT_PROGRESS_RE.search(text):
        findings.append(
            DocFinding("M2", 1, "ROADMAP must leave work state, owners, schedules, and percentages in the tracker")
        )

    if "Purpose" in positions:
        start = positions["Purpose"]
        end = readme_contract.section_end(prose, start, 2)
        if not any(line.strip() for line in prose[start + 1 : end]):
            findings.append(DocFinding("M3", start + 1, "Purpose cannot be empty"))

    seen_ids: set[str] = set()
    seen_names: set[str] = set()
    for horizon in ROADMAP_H2[1:]:
        if horizon not in positions:
            continue
        start = positions[horizon]
        end = readme_contract.section_end(prose, start, 2)
        section_h3 = [(at, name) for at, name in headings(prose, 3) if start < at < end]
        body_without_h3 = [
            line.strip()
            for line in prose[start + 1 : end]
            if line.strip() and not line.startswith("### ")
        ]
        if not section_h3 and body_without_h3 != ["No items."]:
            findings.append(
                DocFinding("M3", start + 1, f"{horizon} must contain H3 items or exactly 'No items.'")
            )
        for item_index, (item_at, name) in enumerate(section_h3):
            item_end = section_h3[item_index + 1][0] if item_index + 1 < len(section_h3) else end
            fields, order = detail_fields(prose, item_at, item_end)
            required = NON_GOAL_FIELDS if horizon == "Non-goals" else OUTCOME_FIELDS
            if tuple(order) != required:
                findings.append(
                    DocFinding(
                        "M4",
                        item_at + 1,
                        f"{horizon} item fields must be exactly: " + " | ".join(required),
                    )
                )
            item_id = parse_id(fields.get("ID", ""))
            if not readme_contract.ID_RE.fullmatch(item_id):
                findings.append(DocFinding("M4", item_at + 1, f"invalid roadmap ID: {fields.get('ID', '')}"))
            if item_id and github_slug(name) != item_id:
                findings.append(
                    DocFinding("M4", item_at + 1, "roadmap ID must match the H3 Markdown anchor")
                )
            if name in seen_names or item_id in seen_ids:
                findings.append(DocFinding("M4", item_at + 1, "roadmap names and IDs must be unique"))
            seen_names.add(name)
            seen_ids.add(item_id)
            content_fields = ("Reason",) if horizon == "Non-goals" else (
                "Outcome",
                "Boundary",
                "Completion evidence",
            )
            for field_name in content_fields:
                if len(strip_inline(fields.get(field_name, "")).split()) < 3:
                    findings.append(
                        DocFinding("M4", item_at + 1, f"{field_name} must state a concrete boundary or outcome")
                    )
            if horizon != "Non-goals":
                tracking = fields.get("Tracking", "")
                if tracking != "Not assigned." and not readme_contract.LINK_RE.search(tracking):
                    findings.append(
                        DocFinding("M5", item_at + 1, "Tracking must be 'Not assigned.' or a Markdown link")
                    )
            items.append(RoadmapItem(name, item_id, horizon, fields))

    findings.extend(validate_relative_links(repo, roadmap, lines, "M6"))
    findings.sort(key=lambda finding: (finding.line, finding.rule, finding.message))
    return RoadmapReport(
        path=str(roadmap.relative_to(repo)),
        sha256=hashlib.sha256(raw).hexdigest(),
        items=items,
        findings=findings,
    )


def validate_supporting_doc(
    repo: Path,
    document: Path,
    kind: str,
    required_h2: tuple[str, ...],
    table_heading: str,
    table_header: tuple[str, ...],
) -> SupportingDocReport:
    raw = document.read_bytes()
    lines = raw.decode("utf-8").splitlines()
    prose = readme_contract.without_fenced_content(lines)
    findings: list[DocFinding] = []
    labels = {
        "protocol": "protocol guide",
        "clients": "generated-client guide",
        "indexing": "indexing guide",
        "querying": "querying guide",
        "gke": "GKE guide",
        "client-integration": "client-integration guide",
        "migration": "migration guide",
    }
    label = labels.get(kind, f"{kind} guide")

    h1 = headings(prose, 1)
    if len(h1) != 1:
        findings.append(
            DocFinding("D1", 1, f"{label} must contain exactly one H1; found {len(h1)}")
        )
    positions, heading_findings = required_heading_positions(
        prose, required_h2, label, "D1"
    )
    findings.extend(heading_findings)

    for section in required_h2:
        if section not in positions:
            continue
        start = positions[section]
        end = readme_contract.section_end(prose, start, 2)
        if not any(line.strip() for line in prose[start + 1 : end]):
            findings.append(DocFinding("D2", start + 1, f"{section} cannot be empty"))

    if table_heading in positions:
        rows = table_after(prose, positions[table_heading])
        if (
            len(rows) < 3
            or tuple(rows[0][1]) != table_header
            or not readme_contract.is_separator_row(rows[1][1])
        ):
            findings.append(
                DocFinding(
                    "D3",
                    positions[table_heading] + 1,
                    f"{table_heading} columns must be: " + " | ".join(table_header),
                )
            )
        else:
            seen_keys: set[str] = set()
            for row_at, cells in rows[2:]:
                if len(cells) != len(table_header) or any(not cell for cell in cells):
                    findings.append(
                        DocFinding(
                            "D3",
                            row_at + 1,
                            f"{table_heading} rows must have {len(table_header)} non-empty cells",
                        )
                    )
                    continue
                key = strip_inline(cells[0]).strip().lower()
                if key in seen_keys:
                    findings.append(
                        DocFinding("D3", row_at + 1, f"duplicate {table_header[0]}: {cells[0]}")
                    )
                seen_keys.add(key)

    if "Supporting documents" in positions:
        start = positions["Supporting documents"]
        end = readme_contract.section_end(prose, start, 2)
        if not markdown_links(lines[start + 1 : end]):
            findings.append(
                DocFinding(
                    "D2",
                    start + 1,
                    "Supporting documents must contain at least one Markdown link",
                )
            )

    findings.extend(validate_relative_links(repo, document, lines, "D4"))
    findings.sort(key=lambda finding: (finding.line, finding.rule, finding.message))
    return SupportingDocReport(
        kind=kind,
        path=str(document.relative_to(repo)),
        sha256=hashlib.sha256(raw).hexdigest(),
        findings=findings,
    )


def supporting_doc_targets(readme: Path) -> set[Path]:
    lines = readme.read_text(encoding="utf-8").splitlines()
    prose = readme_contract.without_fenced_content(lines)
    supporting = [at for at, name in headings(prose, 2) if name == "Supporting documents"]
    if len(supporting) != 1:
        return set()
    start = supporting[0]
    end = readme_contract.section_end(prose, start, 2)
    return {
        (readme.parent / target.partition("#")[0].split("?", 1)[0]).resolve()
        for _line, target in markdown_links(lines[start + 1 : end])
        if target
        and not target.startswith(("#", "/"))
        and not readme_contract.SCHEME_RE.match(target)
    }


def validate_lumen_assertions(project: Path, documents: list[Path]) -> list[DocFinding]:
    """Refuse a small set of unambiguously reversed Lumen contract claims."""

    if project.as_posix().split("/")[-2:] != ["apps", "lumen"]:
        return []
    findings: list[DocFinding] = []
    for document in documents:
        if not document.is_file():
            continue
        lines = readme_contract.without_fenced_content(
            document.read_text(encoding="utf-8").splitlines()
        )
        for index, line in enumerate(lines):
            for assertion in LUMEN_FORBIDDEN_ASSERTIONS:
                if assertion.search(strip_inline(line)):
                    findings.append(
                        DocFinding(
                            "P7",
                            index + 1,
                            f"forbidden Lumen contract assertion in {document.name}: {line.strip()}",
                        )
                    )
    return findings


def validate_project(
    repo: Path,
    project: Path,
    package_index: dict[str, set[str]] | None = None,
) -> ProjectReport:
    findings: list[DocFinding] = []
    supporting_reports: list[SupportingDocReport] = []
    paths = {name: project / name for name in DOC_NAMES}
    for name, path in paths.items():
        if not path.is_file():
            findings.append(DocFinding("P1", 1, f"project document does not exist: {name}"))

    package_index = package_index if package_index is not None else readme_contract.cargo_index(repo)
    readme_report = (
        readme_contract.validate_readme(repo, paths["README.md"], package_index)
        if paths["README.md"].is_file()
        else None
    )
    status_report = (
        validate_status(repo, paths["STATUS.md"], package_index)
        if paths["STATUS.md"].is_file()
        else None
    )
    roadmap_report = (
        validate_roadmap(repo, paths["ROADMAP.md"])
        if paths["ROADMAP.md"].is_file()
        else None
    )

    targets = supporting_doc_targets(paths["README.md"]) if readme_report else set()
    if readme_report:
        for required in ("STATUS.md", "ROADMAP.md"):
            if paths[required].resolve() not in targets:
                findings.append(
                    DocFinding("P2", 1, f"README must link {required} from Supporting documents")
                )

    for relative, spec in SUPPORTING_DOC_SPECS.items():
        document = project / relative
        linked = document.resolve() in targets
        if document.is_file():
            if not linked:
                findings.append(
                    DocFinding(
                        "P6",
                        1,
                        f"README must link {relative} from Supporting documents",
                    )
                )
            kind, required_h2, table_heading, table_header = spec
            supporting_reports.append(
                validate_supporting_doc(
                    repo,
                    document,
                    kind,
                    required_h2,
                    table_heading,
                    table_header,
                )
            )
        elif linked:
            findings.append(
                DocFinding("P6", 1, f"linked supporting document does not exist: {relative}")
            )

    fixed_targets = {(project / relative).resolve() for relative in SUPPORTING_DOC_SPECS}
    for target in sorted(targets):
        if target in fixed_targets:
            continue
        try:
            relative = target.relative_to(project.resolve())
        except ValueError:
            continue
        if relative.parent != Path("docs") or not is_migration_guide(relative.name):
            continue
        if not target.is_file():
            findings.append(
                DocFinding("P6", 1, f"linked supporting document does not exist: {relative}")
            )
            continue
        kind, required_h2, table_heading, table_header = MIGRATION_DOC_SPEC
        document = project / relative
        supporting_reports.append(
            validate_supporting_doc(
                repo,
                document,
                kind,
                required_h2,
                table_heading,
                table_header,
            )
        )

    adopted_paths = [paths[name] for name in DOC_NAMES]
    adopted_paths.extend(repo / document.path for document in supporting_reports)
    findings.extend(validate_lumen_assertions(project, adopted_paths))

    if all(paths[name].is_file() for name in DOC_NAMES):
        titles = {
            name: headings(
                readme_contract.without_fenced_content(
                    paths[name].read_text(encoding="utf-8").splitlines()
                ),
                1,
            )
            for name in DOC_NAMES
        }
        if len(titles["README.md"]) == 1:
            product = titles["README.md"][0][1]
            expected = {
                "STATUS.md": f"{product} status",
                "ROADMAP.md": f"{product} roadmap",
            }
            for name, title in expected.items():
                if len(titles[name]) == 1 and titles[name][0][1] != title:
                    findings.append(
                        DocFinding("P5", 1, f"{name} H1 must be '# {title}'")
                    )

    roadmap_ids = {item.item_id for item in roadmap_report.items} if roadmap_report else set()
    if status_report:
        for surface in status_report.surfaces:
            if surface.roadmap_id and surface.roadmap_id not in roadmap_ids:
                findings.append(
                    DocFinding(
                        "P3",
                        1,
                        f"STATUS surface {surface.surface_id!r} links unknown ROADMAP ID {surface.roadmap_id!r}",
                    )
                )
            if surface.surface_id in roadmap_ids and surface.state == "Supported":
                findings.append(
                    DocFinding(
                        "P4",
                        1,
                        f"current surface {surface.surface_id!r} also appears as future work",
                    )
                )

    findings.sort(key=lambda finding: (finding.line, finding.rule, finding.message))
    return ProjectReport(
        path=str(project.relative_to(repo)),
        readme=readme_report,
        status=status_report,
        roadmap=roadmap_report,
        supporting_docs=supporting_reports,
        findings=findings,
    )


def print_text(reports: list[ProjectReport]) -> None:
    for report in reports:
        state = "PASS" if report.ok else "FAIL"
        print(f"{state} {report.path}")
        if report.readme:
            print(
                f"  README sha256={report.readme.sha256} "
                f"capabilities={len(report.readme.capabilities)}"
            )
            for finding in report.readme.findings:
                print(f"  README {finding.rule} line {finding.line}: {finding.message}")
        if report.status:
            print(
                f"  STATUS sha256={report.status.sha256} "
                f"surfaces={len(report.status.surfaces)}"
            )
            for warning in report.status.warnings:
                print(f"  STATUS WARN {warning}")
            for finding in report.status.findings:
                print(f"  STATUS {finding.rule} line {finding.line}: {finding.message}")
        if report.roadmap:
            print(
                f"  ROADMAP sha256={report.roadmap.sha256} "
                f"items={len(report.roadmap.items)}"
            )
            for finding in report.roadmap.findings:
                print(f"  ROADMAP {finding.rule} line {finding.line}: {finding.message}")
        for document in report.supporting_docs:
            print(
                f"  SUPPORTING kind={document.kind} path={document.path} "
                f"sha256={document.sha256}"
            )
            for finding in document.findings:
                print(
                    f"  SUPPORTING {finding.rule} line {finding.line}: {finding.message}"
                )
        for finding in report.findings:
            print(f"  SET {finding.rule} line {finding.line}: {finding.message}")


def clean_reader_prompt(report: ProjectReport, project: Path) -> str:
    assert report.readme and report.status and report.roadmap
    expected = {
        "README.md": report.readme.sha256,
        "STATUS.md": report.status.sha256,
        "ROADMAP.md": report.roadmap.sha256,
    }
    paths = [project / name for name in DOC_NAMES]
    supporting_by_kind: dict[str, list[SupportingDocReport]] = {}
    for document in report.supporting_docs:
        supporting_by_kind.setdefault(document.kind, []).append(document)
        relative = str(Path(document.path).relative_to(report.path))
        expected[relative] = document.sha256
        paths.append(project / relative)

    output: dict[str, object] = {
        "sha256": {name: "<sha256>" for name in expected},
        "status": "reviewed | stale_input",
        "stated": {
            "purpose": "<what the product is>",
            "not_for": ["<explicit product boundary>"],
            "primary_workflow": ["<ordered steps>"],
            "functional_surfaces": ["<main user-facing functions>"],
            "capabilities": [
                {"name": "<heading>", "id": "<id>", "sources": ["<source>"]}
            ],
        },
        "current_support": [
            {
                "name": "<surface>",
                "id": "<id>",
                "state": "<state>",
                "scope": "<supported scope>",
                "limits": "<limits>",
            }
        ],
        "future": {
            "near_term": [{"name": "<outcome>", "id": "<id>"}],
            "later": [{"name": "<outcome>", "id": "<id>"}],
            "non_goals": [{"name": "<non-goal>", "id": "<id>"}],
        },
        "inferences": ["<clearly labelled inference>"],
        "cross_document_contradictions": [
            "<statement that conflicts across documents>"
        ],
        "missing_details": [
            "<detail delegated to another maintained source or still hard to find>"
        ],
        "comprehension_score": 0,
    }
    interfaces: dict[str, object] = {}
    if "protocol" in supporting_by_kind:
        interfaces["protocol"] = {
            "contract_map": [
                {
                    "fact": "<fact class>",
                    "canonical_source": "<source>",
                    "discovery": "<command or file>",
                }
            ],
            "operation_families": ["<operation family>"],
            "current_boundaries": ["<current protocol boundary>"],
        }
    if "clients" in supporting_by_kind:
        interfaces["clients"] = {
            "artifact_model": "<what is generated, committed, or published>",
            "languages": [
                {
                    "language": "<language>",
                    "generated_form": "<generated form>",
                    "transport": "<transport>",
                    "auth_input": "<auth input>",
                    "current_limits": "<limits>",
                }
            ],
            "current_boundaries": ["<current generated-client boundary>"],
        }
    if "indexing" in supporting_by_kind:
        interfaces["indexing"] = {
            "contract_map": [
                {
                    "fact": "<fact class>",
                    "canonical_source": "<source>",
                    "discovery": "<command or file>",
                }
            ],
            "data_ownership": ["<source and derived-index boundary>"],
            "current_contract": ["<current schema, write, durability, or rebuild fact>"],
            "target_contract": ["<future schema, write, durability, or rebuild fact>"],
            "current_boundaries": ["<current indexing boundary>"],
        }
    if "querying" in supporting_by_kind:
        interfaces["querying"] = {
            "contract_map": [
                {
                    "fact": "<fact class>",
                    "canonical_source": "<source>",
                    "discovery": "<command or file>",
                }
            ],
            "selection_and_hydration": ["<selection and hydration responsibility>"],
            "current_contract": ["<current query or result fact>"],
            "target_contract": ["<future query, facet, metric, or limit fact>"],
            "current_boundaries": ["<current querying boundary>"],
        }
    if "gke" in supporting_by_kind:
        interfaces["gke"] = {
            "contract_map": [
                {
                    "fact": "<fact class>",
                    "canonical_source": "<source>",
                    "discovery": "<command or file>",
                }
            ],
            "support_tiers": ["<current or target environment and support tier>"],
            "runtime_topologies": ["<pod, shard, replica, voter, and HA meaning>"],
            "kubernetes_boundary": ["<portable contract and GKE profile boundary>"],
            "current_boundaries": ["<current GKE or Kubernetes boundary>"],
        }
    if "client-integration" in supporting_by_kind:
        interfaces["client_integration"] = {
            "contract_map": [
                {
                    "fact": "<fact class>",
                    "canonical_source": "<source>",
                    "discovery": "<command or file>",
                }
            ],
            "responsibility_boundary": ["<server, generated-client, template, or caller owner>"],
            "connection_profiles": ["<Standalone or Managed connection behavior>"],
            "workload_template": ["<current or target Kubernetes workload behavior>"],
            "source_integration": ["<source database and hydration behavior>"],
            "current_boundaries": ["<current client-integration boundary>"],
        }
    if "migration" in supporting_by_kind:
        interfaces["migrations"] = [
            {
                "path": "<migration guide path>",
                "compatibility_rows": [
                    {
                        "surface": "<surface>",
                        "from": "<0.4.x behavior>",
                        "to": "<0.5.0 behavior>",
                        "required_action": "<caller action>",
                    }
                ],
                "activation": ["<managed activation rule>"],
                "tools": ["<migration tool>"],
            }
            for _document in supporting_by_kind["migration"]
        ]
    if interfaces:
        output["interfaces"] = interfaces

    path_list = "\n".join(f"- {path}" for path in paths)
    interface_instruction = ""
    if interfaces:
        interface_instruction = (
            " Also list every contract-map row from each adopted guide, "
            "every generated-client language row, and every migration compatibility "
            "row from the adopted supporting guides. Keep current behavior separate "
            "from target behavior."
        )
    return f"""Read only these {len(paths)} files:
{path_list}

Do not inspect any other file, repository state, prior draft, conversation, or
validator output. Do not edit anything. Compute each file SHA-256 yourself.
The expected SHA-256 map is {json.dumps(expected, sort_keys=True)}. Stop and
report stale_input if any value differs.

Act as a first-time developer who must understand what the product does, what
is supported now, and what is future work. Return JSON only:

{json.dumps(output, indent=2)}

List every capability, support surface, roadmap outcome, and non-goal.{interface_instruction} Keep
document facts separate from inferences. The score is diagnostic. Do not turn
it into the verdict. Use an integer from 0 to 100. Use 100 when a first-time
reader can recover every requested fact, state, boundary, and document
relationship without a contradiction. Lower the score for an omitted or
unclear requested fact. Do not lower it only because a detail is clearly
delegated to a named maintained source.
"""


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    subparsers = parser.add_subparsers(dest="command", required=True)

    check_parser = subparsers.add_parser("check", help="run deterministic project-doc checks")
    check_parser.add_argument(
        "paths",
        nargs="+",
        help="project directory, core document, or conventional supporting document",
    )
    check_parser.add_argument("--format", choices=("text", "json"), default="text")

    prompt_parser = subparsers.add_parser("prompt", help="emit the clean-reader task")
    prompt_parser.add_argument(
        "path",
        help="project directory, core document, or conventional supporting document",
    )

    args = parser.parse_args(argv)
    repo = readme_contract.repo_root()
    package_index = readme_contract.cargo_index(repo)

    if args.command == "check":
        reports = [
            validate_project(repo, resolve_project(repo, raw), package_index)
            for raw in args.paths
        ]
        if args.format == "json":
            print(json.dumps({"reports": [report.as_dict() for report in reports]}, indent=2))
        else:
            print_text(reports)
        return 0 if all(report.ok for report in reports) else 1

    project = resolve_project(repo, args.path)
    report = validate_project(repo, project, package_index)
    if not report.ok:
        print_text([report])
        print(
            "error: clean-reader review is skipped until deterministic findings are fixed",
            file=sys.stderr,
        )
        return 1
    print(clean_reader_prompt(report, project))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
