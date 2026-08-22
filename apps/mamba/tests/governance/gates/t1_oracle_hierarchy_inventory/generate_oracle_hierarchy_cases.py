#!/usr/bin/env python3.12
"""Deterministic eight-row executable cases producer for Mamba Tier 1 oracle hierarchy."""

import argparse
import datetime
import json
import os
import pathlib
import shlex
import sys

# Ensure sibling directory is in python path
file_dir = pathlib.Path(__file__).resolve().parent
if str(file_dir) not in sys.path:
    sys.path.insert(0, str(file_dir))

import generate_oracle_hierarchy_source_sets


def find_repo_root() -> pathlib.Path:
    fdir = pathlib.Path(__file__).resolve().parent
    while fdir != fdir.parent:
        if (fdir / "projects/mamba").is_dir() and (
            (fdir / ".git").exists() or (fdir / "aw.toml").exists()
        ):
            return fdir
        fdir = fdir.parent
    raise RuntimeError(
        "Could not determine repository root containing projects/mamba and (.git or aw.toml)"
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
        raise ValueError("platform token must be non-empty string")


def generate_rows(repo_root: pathlib.Path, git_sha: str, binary_sha: str, platform: str) -> list[dict]:
    diag_file = (
        repo_root
        / "apps/mamba/tests/governance/gates/t1_oracle_hierarchy_inventory/oracles/force_typed_expected_diagnostics.json"
    )
    if not diag_file.is_file():
        raise FileNotFoundError(f"Missing force_typed_expected_diagnostics.json at {diag_file}")

    verify_diag_script = (
        "apps/mamba/tests/governance/gates/t1_oracle_hierarchy_inventory/oracles/verify_force_typed_expected_diagnostic.py"
    )
    verify_diag_path = repo_root / verify_diag_script
    if not verify_diag_path.is_file():
        raise FileNotFoundError(f"Missing verify_force_typed_expected_diagnostic.py at {verify_diag_path}")

    with open(diag_file, "r", encoding="utf-8") as f:
        diag_data = json.load(f)

    cpython312_bin = "/Users/chrischeng/.pyenv/versions/3.12.11/bin/python3.12"
    cpython313t_bin = "/Users/chrischeng/.local/bin/python3.13t"

    raw_rows = [
        # 1. Behavior green
        {
            "case_id": "mamba-t1-to-thread-gather-results",
            "probe_id": "mamba-t1-to-thread-gather-results",
            "source_identity": "mamba-t1-to-thread-gather-results",
            "source_set": "tier1_ec_cases",
            "tier1_dimension": "behavior",
            "channel": "behavior",
            "expected_result_channel": "behavior",
            "fixture_or_probe_path": "apps/mamba/tests/governance/gates/t1_oracle_hierarchy_inventory/probes/to_thread_gather_behavior_green.py",
            "oracle_kind": "cpython312_identity",
            "oracle_executable": cpython312_bin,
            "oracle_version": "Python 3.12",
            "oracle_command": f"{cpython312_bin} apps/mamba/tests/governance/gates/t1_oracle_hierarchy_inventory/probes/to_thread_gather_behavior_green.py",
            "sut_command": "target/release/mamba run apps/mamba/tests/governance/gates/t1_oracle_hierarchy_inventory/probes/to_thread_gather_behavior_green.py",
            "expected_outcome_kind": "ok",
            "expected_divergence_class": "none",
            "expected_probe_anchor": "ROUND_OK",
            "expected_terminal_classification": "green",
            "sample_role": "green",
        },
        # 2. Stability green
        {
            "case_id": "mamba-t1-to-thread-gather-stability",
            "probe_id": "mamba-t1-to-thread-gather-stability-green",
            "source_identity": "mamba-t1-to-thread-gather-stability",
            "source_set": "tier1_ec_cases",
            "tier1_dimension": "stability",
            "channel": "concurrency",
            "expected_result_channel": "concurrency",
            "fixture_or_probe_path": "apps/mamba/tests/governance/gates/t1_oracle_hierarchy_inventory/probes/to_thread_gather_stability_green.py",
            "oracle_kind": "property",
            "oracle_executable": cpython313t_bin,
            "oracle_version": "Python 3.13",
            "oracle_command": f"{cpython313t_bin} apps/mamba/tests/governance/gates/t1_oracle_hierarchy_inventory/probes/to_thread_gather_stability_green.py",
            "sut_command": "target/release/mamba run apps/mamba/tests/governance/gates/t1_oracle_hierarchy_inventory/probes/to_thread_gather_stability_green.py",
            "expected_outcome_kind": "ok",
            "expected_divergence_class": "none",
            "expected_probe_anchor": "MAMBA-T1-FT-GATHER-STABILITY",
            "expected_terminal_classification": "green",
            "sample_role": "green",
        },
        # 3. Efficiency green
        {
            "case_id": "mamba-t1-to-thread-gather-efficiency",
            "probe_id": "mamba-t1-to-thread-gather-efficiency-green",
            "source_identity": "mamba-t1-to-thread-gather-efficiency",
            "source_set": "tier1_ec_cases",
            "tier1_dimension": "efficiency",
            "channel": "performance",
            "expected_result_channel": "performance",
            "fixture_or_probe_path": "apps/mamba/tests/governance/gates/t1_oracle_hierarchy_inventory/probes/to_thread_gather_efficiency_green.py",
            "oracle_kind": "property",
            "oracle_executable": cpython313t_bin,
            "oracle_version": "Python 3.13",
            "oracle_command": f"{cpython313t_bin} apps/mamba/tests/governance/gates/t1_oracle_hierarchy_inventory/probes/to_thread_gather_efficiency_green.py",
            "sut_command": "target/release/mamba run apps/mamba/tests/governance/gates/t1_oracle_hierarchy_inventory/probes/to_thread_gather_efficiency_green.py",
            "expected_outcome_kind": "ok",
            "expected_divergence_class": "none",
            "expected_probe_anchor": "MAMBA-T1-FT-GATHER-EFFICIENCY",
            "expected_terminal_classification": "green",
            "sample_role": "green",
        },
        # 4. Behavior intentional-red (ord)
        {
            "case_id": "builtin_test__test_ord",
            "probe_id": "ord(42)",
            "source_identity": "apps/mamba/tests/cpython/behavior/builtin-libs/builtins/builtin_test__test_ord.py",
            "fixture_or_probe_path": "apps/mamba/tests/cpython/behavior/builtin-libs/builtins/builtin_test__test_ord.py",
            "source_set": "ordinary_parity_corpus",
            "tier1_dimension": "behavior",
            "channel": "compile",
            "expected_result_channel": "compile",
            "oracle_kind": "force_typed_expected",
            "oracle_executable": cpython312_bin,
            "oracle_version": "Python 3.12",
            "oracle_command": f"{cpython312_bin} {verify_diag_script} apps/mamba/tests/cpython/behavior/builtin-libs/builtins/builtin_test__test_ord.py",
            "sut_command": "target/release/mamba check apps/mamba/tests/cpython/behavior/builtin-libs/builtins/builtin_test__test_ord.py",
            "expected_outcome_kind": "compile_error",
            "expected_divergence_class": "force_typed_compile_reject",
            "expected_terminal_classification": "intentional_red",
            "sample_role": "intentional_red",
            "diagnostic_class": diag_data["apps/mamba/tests/cpython/behavior/builtin-libs/builtins/builtin_test__test_ord.py"]["diagnostic_class"],
            "diagnostic_span": diag_data["apps/mamba/tests/cpython/behavior/builtin-libs/builtins/builtin_test__test_ord.py"]["diagnostic_span"],
            "expected_probe_anchor": diag_data["apps/mamba/tests/cpython/behavior/builtin-libs/builtins/builtin_test__test_ord.py"]["message_anchor"],
        },
        # 5. Behavior intentional-red (input)
        {
            "case_id": "builtin_test__test_input",
            "probe_id": "input(42,42)",
            "source_identity": "apps/mamba/tests/cpython/behavior/builtin-libs/builtins/builtin_test__test_input.py",
            "fixture_or_probe_path": "apps/mamba/tests/cpython/behavior/builtin-libs/builtins/builtin_test__test_input.py",
            "source_set": "ordinary_parity_corpus",
            "tier1_dimension": "behavior",
            "channel": "compile",
            "expected_result_channel": "compile",
            "oracle_kind": "force_typed_expected",
            "oracle_executable": cpython312_bin,
            "oracle_version": "Python 3.12",
            "oracle_command": f"{cpython312_bin} {verify_diag_script} apps/mamba/tests/cpython/behavior/builtin-libs/builtins/builtin_test__test_input.py",
            "sut_command": "target/release/mamba check apps/mamba/tests/cpython/behavior/builtin-libs/builtins/builtin_test__test_input.py",
            "expected_outcome_kind": "compile_error",
            "expected_divergence_class": "force_typed_compile_reject",
            "expected_terminal_classification": "intentional_red",
            "sample_role": "intentional_red",
            "diagnostic_class": diag_data["apps/mamba/tests/cpython/behavior/builtin-libs/builtins/builtin_test__test_input.py"]["diagnostic_class"],
            "diagnostic_span": diag_data["apps/mamba/tests/cpython/behavior/builtin-libs/builtins/builtin_test__test_input.py"]["diagnostic_span"],
            "expected_probe_anchor": diag_data["apps/mamba/tests/cpython/behavior/builtin-libs/builtins/builtin_test__test_input.py"]["message_anchor"],
        },
        # 6. Behavior intentional-red (invalid context)
        {
            "case_id": "py_context_input_validation__test_invalid_context",
            "probe_id": "getattr(c,9)",
            "source_identity": "apps/mamba/tests/cpython/behavior/std-libs/decimal/py_context_input_validation__test_invalid_context.py",
            "fixture_or_probe_path": "apps/mamba/tests/cpython/behavior/std-libs/decimal/py_context_input_validation__test_invalid_context.py",
            "source_set": "ordinary_parity_corpus",
            "tier1_dimension": "behavior",
            "channel": "compile",
            "expected_result_channel": "compile",
            "oracle_kind": "force_typed_expected",
            "oracle_executable": cpython312_bin,
            "oracle_version": "Python 3.12",
            "oracle_command": f"{cpython312_bin} {verify_diag_script} apps/mamba/tests/cpython/behavior/std-libs/decimal/py_context_input_validation__test_invalid_context.py",
            "sut_command": "target/release/mamba check apps/mamba/tests/cpython/behavior/std-libs/decimal/py_context_input_validation__test_invalid_context.py",
            "expected_outcome_kind": "compile_error",
            "expected_divergence_class": "force_typed_compile_reject",
            "expected_terminal_classification": "intentional_red",
            "sample_role": "intentional_red",
            "diagnostic_class": diag_data["apps/mamba/tests/cpython/behavior/std-libs/decimal/py_context_input_validation__test_invalid_context.py"]["diagnostic_class"],
            "diagnostic_span": diag_data["apps/mamba/tests/cpython/behavior/std-libs/decimal/py_context_input_validation__test_invalid_context.py"]["diagnostic_span"],
            "expected_probe_anchor": diag_data["apps/mamba/tests/cpython/behavior/std-libs/decimal/py_context_input_validation__test_invalid_context.py"]["message_anchor"],
        },
        # 7. Stability intentional-red
        {
            "case_id": "mamba-t1-to-thread-gather-stability-intentional-red",
            "probe_id": "mamba-t1-to-thread-gather-stability-red",
            "source_identity": "cpython_ported::gen::behavior::std_libs::threading::test_gen_behavior_std_libs_threading_is_alive_lifecycle",
            "source_set": "tier1_gate_denominators",
            "tier1_dimension": "stability",
            "channel": "concurrency",
            "expected_result_channel": "concurrency",
            "fixture_or_probe_path": "apps/mamba/tests/governance/gates/t1_oracle_hierarchy_inventory/probes/to_thread_gather_stability_intentional_red.py",
            "oracle_kind": "property",
            "oracle_executable": cpython313t_bin,
            "oracle_version": "Python 3.13",
            "oracle_command": f"{cpython313t_bin} apps/mamba/tests/governance/gates/t1_oracle_hierarchy_inventory/probes/to_thread_gather_stability_intentional_red.py",
            "sut_command": "target/release/mamba run apps/mamba/tests/governance/gates/t1_oracle_hierarchy_inventory/probes/to_thread_gather_stability_intentional_red.py",
            "expected_outcome_kind": "property_red",
            "expected_divergence_class": "thread_quiescence_bound",
            "expected_probe_anchor": "MAMBA-T1-FT-GATHER-STABILITY-RED",
            "expected_terminal_classification": "intentional_red",
            "sample_role": "intentional_red",
        },
        # 8. Efficiency intentional-red
        {
            "case_id": "mamba-t1-to-thread-gather-efficiency-intentional-red",
            "probe_id": "mamba-t1-to-thread-gather-efficiency-red",
            "source_identity": "cpython_ported::gen::behavior::std_libs::concurrent_futures::test_gen_behavior_std_libs_concurrent_futures_thread_pool_runs_all_tasks",
            "source_set": "tier1_gate_denominators",
            "tier1_dimension": "efficiency",
            "channel": "performance",
            "expected_result_channel": "performance",
            "fixture_or_probe_path": "apps/mamba/tests/governance/gates/t1_oracle_hierarchy_inventory/probes/to_thread_gather_efficiency_intentional_red.py",
            "oracle_kind": "property",
            "oracle_executable": cpython313t_bin,
            "oracle_version": "Python 3.13",
            "oracle_command": f"{cpython313t_bin} apps/mamba/tests/governance/gates/t1_oracle_hierarchy_inventory/probes/to_thread_gather_efficiency_intentional_red.py",
            "sut_command": "target/release/mamba run apps/mamba/tests/governance/gates/t1_oracle_hierarchy_inventory/probes/to_thread_gather_efficiency_intentional_red.py",
            "expected_outcome_kind": "property_red",
            "expected_divergence_class": "perf_bound",
            "expected_probe_anchor": "MAMBA-T1-FT-GATHER-EFFICIENCY-RED",
            "expected_terminal_classification": "intentional_red",
            "sample_role": "intentional_red",
        },
    ]

    for row in raw_rows:
        row["mamba_git_sha"] = git_sha
        row["mamba_binary_sha256"] = binary_sha
        row["platform"] = platform

    return raw_rows


def validate_rows(repo_root: pathlib.Path, rows: list[dict], discovered_sets: dict) -> None:
    if len(rows) != 8:
        raise ValueError(f"Expected exactly 8 rows, got {len(rows)}")

    case_ids = set()
    probe_ids = set()
    discovered_pairs = set()

    dim_class_counts = {"behavior": set(), "stability": set(), "efficiency": set()}
    channels = set()

    shell_forbidden_chars = set("|&;<>$`(){}#\n\r")

    for row in rows:
        cid = row["case_id"]
        pid = row["probe_id"]
        if cid in case_ids:
            raise ValueError(f"Duplicate case_id: {cid}")
        case_ids.add(cid)
        if pid in probe_ids:
            raise ValueError(f"Duplicate probe_id: {pid}")
        probe_ids.add(pid)

        # Require fixture/probe path to be regular file
        fpath = repo_root / row["fixture_or_probe_path"]
        if not fpath.is_file():
            raise FileNotFoundError(f"Fixture/probe path does not exist as regular file: {fpath}")

        # Require oracle_executable to be regular file
        oexec = pathlib.Path(row["oracle_executable"])
        if not oexec.is_file():
            raise FileNotFoundError(f"Oracle executable does not exist as regular file: {oexec}")

        # Validate SUT and oracle commands with shlex.split and check for forbidden shell characters
        sut_cmd = row["sut_command"]
        oracle_cmd = row["oracle_command"]
        fix_path = row["fixture_or_probe_path"]

        if any(c in shell_forbidden_chars for c in sut_cmd):
            raise ValueError(f"SUT command for case '{cid}' contains forbidden shell characters: '{sut_cmd}'")
        if any(c in shell_forbidden_chars for c in oracle_cmd):
            raise ValueError(f"Oracle command for case '{cid}' contains forbidden shell characters: '{oracle_cmd}'")

        try:
            sut_tokens = shlex.split(sut_cmd)
        except Exception as e:
            raise ValueError(f"SUT command for case '{cid}' failed shlex parse: {e}")

        try:
            oracle_tokens = shlex.split(oracle_cmd)
        except Exception as e:
            raise ValueError(f"Oracle command for case '{cid}' failed shlex parse: {e}")

        if len(sut_tokens) < 3:
            raise ValueError(f"SUT command for case '{cid}' has too few tokens: {sut_tokens}")

        if sut_tokens[0] != "target/release/mamba":
            raise ValueError(f"SUT argv[0] for case '{cid}' is '{sut_tokens[0]}', expected 'target/release/mamba'")

        expected_subcmd = "check" if row.get("oracle_kind") == "force_typed_expected" else "run"
        if sut_tokens[1] != expected_subcmd:
            raise ValueError(
                f"SUT argv[1] for case '{cid}' is '{sut_tokens[1]}', expected '{expected_subcmd}'"
            )

        if fix_path not in sut_tokens:
            raise ValueError(f"Fixture/probe path '{fix_path}' is not an exact token in SUT command tokens {sut_tokens}")

        if fix_path not in oracle_tokens:
            raise ValueError(f"Fixture/probe path '{fix_path}' is not an exact token in oracle command tokens {oracle_tokens}")

        # Require source_set and source_identity to be discovered
        sset = row["source_set"]
        sid = row["source_identity"]
        if sset not in discovered_sets or sid not in discovered_sets[sset]["source_identities"]:
            raise ValueError(f"Row pair ({sset}, {sid}) for case '{cid}' was not discovered in source sets")
        discovered_pairs.add((sset, sid))

        dim = row["tier1_dimension"]
        if dim not in dim_class_counts:
            raise ValueError(f"Unknown tier1_dimension: {dim}")
        term_cls = row["expected_terminal_classification"]
        dim_class_counts[dim].add(term_cls)

        ch = row["channel"]
        channels.add(ch)

        if ch == "compile":
            diag_cls = row.get("diagnostic_class")
            diag_span = row.get("diagnostic_span")
            if not diag_cls or not str(diag_cls).strip():
                raise ValueError(f"Compile row '{cid}' missing or empty diagnostic_class")
            if not diag_span or not str(diag_span).strip():
                raise ValueError(f"Compile row '{cid}' missing or empty diagnostic_span")
        else:
            if "diagnostic_class" in row or "diagnostic_span" in row:
                raise ValueError(f"Non-compile row '{cid}' has diagnostic fields")

        if row["sample_role"] != row["expected_terminal_classification"]:
            raise ValueError(f"sample_role != expected_terminal_classification for case '{cid}'")
        if row["expected_result_channel"] != row["channel"]:
            raise ValueError(f"expected_result_channel != channel for case '{cid}'")

    for dim, classes in dim_class_counts.items():
        if classes != {"green", "intentional_red"}:
            raise ValueError(f"Dimension '{dim}' does not contain both green and intentional_red: {classes}")

    expected_channels = {"compile", "behavior", "concurrency", "performance"}
    if channels != expected_channels:
        raise ValueError(f"Channels mismatch: expected {expected_channels}, got {channels}")

    if len(discovered_pairs) != 8:
        raise ValueError(
            f"Expected exactly 8 distinct (source_set, source_identity) pairs, got {len(discovered_pairs)}: {discovered_pairs}"
        )


def write_output_atomically(rows: list[dict], output_path_str: str) -> None:
    out_path = pathlib.Path(output_path_str).resolve()
    out_path.parent.mkdir(parents=True, exist_ok=True)
    tmp_path = out_path.with_name(f".{out_path.name}.tmp.{os.getpid()}")

    try:
        with open(tmp_path, "w", encoding="utf-8") as f:
            for row in rows:
                json_str = json.dumps(row, sort_keys=True)
                f.write(json_str + "\n")
        tmp_path.replace(out_path)
    except Exception:
        if tmp_path.exists():
            tmp_path.unlink()
        raise


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Deterministic eight-row executable cases producer for Mamba Tier 1 oracle hierarchy."
    )
    parser.add_argument("--mamba-git-sha", required=True, help="40 lowercase hex git SHA")
    parser.add_argument("--mamba-binary-sha256", required=True, help="64 lowercase hex binary SHA256")
    parser.add_argument("--capture-timestamp", required=True, help="Explicit RFC3339 UTC timestamp")
    parser.add_argument("--platform", required=True, help="Explicit platform token")
    parser.add_argument("--output", required=True, help="Output JSONL path")
    args = parser.parse_args()

    validate_args(args)

    repo_root = find_repo_root()
    discovered_sets = generate_oracle_hierarchy_source_sets.discover_authoritative_source_sets(repo_root)

    rows = generate_rows(repo_root, args.mamba_git_sha, args.mamba_binary_sha256, args.platform)
    validate_rows(repo_root, rows, discovered_sets)

    write_output_atomically(rows, args.output)


if __name__ == "__main__":
    main()
