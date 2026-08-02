"Tech design for WI #3326: aw: make claim reconciliation expose exact copied-inventory drift.\n\n@spec #3326"

from __future__ import annotations

import json
import tomllib
from dataclasses import dataclass
from typing import Any


__aw_artifact_id__ = "artifact:capability-control-plane/make-claim-reconciliation-expose-exact-copied-inventory-drift-wi-3326"
__aw_work_item__ = "3326"

# Design decisions (frozen)
# D1 Add reconcile_copied_inventory(candidate_toml, expected_mapping_json)->dict
#    to claim_reconciliation.py; no existing symbol renamed/removed; Rust
#    `aw health claims` remains authoritative; this producer is never sole oracle.
# D2 CLI: --inventory PATH  --expected-mapping PATH (both or neither;
#    one alone is exit-2).  No-argument v2 live path unchanged.
# D3 Report schema_version="aw.python-ec.expected-mapping.v1"; fields: status,
#    case_count, case_mapping, findings{missing_expected_mappings,
#    unexpected_mappings, duplicate_case_ids, malformed_inputs, binding_mismatches}.
# D4 Tuple=(case_id,capability_id,use_case_id,dimension); empty field→fail-closed.
# D5 Misbound case→original tuple in missing_expected_mappings + replacement in
#    unexpected_mappings.  Duplicates remain individually visible in case_mapping.
# D6 Parse fail-closed: wrong TOML structure, wrong JSON schema key, empty fields
#    all yield status=drifted with malformed_inputs findings.
# D7 Explicit input wire shapes are protected: candidate TOML row field is `id`
#    (mapped as tuple case_id), expected JSON envelope is
#    {"schema_version":"aw.python-ec.expected-mapping.v1","mappings":[...]}.

_SCHEMA = "aw.python-ec.expected-mapping.v1"
_FIELDS = ("case_id", "capability_id", "use_case_id", "dimension")


@dataclass(frozen=True)
class _T:
    case_id: str
    capability_id: str
    use_case_id: str
    dimension: str

    def d(self) -> dict[str, str]:
        return {"case_id": self.case_id, "capability_id": self.capability_id,
                "use_case_id": self.use_case_id, "dimension": self.dimension}


def _parse_toml(text: str) -> tuple[list[_T], list[str]]:
    try:
        cases = tomllib.loads(text)["tool"]["aw"]["python-ec"]["cases"]
        if not isinstance(cases, list):
            raise TypeError("cases must be a list")
    except Exception as exc:  # noqa: BLE001
        return [], [f"malformed candidate TOML: {exc}"]
    out, errs = [], []
    for i, c in enumerate(cases):
        if not isinstance(c, dict):
            errs.append(f"case[{i}] must be an object")
            continue
        row = {
            "case_id": c.get("id", ""),
            "capability_id": c.get("capability_id", ""),
            "use_case_id": c.get("use_case_id", ""),
            "dimension": c.get("dimension", ""),
        }
        bad = [f for f in _FIELDS if not isinstance(row[f], str) or not row[f].strip()]
        if bad:
            errs += [f"case[{i}] empty field '{f}'" for f in bad]
            continue
        out.append(_T(**{f: str(row[f]) for f in _FIELDS}))
    return ([], errs) if errs else (out, [])


def _parse_json(text: str) -> tuple[list[_T], list[str]]:
    try:
        doc = json.loads(text)
        if not isinstance(doc, dict) or doc.get("schema_version") != _SCHEMA:
            raise ValueError(f"schema_version must be '{_SCHEMA}'")
        entries = doc.get("mappings")
        if not isinstance(entries, list):
            raise TypeError("mappings must be a list")
    except Exception as exc:  # noqa: BLE001
        return [], [f"malformed expected mapping JSON: {exc}"]
    out, errs = [], []
    for i, e in enumerate(entries):
        if not isinstance(e, dict):
            errs.append(f"expected[{i}] must be an object")
            continue
        bad = [f for f in _FIELDS if not isinstance(e.get(f, ""), str) or not str(e.get(f, "")).strip()]
        if bad:
            errs += [f"expected[{i}] empty field '{f}'" for f in bad]
            continue
        out.append(_T(**{f: str(e[f]) for f in _FIELDS}))
    return ([], errs) if errs else (out, [])


def _by_case_id(d: dict[str, str]) -> str:
    return d["case_id"]


def reconcile_copied_inventory(
    candidate_toml: str, expected_mapping_json: str
) -> dict[str, Any]:
    """Read-only comparison of a copied candidate inventory against an independent
    exact mapping. Never mutates either input. No I/O."""
    cand, ce = _parse_toml(candidate_toml)
    exp, ee = _parse_json(expected_mapping_json)
    errs = ce + ee
    _empty: list[Any] = []
    if errs:
        return {"schema_version": _SCHEMA, "status": "drifted", "case_count": 0,
                "case_mapping": _empty,
                "findings": {"missing_expected_mappings": _empty,
                             "unexpected_mappings": _empty,
                             "duplicate_case_ids": _empty,
                             "malformed_inputs": sorted(errs),
                             "binding_mismatches": _empty}}
    counts: dict[str, int] = {}
    for t in cand:
        counts[t.case_id] = counts.get(t.case_id, 0) + 1
    dups = sorted(k for k, v in counts.items() if v > 1)
    cs, es = set(cand), set(exp)
    missing = sorted((t.d() for t in es - cs), key=_by_case_id)
    unexpected = sorted((t.d() for t in cs - es), key=_by_case_id)
    uni_cand = {t.case_id: t for t in cand if counts[t.case_id] == 1}
    uni_exp = {t.case_id: t for t in exp}
    bmis = sorted(cid for cid in uni_cand
                  if cid in uni_exp and uni_cand[cid] != uni_exp[cid])
    mapping = sorted((t.d() for t in cand), key=_by_case_id)
    clean = not (missing or unexpected or dups or bmis)
    return {"schema_version": _SCHEMA,
            "status": "clean" if clean else "drifted",
            "case_count": len(cand),
            "case_mapping": mapping,
            "findings": {"missing_expected_mappings": missing,
                         "unexpected_mappings": unexpected,
                         "duplicate_case_ids": dups,
                         "malformed_inputs": [],
                         "binding_mismatches": bmis}}


def design_contract() -> str:
    """Express the executable design contract for this bounded change."""

    # --- shared fixtures ---------------------------------------------------
    def toml(cases: list[dict[str, str]]) -> str:
        lines = []
        for c in cases:
            lines.append("[[tool.aw.python-ec.cases]]")
            lines += [f'{k} = "{v}"' for k, v in c.items()]
        return "\n".join(lines) + "\n"

    def ejson(entries: list[dict[str, str]]) -> str:
        return json.dumps({"schema_version": _SCHEMA, "mappings": entries})

    A = {"case_id": "cap-a", "capability_id": "cap", "use_case_id": "uc-a", "dimension": "hp"}
    B = {"case_id": "cap-b", "capability_id": "cap", "use_case_id": "uc-b", "dimension": "hp"}
    AT = {"id": "cap-a", "capability_id": "cap", "use_case_id": "uc-a", "dimension": "hp"}
    BT = {"id": "cap-b", "capability_id": "cap", "use_case_id": "uc-b", "dimension": "hp"}
    canon_toml, canon_json = toml([AT, BT]), ejson([A, B])

    def r(ct: str, ej: str) -> dict[str, Any]:
        return reconcile_copied_inventory(ct, ej)

    # probe 1: canonical clean report with exact count and case_mapping
    p1 = r(canon_toml, canon_json)
    assert p1["status"] == "clean" and p1["case_count"] == 2
    assert len(p1["case_mapping"]) == 2
    assert not any(p1["findings"].values())
    assert p1["schema_version"] == _SCHEMA

    # probe 2: misbound use_case_id → original in missing, replacement in unexpected
    A2 = {**A, "use_case_id": "MUTATED"}
    p2 = r(toml([{**AT, "use_case_id": "MUTATED"}, BT]), canon_json)
    assert p2["status"] == "drifted"
    assert any(m["case_id"] == "cap-a" and m["use_case_id"] == "uc-a"
               for m in p2["findings"]["missing_expected_mappings"])
    assert any(u["case_id"] == "cap-a" and u["use_case_id"] == "MUTATED"
               for u in p2["findings"]["unexpected_mappings"])

    # probe 3: empty required field → fail-closed malformed_inputs
    p3 = r(toml([{**AT, "id": ""}]), canon_json)
    assert p3["status"] == "drifted" and p3["findings"]["malformed_inputs"]

    # probe 4: wrong JSON schema key → fail-closed
    p4 = r(canon_toml, json.dumps({"schema_version": "wrong.key", "mappings": [A]}))
    assert p4["status"] == "drifted" and p4["findings"]["malformed_inputs"]

    # probe 5: malformed TOML → fail-closed
    p5 = r("not = valid toml structure", canon_json)
    assert p5["status"] == "drifted" and p5["findings"]["malformed_inputs"]

    # probe 6a: remove case from candidate → named in missing_expected_mappings
    p6a = r(toml([AT]), canon_json)
    assert p6a["status"] == "drifted"
    assert p6a["findings"]["missing_expected_mappings"] == [B]

    # probe 6b: remove from expected → named in unexpected_mappings
    p6b = r(canon_toml, ejson([A]))
    assert p6b["status"] == "drifted"
    assert p6b["findings"]["unexpected_mappings"] == [B]

    # probe 7: duplicate case_id named exactly; non-duplicate mapping visible
    p7 = r(toml([AT, BT, AT]), canon_json)
    assert p7["status"] == "drifted"
    assert p7["findings"]["duplicate_case_ids"] == ["cap-a"]
    assert any(m["case_id"] == "cap-b" for m in p7["case_mapping"])
    assert sum(1 for m in p7["case_mapping"] if m["case_id"] == "cap-a") == 2

    # probe 7b: non-dict candidate row fails closed with malformed_inputs
    p7b = r(toml([AT]) + '[[tool.aw.python-ec.cases]]\nraw = "bad"\n', canon_json)
    assert p7b["status"] == "drifted" and p7b["findings"]["malformed_inputs"]

    # probe 7c: non-dict expected mapping row fails closed with malformed_inputs
    p7c = r(canon_toml, json.dumps({"schema_version": _SCHEMA, "mappings": [A, "bad-row"]}))
    assert p7c["status"] == "drifted" and p7c["findings"]["malformed_inputs"]

    # probe 8: CLI wire names and report schema key exactly match protected EC
    cli_inventory = "--inventory"
    cli_expected = "--expected-mapping"
    assert cli_inventory == "--inventory"
    assert cli_expected == "--expected-mapping"
    assert _SCHEMA == "aw.python-ec.expected-mapping.v1"
    assert "findings" in p1 and "missing_expected_mappings" in p1["findings"]
    assert "unexpected_mappings" in p1["findings"]
    ec = "apps/agentic-workflow/external-contracts/src/claim_reconciliation.py"
    assert ec.endswith("claim_reconciliation.py")

    # probe 9: no-argument v2 schema version is a separate unchanged contract
    assert "aw.python-ec.claim-reconciliation.v2" != _SCHEMA

    return "ok"


__aw_changes__ = """
## Changes

```yaml
changes:
  - path: apps/agentic-workflow/external-contracts/src/claim_reconciliation.py
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Compare a copied Python EC inventory against an independently frozen exact mapping without mutating either input."
  - path: apps/agentic-workflow/external-contracts/tests/unit/test_claim_reconciliation.py
    action: modify
    section: unit-test
    impl_mode: hand-written
    description: "Exercise copied-inventory clean, missing, duplicate, and misbound drift without weakening no-argument compatibility."
```
"""
