#!/usr/bin/env python3.12
"""Deterministic authoritative source-set generator for Mamba Tier 1 oracle hierarchy."""

import argparse
import hashlib
import json
import os
import pathlib
import sys
import tomllib


def find_repo_root() -> pathlib.Path:
    current = pathlib.Path.cwd().resolve()
    while current != current.parent:
        if (current / "projects/mamba").is_dir():
            return current
        current = current.parent
    file_dir = pathlib.Path(__file__).resolve().parent
    while file_dir != file_dir.parent:
        if (file_dir / "projects/mamba").is_dir():
            return file_dir
        file_dir = file_dir.parent
    raise RuntimeError("Could not determine repository root containing projects/mamba")


def compute_length_framed_set_digest(repo_root: pathlib.Path, paths: list[str]) -> str:
    hasher = hashlib.sha256()
    for p in paths:
        abs_p = repo_root / p
        if abs_p.is_file():
            bytes_data = abs_p.read_bytes()
        else:
            bytes_data = b"<missing>"
        p_bytes = p.encode("utf-8")
        hasher.update(len(p_bytes).to_bytes(8, byteorder="big", signed=False))
        hasher.update(p_bytes)
        hasher.update(len(bytes_data).to_bytes(8, byteorder="big", signed=False))
        hasher.update(bytes_data)
    return hasher.hexdigest()


def parse_exact_manifest_case_identity(source_id: str) -> tuple[str, str]:
    case_delim = "#case="
    if source_id.count(case_delim) != 1:
        raise ValueError(f"manifest identity must contain exactly one '{case_delim}' delimiter: '{source_id}'")
    idx = source_id.find(case_delim)
    manifest_path = source_id[:idx]
    case_name = source_id[idx + len(case_delim):]

    if "#" in manifest_path or "#" in case_name:
        raise ValueError(f"manifest identity parts must not contain '#': '{source_id}'")
    if not manifest_path.endswith(".toml"):
        raise ValueError(f"manifest identity path must end with '.toml': '{manifest_path}'")
    if "\\" in manifest_path:
        raise ValueError(f"manifest path in identity must not contain backslashes: '{manifest_path}'")
    if manifest_path.startswith("/"):
        raise ValueError(f"manifest path in identity must be relative, got absolute: '{manifest_path}'")

    for seg in manifest_path.split("/"):
        if not seg or seg == "." or seg == "..":
            raise ValueError(f"manifest path in identity contains invalid path segment ('', '.', or '..'): '{manifest_path}'")

    if not case_name or case_name != case_name.strip():
        raise ValueError(f"manifest case name must not be empty or have surrounding whitespace: '{case_name}'")
    if "/" in case_name or "\\" in case_name:
        raise ValueError(f"manifest case name must not contain path separators: '{case_name}'")

    return manifest_path, case_name


def validate_denominator_manifest(
    manifest_toml: dict,
    dir_name: str,
    parsed_identities_len: int,
    actual_sha256: str,
) -> None:
    family = manifest_toml.get("family")
    if not family or not isinstance(family, str):
        raise ValueError(f"manifest in '{dir_name}' missing required field 'family'")
    if family != dir_name:
        raise ValueError(f"manifest family '{family}' does not match directory name '{dir_name}'")

    cap = manifest_toml.get("capability")
    if not cap or not isinstance(cap, str):
        raise ValueError(f"manifest in '{dir_name}' missing required field 'capability'")
    if cap != "mamba-core-semantics":
        raise ValueError(f"manifest capability '{cap}' is not 'mamba-core-semantics' in '{dir_name}'")

    manifest_row_count = manifest_toml.get("row_count")
    if manifest_row_count is None or not isinstance(manifest_row_count, int):
        raise ValueError(f"manifest in '{dir_name}' missing required field 'row_count'")

    expected_sha256 = manifest_toml.get("denominator_sha256")
    if not expected_sha256 or not isinstance(expected_sha256, str):
        raise ValueError(f"manifest in '{dir_name}' missing required field 'denominator_sha256'")

    if actual_sha256 != expected_sha256:
        raise ValueError(
            f"denominator sha256 mismatch in '{dir_name}': expected {expected_sha256}, found {actual_sha256}"
        )

    if parsed_identities_len != manifest_row_count:
        raise ValueError(
            f"row_count {manifest_row_count} in manifest '{dir_name}' does not match parsed identities count {parsed_identities_len}"
        )


def discover_authoritative_source_sets(repo_root: pathlib.Path) -> dict:
    source_sets = {}

    # 1. ordinary_parity_corpus
    ordinary_manifest_paths = []
    manifests_dir = repo_root / "apps/mamba/tests/harness/cpython/config/manifests"
    if manifests_dir.is_dir():
        for p in manifests_dir.rglob("*.toml"):
            ordinary_manifest_paths.append(p.relative_to(repo_root).as_posix())
    ordinary_manifest_paths.sort()

    ordinary_paths = list(ordinary_manifest_paths)
    ordinary_identities_set = set()

    for path_str in ordinary_manifest_paths:
        abs_path = repo_root / path_str
        with open(abs_path, "rb") as f:
            toml_val = tomllib.load(f)

        cases = toml_val.get("case") or toml_val.get("cases")
        if not cases or not isinstance(cases, list) or len(cases) == 0:
            raise ValueError(f"manifest '{path_str}' missing or empty 'case' array")

        for c in cases:
            if not isinstance(c, dict):
                raise ValueError(f"item in 'case' array of '{path_str}' is not a table")
            case_name = c.get("case")
            if not case_name or not isinstance(case_name, str) or not case_name.strip():
                raise ValueError(f"case in '{path_str}' missing or empty string field 'case'")

            source_id = f"{path_str}#case={case_name}"
            parse_exact_manifest_case_identity(source_id)

            if source_id in ordinary_identities_set:
                raise ValueError(f"duplicate manifest case identity '{source_id}' in manifest '{path_str}'")
            ordinary_identities_set.add(source_id)

    gaps_path = repo_root / "apps/mamba/tests/harness/cpython/config/behavior_gaps.txt"
    if gaps_path.is_file():
        ordinary_paths.append(gaps_path.relative_to(repo_root).as_posix())
        for line in gaps_path.read_text(encoding="utf-8").splitlines():
            trimmed = line.strip()
            if trimmed and not trimmed.startswith("#"):
                ordinary_identities_set.add(trimmed)

    div_path = repo_root / "apps/mamba/tests/harness/cpython/config/type_divergences.txt"
    if div_path.is_file():
        ordinary_paths.append(div_path.relative_to(repo_root).as_posix())
        for line in div_path.read_text(encoding="utf-8").splitlines():
            trimmed = line.strip()
            if trimmed and not trimmed.startswith("#"):
                ordinary_identities_set.add(trimmed)

    ordinary_paths = sorted(list(set(ordinary_paths)))
    ordinary_digest = compute_length_framed_set_digest(repo_root, ordinary_paths)
    ordinary_identities_sorted = sorted(list(ordinary_identities_set))

    source_sets["ordinary_parity_corpus"] = {
        "name": "ordinary_parity_corpus",
        "paths": ordinary_paths,
        "identity_count": len(ordinary_identities_sorted),
        "sha256_digest": ordinary_digest,
        "source_identities": ordinary_identities_sorted,
    }

    # 2. tier1_ec_cases
    ec_paths = []
    ec_identities_set = set()
    ec_dirs = [
        repo_root / "apps/mamba/external-contracts/behavior",
        repo_root / "apps/mamba/external-contracts/stability",
        repo_root / "apps/mamba/external-contracts/efficiency",
    ]

    for d in ec_dirs:
        if d.is_dir():
            for p in sorted(d.iterdir(), key=lambda x: x.name):
                if p.is_file() and p.suffix == ".md":
                    content = p.read_text(encoding="utf-8")
                    in_yaml = False
                    yaml_lines = []
                    found_cap = False
                    for line in content.splitlines():
                        trimmed = line.strip()
                        if trimmed.startswith("```yaml"):
                            in_yaml = True
                            yaml_lines = []
                        elif in_yaml and trimmed.startswith("```"):
                            in_yaml = False
                            block = "\n".join(yaml_lines)
                            if "e2e_tests:" in block:
                                current_id = ""
                                current_cap = ""
                                for yline in block.splitlines():
                                    ytrim = yline.strip()
                                    if ytrim.startswith("- id:") or ytrim.startswith("id:"):
                                        if current_id and current_cap == "mamba-core-semantics":
                                            ec_identities_set.add(current_id)
                                            found_cap = True
                                        current_id = ytrim.split(":", 1)[1].strip().strip('"').strip("'")
                                        current_cap = ""
                                    elif ytrim.startswith("capability_id:"):
                                        current_cap = ytrim.split(":", 1)[1].strip().strip('"').strip("'")
                                if current_id and current_cap == "mamba-core-semantics":
                                    ec_identities_set.add(current_id)
                                    found_cap = True
                        elif in_yaml:
                            yaml_lines.append(line)
                    if found_cap:
                        ec_paths.append(p.relative_to(repo_root).as_posix())

    ec_paths = sorted(list(set(ec_paths)))
    ec_digest = compute_length_framed_set_digest(repo_root, ec_paths)
    ec_identities_sorted = sorted(list(ec_identities_set))

    source_sets["tier1_ec_cases"] = {
        "name": "tier1_ec_cases",
        "paths": ec_paths,
        "identity_count": len(ec_identities_sorted),
        "sha256_digest": ec_digest,
        "source_identities": ec_identities_sorted,
    }

    # 3. tier1_gate_denominators
    denom_paths = []
    denom_identities_set = set()
    gates_dir = repo_root / "apps/mamba/tests/governance/gates"

    if gates_dir.is_dir():
        for entry in sorted(gates_dir.iterdir(), key=lambda x: x.name):
            dir_name = entry.name
            if entry.is_dir() and dir_name.startswith("t1_") and dir_name.endswith("_denominator"):
                manifest_path = entry / "manifest.toml"
                denom_txt = entry / "denominator.txt"
                manifest_rel = f"apps/mamba/tests/governance/gates/{dir_name}/manifest.toml"
                denom_rel = f"apps/mamba/tests/governance/gates/{dir_name}/denominator.txt"

                denom_paths.append(manifest_rel)
                denom_paths.append(denom_rel)

                if manifest_path.is_file():
                    with open(manifest_path, "rb") as f:
                        manifest_toml = tomllib.load(f)

                    if not denom_txt.is_file():
                        raise RuntimeError(
                            f"denominator gate directory '{entry.as_posix()}' has manifest.toml but is missing denominator.txt"
                        )

                    denom_bytes = denom_txt.read_bytes()
                    actual_sha = hashlib.sha256(denom_bytes).hexdigest()

                    denom_raw = denom_bytes.decode("utf-8", errors="replace")
                    parsed_identities = [
                        l.strip()
                        for l in denom_raw.splitlines()
                        if l.strip() and not l.strip().startswith("#")
                    ]

                    validate_denominator_manifest(
                        manifest_toml,
                        dir_name,
                        len(parsed_identities),
                        actual_sha,
                    )

                    if not parsed_identities:
                        denom_identities_set.add(denom_rel)
                    else:
                        for pid in parsed_identities:
                            denom_identities_set.add(pid)
                else:
                    if denom_txt.is_file():
                        denom_bytes = denom_txt.read_bytes()
                        denom_raw = denom_bytes.decode("utf-8", errors="replace")
                        parsed_identities = [
                            l.strip()
                            for l in denom_raw.splitlines()
                            if l.strip() and not l.strip().startswith("#")
                        ]
                        if not parsed_identities:
                            denom_identities_set.add(denom_rel)
                        else:
                            for pid in parsed_identities:
                                denom_identities_set.add(pid)
                    else:
                        denom_identities_set.add(denom_rel)

    denom_paths = sorted(list(set(denom_paths)))
    denom_digest = compute_length_framed_set_digest(repo_root, denom_paths)
    denom_identities_sorted = sorted(list(denom_identities_set))

    source_sets["tier1_gate_denominators"] = {
        "name": "tier1_gate_denominators",
        "paths": denom_paths,
        "identity_count": len(denom_identities_sorted),
        "sha256_digest": denom_digest,
        "source_identities": denom_identities_sorted,
    }

    return source_sets


def main():
    parser = argparse.ArgumentParser(description="Generate oracle hierarchy source sets.")
    parser.add_argument("--output", required=True, help="Output JSON path")
    parser.add_argument("--represented-jsonl", required=False, help="Optional represented JSONL path")
    args = parser.parse_args()

    repo_root = find_repo_root()
    discovered = discover_authoritative_source_sets(repo_root)

    represented_pairs = set()
    if args.represented_jsonl:
        rep_path = pathlib.Path(args.represented_jsonl).resolve()
        if not rep_path.is_file():
            raise FileNotFoundError(f"represented JSONL file not found: {args.represented_jsonl}")
        for line in rep_path.read_text(encoding="utf-8").splitlines():
            trimmed = line.strip()
            if not trimmed:
                continue
            row = json.loads(trimmed)
            sset = row.get("source_set")
            sid = row.get("source_identity")
            if not sset or not sid:
                raise ValueError(f"JSONL row missing source_set or source_identity: {trimmed}")
            pair = (sset, sid)
            if pair in represented_pairs:
                raise ValueError(f"duplicate represented pair ({sset}, {sid}) in JSONL")
            if sset not in discovered or sid not in discovered[sset]["source_identities"]:
                raise ValueError(f"represented pair ({sset}, {sid}) was not discovered")
            represented_pairs.add(pair)

    source_set_names = ["ordinary_parity_corpus", "tier1_ec_cases", "tier1_gate_denominators"]
    source_set_records = []
    source_identities_obj = {}
    dispositions = []

    for name in source_set_names:
        info = discovered[name]
        source_set_records.append({
            "name": info["name"],
            "paths": info["paths"],
            "identity_count": info["identity_count"],
            "sha256_digest": info["sha256_digest"],
        })
        source_identities_obj[name] = info["source_identities"]

        for identity in info["source_identities"]:
            if (name, identity) in represented_pairs:
                continue
            reviewed_against = "#2022" if identity == "apps/mamba/tests/governance/gates/t1_multicore_scaling_denominator/denominator.txt" else "#2010"
            reason = f"{name} identity is outside the bounded eight-row oracle sample"
            dispositions.append({
                "source_set": name,
                "source_identity": identity,
                "reason": reason,
                "reviewed_against": reviewed_against,
            })

    dispositions.sort(key=lambda d: (d["source_set"], d["source_identity"]))

    output_data = {
        "source_set_records": source_set_records,
        "source_identities": source_identities_obj,
        "out_of_scope_dispositions": dispositions,
    }

    out_path = pathlib.Path(args.output).resolve()
    out_path.parent.mkdir(parents=True, exist_ok=True)
    tmp_path = out_path.with_name(f".{out_path.name}.tmp.{os.getpid()}")

    try:
        with open(tmp_path, "w", encoding="utf-8") as f:
            json.dump(output_data, f, indent=2)
            f.write("\n")
        tmp_path.replace(out_path)
    except Exception:
        if tmp_path.exists():
            tmp_path.unlink()
        raise


if __name__ == "__main__":
    main()
