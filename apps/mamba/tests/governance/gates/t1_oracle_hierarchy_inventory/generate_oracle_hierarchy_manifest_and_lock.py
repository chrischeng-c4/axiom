#!/usr/bin/env python3.12
"""Deterministic manifest and evidence-lock producer for Mamba Tier 1 oracle hierarchy."""

import argparse
import datetime
import hashlib
import json
import os
import pathlib
import sys


def find_repo_root() -> pathlib.Path:
    script_path = pathlib.Path(__file__).resolve()
    fdir = script_path.parent
    while fdir != fdir.parent:
        if (fdir / "apps/mamba").is_dir() and (
            (fdir / ".git").exists() or (fdir / "aw.toml").exists()
        ):
            return fdir
        fdir = fdir.parent
    raise RuntimeError(
        "Could not determine repository root containing apps/mamba and (.git or aw.toml)"
    )


def validate_args(args: argparse.Namespace) -> None:
    # 1. mamba-git-sha: 40 lowercase hex
    if len(args.mamba_git_sha) != 40 or not all(c in "0123456789abcdef" for c in args.mamba_git_sha):
        raise ValueError(f"mamba-git-sha must be 40 lowercase hex characters, got: '{args.mamba_git_sha}'")

    # 2. mamba-binary-sha256: 64 lowercase hex
    if len(args.mamba_binary_sha256) != 64 or not all(c in "0123456789abcdef" for c in args.mamba_binary_sha256):
        raise ValueError(f"mamba-binary-sha256 must be 64 lowercase hex characters, got: '{args.mamba_binary_sha256}'")

    # 3. capture-timestamp: explicit RFC3339 UTC timestamp
    ts_str = args.capture_timestamp
    if not (ts_str.endswith("Z") or ts_str.endswith("+00:00") or ts_str.endswith("-00:00")):
        raise ValueError(f"capture-timestamp must be an explicit RFC3339 UTC timestamp, got: '{ts_str}'")
    try:
        dt = datetime.datetime.fromisoformat(ts_str.replace("Z", "+00:00"))
        if dt.utcoffset() != datetime.timedelta(0):
            raise ValueError("timestamp timezone offset must be UTC")
    except Exception as e:
        raise ValueError(f"invalid RFC3339 UTC timestamp '{ts_str}': {e}")

    # 4. platform: explicit platform token
    if not args.platform or not isinstance(args.platform, str):
        raise ValueError("platform token must be a non-empty string")


def validate_and_parse_cases(
    cases_path: pathlib.Path, args: argparse.Namespace
) -> tuple[bytes, str, list[dict], list[tuple[str, str]]]:
    if not cases_path.is_file():
        raise FileNotFoundError(f"cases-jsonl file not found: '{cases_path}'")

    cases_bytes = cases_path.read_bytes()
    cases_sha256 = hashlib.sha256(cases_bytes).hexdigest()

    lines = cases_bytes.decode("utf-8").splitlines()
    non_empty_lines = [line.strip() for line in lines if line.strip()]
    if len(non_empty_lines) != 8:
        raise ValueError(f"cases JSONL must contain exactly 8 non-empty rows, found {len(non_empty_lines)}")

    rows = []
    case_ids = []
    probe_ids = []
    represented_pairs = []
    dimensions = []
    channels = []
    dim_roles = {}

    for line in non_empty_lines:
        row = json.loads(line)
        rows.append(row)

        if row.get("mamba_git_sha") != args.mamba_git_sha:
            raise ValueError(
                f"row mamba_git_sha '{row.get('mamba_git_sha')}' does not match explicit arg '{args.mamba_git_sha}'"
            )
        if row.get("mamba_binary_sha256") != args.mamba_binary_sha256:
            raise ValueError(
                f"row mamba_binary_sha256 '{row.get('mamba_binary_sha256')}' does not match explicit arg '{args.mamba_binary_sha256}'"
            )
        if row.get("platform") != args.platform:
            raise ValueError(
                f"row platform '{row.get('platform')}' does not match explicit arg '{args.platform}'"
            )

        cid = row.get("case_id")
        pid = row.get("probe_id")
        sset = row.get("source_set")
        sid = row.get("source_identity")
        dim = row.get("tier1_dimension")
        chan = row.get("channel")
        role = row.get("sample_role")

        if not cid or not isinstance(cid, str):
            raise ValueError(f"row missing valid case_id: {line}")
        if not pid or not isinstance(pid, str):
            raise ValueError(f"row missing valid probe_id: {line}")
        if not sset or not isinstance(sset, str):
            raise ValueError(f"row missing valid source_set: {line}")
        if not sid or not isinstance(sid, str):
            raise ValueError(f"row missing valid source_identity: {line}")

        case_ids.append(cid)
        probe_ids.append(pid)
        represented_pairs.append((sset, sid))
        if dim:
            dimensions.append(dim)
            dim_roles.setdefault(dim, set()).add(role)
        if chan:
            channels.append(chan)

    if len(set(case_ids)) != 8:
        raise ValueError(f"case_ids must be unique, got duplicates in {case_ids}")
    if len(set(probe_ids)) != 8:
        raise ValueError(f"probe_ids must be unique, got duplicates in {probe_ids}")
    if len(set(represented_pairs)) != 8:
        raise ValueError(f"represented (source_set, source_identity) pairs must be unique, got {len(set(represented_pairs))}")

    req_dimensions = {"behavior", "stability", "efficiency"}
    if set(dimensions) != req_dimensions:
        raise ValueError(f"dimensions must be exactly {req_dimensions}, found {set(dimensions)}")

    for d in req_dimensions:
        roles = {r.replace("_", "-") for r in dim_roles.get(d, set())}
        if "green" not in roles or "intentional-red" not in roles:
            raise ValueError(f"dimension '{d}' must contain both 'green' and 'intentional-red' sample roles, found {roles}")

    req_channels = {"compile", "behavior", "concurrency", "performance"}
    if not req_channels.issubset(set(channels)):
        raise ValueError(f"channels must include {req_channels}, found {set(channels)}")

    return cases_bytes, cases_sha256, rows, represented_pairs


def validate_and_parse_source_sets(
    source_sets_path: pathlib.Path, represented_pairs: list[tuple[str, str]]
) -> tuple[list[dict], list[dict]]:
    if not source_sets_path.is_file():
        raise FileNotFoundError(f"source-sets-json file not found: '{source_sets_path}'")

    data = json.loads(source_sets_path.read_text(encoding="utf-8"))

    records = data.get("source_set_records")
    if not isinstance(records, list) or len(records) != 3:
        raise ValueError("source-sets JSON must contain exactly 3 'source_set_records'")

    expected_names = ["ordinary_parity_corpus", "tier1_ec_cases", "tier1_gate_denominators"]
    record_names = [r.get("name") for r in records if isinstance(r, dict)]
    if record_names != expected_names:
        raise ValueError(f"source_set_records names must be exactly {expected_names}, found {record_names}")

    total_identity_count = 0
    record_counts = {}
    for r in records:
        name = r.get("name")
        paths = r.get("paths")
        count = r.get("identity_count")
        digest = r.get("sha256_digest")

        if not paths or not isinstance(paths, list):
            raise ValueError(f"record '{name}' paths must be non-empty list")
        if paths != sorted(paths) or len(paths) != len(set(paths)):
            raise ValueError(f"record '{name}' paths must be non-empty sorted unique list")
        if not isinstance(count, int) or count <= 0:
            raise ValueError(f"record '{name}' identity_count must be positive integer")
        if not isinstance(digest, str) or len(digest) != 64 or not all(c in "0123456789abcdef" for c in digest):
            raise ValueError(f"record '{name}' sha256_digest must be 64-char lowercase hex")

        total_identity_count += count
        record_counts[name] = count

    identities_map = data.get("source_identities")
    if not isinstance(identities_map, dict):
        raise ValueError("source-sets JSON missing 'source_identities' dict")

    discovered_pairs = set()
    for name in expected_names:
        if name not in identities_map:
            raise ValueError(f"source_identities missing key '{name}'")
        sids = identities_map[name]
        if not isinstance(sids, list):
            raise ValueError(f"source_identities['{name}'] must be list")
        if sids != sorted(sids):
            raise ValueError(f"source_identities['{name}'] must be sorted")
        if len(sids) != record_counts[name]:
            raise ValueError(f"source_identities['{name}'] length {len(sids)} does not match record identity_count {record_counts[name]}")
        for sid in sids:
            discovered_pairs.add((name, sid))

    rep_set = set(represented_pairs)
    for pair in rep_set:
        if pair not in discovered_pairs:
            raise ValueError(f"represented pair {pair} not found in discovered source_identities")

    dispositions = data.get("out_of_scope_dispositions")
    if not isinstance(dispositions, list):
        raise ValueError("source-sets JSON missing 'out_of_scope_dispositions' list")

    expected_disp_fields = {"source_set", "source_identity", "reason", "reviewed_against"}
    disp_pairs = []
    multicore_2022_found = False

    for d in dispositions:
        if not isinstance(d, dict):
            raise ValueError("disposition must be dict")
        if set(d.keys()) != expected_disp_fields:
            raise ValueError(f"disposition must contain exact fields {expected_disp_fields}, got {set(d.keys())}")

        sset = d["source_set"]
        sid = d["source_identity"]
        reason = d["reason"]
        reviewer = d["reviewed_against"]

        if not isinstance(reason, str) or not reason.strip():
            raise ValueError("disposition reason must be non-empty string")
        if not isinstance(reviewer, str) or not reviewer.strip():
            raise ValueError("disposition reviewed_against must be non-empty string")

        pair = (sset, sid)
        if pair not in discovered_pairs:
            raise ValueError(f"disposition pair {pair} refers to undiscovered identity")
        if pair in rep_set:
            raise ValueError(f"disposition pair {pair} overlaps represented pair")

        disp_pairs.append(pair)

        if sid == "apps/mamba/tests/governance/gates/t1_multicore_scaling_denominator/denominator.txt" and reviewer == "#2022":
            multicore_2022_found = True

    if disp_pairs != sorted(disp_pairs) or len(disp_pairs) != len(set(disp_pairs)):
        raise ValueError("out_of_scope_dispositions must be sorted unique pairs")

    expected_disp_count = total_identity_count - 8
    if expected_disp_count != 21896:
        raise ValueError(f"expected sum(identity_count) - 8 = 21,896, but calculated {expected_disp_count}")
    if len(disp_pairs) != 21896:
        raise ValueError(f"out_of_scope_dispositions count must be 21,896, found {len(disp_pairs)}")

    if not multicore_2022_found:
        raise ValueError("multicore placeholder disposition with reviewed_against '#2022' not found")

    return records, dispositions


def to_toml_string(s: str) -> str:
    res = []
    for char in s:
        if char == "\\":
            res.append("\\\\")
        elif char == '"':
            res.append('\\"')
        elif char == "\b":
            res.append("\\b")
        elif char == "\f":
            res.append("\\f")
        elif char == "\n":
            res.append("\\n")
        elif char == "\r":
            res.append("\\r")
        elif char == "\t":
            res.append("\\t")
        elif ord(char) < 0x20 or ord(char) == 0x7F:
            res.append(f"\\u{ord(char):04x}")
        else:
            res.append(char)
    return '"' + "".join(res) + '"'


def generate_manifest_toml(
    inventory_sha256: str,
    records: list[dict],
    platform: str,
    mamba_git_sha: str,
    mamba_binary_sha256: str,
    generated_at: str,
) -> str:
    lines = [
        "schema_version = 1",
        'inventory_path = "apps/mamba/tests/governance/gates/t1_oracle_hierarchy_inventory/cases.jsonl"',
        f"inventory_sha256 = {to_toml_string(inventory_sha256)}",
        "row_count = 8",
        'required_dimensions = ["behavior", "stability", "efficiency"]',
        'required_channels = ["compile", "behavior", "concurrency", "performance"]',
        'source_sets = ["ordinary_parity_corpus", "tier1_ec_cases", "tier1_gate_denominators"]',
        "migration_inputs = [",
        '    "apps/mamba/tests/harness/cpython/config/type_divergences.txt",',
        '    "apps/mamba/tests/harness/cpython/config/behavior_gaps.txt",',
        '    "apps/mamba/tests/harness/cpython/config/manifests/",',
        '    "apps/mamba/tests/governance/gates/",',
        '    "apps/mamba/external-contracts/"',
        "]",
        'cpython312_executable = "/Users/chrischeng/.pyenv/versions/3.12.11/bin/python3.12"',
        'cpython312_version = "Python 3.12"',
        'cpython313t_executable = "/Users/chrischeng/.local/bin/python3.13t"',
        'cpython313t_version = "Python 3.13"',
        f"platform = {to_toml_string(platform)}",
        f"mamba_git_sha = {to_toml_string(mamba_git_sha)}",
        f"mamba_binary_sha256 = {to_toml_string(mamba_binary_sha256)}",
        f"generated_at = {to_toml_string(generated_at)}",
    ]

    for rec in records:
        lines.append("")
        lines.append("[[source_set_records]]")
        lines.append(f"name = {to_toml_string(rec['name'])}")
        lines.append("paths = [")
        for i, p in enumerate(rec["paths"]):
            comma = "," if i < len(rec["paths"]) - 1 else ""
            lines.append(f"    {to_toml_string(p)}{comma}")
        lines.append("]")
        lines.append(f"identity_count = {rec['identity_count']}")
        lines.append(f"sha256_digest = {to_toml_string(rec['sha256_digest'])}")

    return "\n".join(lines) + "\n"


def generate_evidence_json(
    inventory_sha256: str,
    manifest_sha256: str,
    records: list[dict],
    dispositions: list[dict],
    platform: str,
    mamba_git_sha: str,
    mamba_binary_sha256: str,
    capture_timestamp: str,
) -> str:
    obj = {
        "schema_version": 1,
        "inventory_path": "apps/mamba/tests/governance/gates/t1_oracle_hierarchy_inventory/cases.jsonl",
        "inventory_sha256": inventory_sha256,
        "manifest_path": "apps/mamba/tests/governance/gates/t1_oracle_hierarchy_inventory/manifest.toml",
        "manifest_sha256": manifest_sha256,
        "evidence_path": "apps/mamba/external-contracts/evidence/mamba-t1-oracle-hierarchy-lock.json",
        "row_count": 8,
        "required_dimensions": ["behavior", "stability", "efficiency"],
        "required_channels": ["compile", "behavior", "concurrency", "performance"],
        "source_sets": ["ordinary_parity_corpus", "tier1_ec_cases", "tier1_gate_denominators"],
        "migration_inputs": [
            "apps/mamba/tests/harness/cpython/config/type_divergences.txt",
            "apps/mamba/tests/harness/cpython/config/behavior_gaps.txt",
            "apps/mamba/tests/harness/cpython/config/manifests/",
            "apps/mamba/tests/governance/gates/",
            "apps/mamba/external-contracts/",
        ],
        "cpython312_executable": "/Users/chrischeng/.pyenv/versions/3.12.11/bin/python3.12",
        "cpython312_version": "Python 3.12",
        "cpython313t_executable": "/Users/chrischeng/.local/bin/python3.13t",
        "cpython313t_version": "Python 3.13",
        "platform": platform,
        "mamba_git_sha": mamba_git_sha,
        "mamba_binary_sha256": mamba_binary_sha256,
        "generated_at": capture_timestamp,
        "verifier_command": "cargo test -p mamba --release --test mamba_core_semantics_ec -- oracle_hierarchy_and_result_identity --exact",
        "source_revision": mamba_git_sha,
        "capture_timestamp": capture_timestamp,
        "source_set_records": records,
        "out_of_scope_dispositions": dispositions,
    }

    return json.dumps(obj, indent=2, sort_keys=True) + "\n"


def main():
    parser = argparse.ArgumentParser(
        description="Generate deterministic manifest TOML and evidence JSON lock for Mamba Tier 1 oracle hierarchy."
    )
    parser.add_argument("--cases-jsonl", required=True, help="Path to cases JSONL file")
    parser.add_argument("--source-sets-json", required=True, help="Path to source-sets JSON file")
    parser.add_argument("--mamba-git-sha", required=True, help="40 lowercase hex Mamba git SHA")
    parser.add_argument("--mamba-binary-sha256", required=True, help="64 lowercase hex Mamba binary SHA256")
    parser.add_argument("--capture-timestamp", required=True, help="RFC3339 UTC capture timestamp")
    parser.add_argument("--platform", required=True, help="Target platform token")
    parser.add_argument("--manifest-output", required=True, help="Output path for generated manifest TOML")
    parser.add_argument("--evidence-output", required=True, help="Output path for generated evidence JSON")
    args = parser.parse_args()

    repo_root = find_repo_root()
    validate_args(args)

    cases_path = pathlib.Path(args.cases_jsonl).resolve()
    source_sets_path = pathlib.Path(args.source_sets_json).resolve()

    cases_bytes, inventory_sha256, rows, represented_pairs = validate_and_parse_cases(cases_path, args)
    records, dispositions = validate_and_parse_source_sets(source_sets_path, represented_pairs)

    manifest_str = generate_manifest_toml(
        inventory_sha256=inventory_sha256,
        records=records,
        platform=args.platform,
        mamba_git_sha=args.mamba_git_sha,
        mamba_binary_sha256=args.mamba_binary_sha256,
        generated_at=args.capture_timestamp,
    )
    manifest_bytes = manifest_str.encode("utf-8")
    manifest_sha256 = hashlib.sha256(manifest_bytes).hexdigest()

    evidence_str = generate_evidence_json(
        inventory_sha256=inventory_sha256,
        manifest_sha256=manifest_sha256,
        records=records,
        dispositions=dispositions,
        platform=args.platform,
        mamba_git_sha=args.mamba_git_sha,
        mamba_binary_sha256=args.mamba_binary_sha256,
        capture_timestamp=args.capture_timestamp,
    )
    evidence_bytes = evidence_str.encode("utf-8")

    manifest_output_path = pathlib.Path(args.manifest_output).resolve()
    evidence_output_path = pathlib.Path(args.evidence_output).resolve()

    manifest_output_path.parent.mkdir(parents=True, exist_ok=True)
    evidence_output_path.parent.mkdir(parents=True, exist_ok=True)

    tmp_manifest = manifest_output_path.with_name(f".{manifest_output_path.name}.tmp.{os.getpid()}")
    tmp_evidence = evidence_output_path.with_name(f".{evidence_output_path.name}.tmp.{os.getpid()}")

    try:
        with open(tmp_manifest, "wb") as f:
            f.write(manifest_bytes)
            f.flush()
            os.fsync(f.fileno())

        with open(tmp_evidence, "wb") as f:
            f.write(evidence_bytes)
            f.flush()
            os.fsync(f.fileno())

        tmp_manifest.replace(manifest_output_path)
        tmp_evidence.replace(evidence_output_path)
    except Exception:
        if tmp_manifest.exists():
            try:
                tmp_manifest.unlink()
            except OSError:
                pass
        if tmp_evidence.exists():
            try:
                tmp_evidence.unlink()
            except OSError:
                pass
        raise


if __name__ == "__main__":
    main()
