#!/usr/bin/env python3
"""Fail-closed verifier for sixteen Lumen durable-performance receipts.

This program verifies only the measurements in receipt schema v1.  It never
contacts a registry, a workflow service, or a running Lumen process.
"""
import argparse
import hashlib
import json
import os
import pathlib
import re
import stat
import sys
import tempfile


SCHEMA_VERSION = 1
CELL_KIND = "lumen.durable-perf-cell"
SUMMARY_KIND = "lumen.durable-perf-summary"
IMAGE_RE = re.compile(r"^[^@\s]+@sha256:[0-9a-f]{64}$")
IMAGE_ID_RE = re.compile(r"^sha256:[0-9a-f]{64}$")
EXPECTED_CELLS = frozenset(
    f"{endpoint}-{batch}-{backend}"
    for endpoint, batches in (("index", (1, 100, 1000)),
                              ("unindex", (1, 100, 1000)),
                              ("replace", (1, 32)))
    for batch in batches
    for backend in ("flat-cpu", "hnsw-cpu")
)
REQUIRED_LIMITS = {
    "input_seconds": 1800,
    "docops_per_second": 100,
    "query_qps": 10,
    "completed_percent": 95,
    "query_p99_limit_ms": 1000,
    "query_max_limit_ms": 5000,
    "drain_limit_ms": 60000,
    "rss_limit_bytes": 12884901888,
    "docker_cpus_milli": 2500,
    "docker_memory_bytes": 17179869184,
    "snapshot_seconds": 15,
    "hot_documents": 500000,
    "idle_collections": 181,
    "idle_documents_per_collection": 100,
    "field_count": 14,
    "seed": 4775511124152791525,
    "ngram_text_fields": 3,
    "vector_dimensions": 8,
}
MEASUREMENT_INTS = frozenset((
    "input_duration_ms", "observed_input_elapsed_ms", "requests_offered", "requests_finished",
    "requests_completed", "requests_started_in_input", "requests_finished_in_input",
    "requests_completed_in_input", "requests_started_per_second_milli",
    "requests_completed_per_second_milli", "request_errors", "client_cancellations",
    "items_offered", "items_completed", "items_failed", "items_started_in_input",
    "items_completed_in_input", "items_started_per_second_milli",
    "items_completed_per_second_milli", "docops_offered",
    "docops_completed", "docops_offered_in_input", "docops_completed_in_input",
    "index_requests_started_in_input", "replace_requests_started_in_input",
    "unindex_requests_started_in_input",
    "docops_offered_per_second_milli", "docops_completed_per_second_milli",
    "docops_completion_percent_milli", "request_latency_p99_ms",
    "request_latency_max_ms", "queries_offered", "queries_completed",
    "queries_started_in_input", "queries_completed_in_input",
    "hot_queries_started_in_input", "idle_queries_started_in_input",
    "queries_started_per_second_milli", "queries_completed_per_second_milli",
    "query_errors_or_timeouts", "query_latency_p99_ms", "query_latency_max_ms",
    "request_drain_ms", "query_drain_ms", "checkpoint_delta", "merge_delta",
    "checkpoint_bytes", "merge_read_bytes", "merge_write_bytes",
    "capture_hold_ns_total", "pending_delta_bytes", "pending_delta_layers",
    "backpressure_events", "segment_disk_bytes", "peak_rss_bytes",
    "restart_duration_ms",
))
MEASUREMENT_KEYS = MEASUREMENT_INTS | {
    "restart_recovered", "live_mutation_readback", "cold_mutation_readback",
}


def fail(message):
    raise ValueError(message)


def exact_keys(value, keys, name):
    if not isinstance(value, dict):
        fail(f"{name} must be an object")
    expected = set(keys)
    actual = set(value)
    if actual != expected:
        fail(f"{name} keys differ: missing={sorted(expected - actual)!r} extra={sorted(actual - expected)!r}")


def require_string(value, name):
    if not isinstance(value, str) or not value:
        fail(f"{name} must be a non-empty string")
    return value


def require_u64(value, name):
    if isinstance(value, bool) or not isinstance(value, int) or value < 0 or value > 0xffffffffffffffff:
        fail(f"{name} must be an unsigned 64-bit integer")
    return value


def require_bool(value, name):
    if type(value) is not bool:
        fail(f"{name} must be a boolean")
    return value


def no_duplicate_object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            fail(f"duplicate JSON key {key!r}")
        result[key] = value
    return result


def load_receipt(path):
    try:
        raw = path.read_bytes()
    except OSError as error:
        fail(f"cannot read receipt {path.name!r}: {error}")
    try:
        value = json.loads(raw.decode("utf-8"), object_pairs_hook=no_duplicate_object,
                            parse_constant=lambda value: fail(f"invalid JSON number {value!r}"))
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        fail(f"invalid JSON receipt {path.name!r}: {error}")
    return raw, value


def validate_cell(value, expected_binding):
    exact_keys(value, {"schema_version", "kind", "binding", "cell", "limits", "measurement", "outcome"}, "receipt")
    if require_u64(value["schema_version"], "schema_version") != SCHEMA_VERSION:
        fail("unsupported receipt schema_version")
    if value["kind"] != CELL_KIND:
        fail("unsupported receipt kind")

    binding = value["binding"]
    binding_keys = {"repository", "run_id", "run_attempt", "commit", "image_reference", "actual_image_id"}
    exact_keys(binding, binding_keys, "binding")
    for key in binding_keys:
        require_string(binding[key], f"binding.{key}")
    if not IMAGE_RE.fullmatch(binding["image_reference"]):
        fail("binding.image_reference must be digest-pinned")
    if not IMAGE_ID_RE.fullmatch(binding["actual_image_id"]):
        fail("binding.actual_image_id must be an immutable sha256")
    if binding != expected_binding:
        fail("receipt binding does not match command-line binding")

    cell = value["cell"]
    exact_keys(cell, {"id", "endpoint", "batch_size", "backend"}, "cell")
    cell_id = require_string(cell["id"], "cell.id")
    endpoint = require_string(cell["endpoint"], "cell.endpoint")
    backend = require_string(cell["backend"], "cell.backend")
    batch = require_u64(cell["batch_size"], "cell.batch_size")
    allowed_batches = {"index": {1, 100, 1000}, "unindex": {1, 100, 1000}, "replace": {1, 32}}
    if endpoint not in allowed_batches or batch not in allowed_batches[endpoint] or backend not in {"flat-cpu", "hnsw-cpu"}:
        fail(f"unsupported cell {cell_id!r}")
    if cell_id != f"{endpoint}-{batch}-{backend}":
        fail("cell.id does not agree with cell fields")

    limits = value["limits"]
    exact_keys(limits, REQUIRED_LIMITS, "limits")
    for key, required in REQUIRED_LIMITS.items():
        if require_u64(limits[key], f"limits.{key}") != required:
            fail(f"limits.{key} does not match frozen durable gate")

    measurement = value["measurement"]
    exact_keys(measurement, MEASUREMENT_KEYS, "measurement")
    for key in MEASUREMENT_INTS:
        require_u64(measurement[key], f"measurement.{key}")
    require_bool(measurement["restart_recovered"], "measurement.restart_recovered")
    require_bool(measurement["live_mutation_readback"], "measurement.live_mutation_readback")
    require_bool(measurement["cold_mutation_readback"], "measurement.cold_mutation_readback")
    validate_measurement(measurement)

    outcome = value["outcome"]
    exact_keys(outcome, {"qualifying", "diagnostic", "succeeded"}, "outcome")
    if (require_bool(outcome["qualifying"], "outcome.qualifying") is not True or
            require_bool(outcome["diagnostic"], "outcome.diagnostic") is not False or
            require_bool(outcome["succeeded"], "outcome.succeeded") is not True):
        fail("receipt outcome is not a successful qualifying measurement")
    return cell_id


def rate(count, duration_ms):
    return (count * 1000000) // duration_ms


def validate_measurement(m):
    duration = m["input_duration_ms"]
    if duration != 1800000:
        fail("input duration must be the fixed 1800-second membership window")
    if m["observed_input_elapsed_ms"] < duration:
        fail("observed input elapsed time did not reach the membership window")
    if m["docops_offered_in_input"] < 180000:
        fail("input docops offered is below 180000")
    if m["docops_completed_in_input"] * 100 < m["docops_offered_in_input"] * 95:
        fail("input docops completion is below 95 percent")
    if m["queries_started_in_input"] < 18000 or m["queries_completed_in_input"] < 18000:
        fail("input query count is below 18000")
    if m["requests_finished"] != m["requests_offered"] or m["requests_completed"] != m["requests_offered"]:
        fail("request count conservation failed")
    if (m["requests_started_in_input"] == 0 or
            m["requests_started_in_input"] > m["requests_offered"] or
            m["requests_finished_in_input"] > m["requests_finished"] or
            m["requests_completed_in_input"] > m["requests_completed"] or
            m["requests_finished_in_input"] > m["requests_started_in_input"] or
            m["requests_completed_in_input"] > m["requests_finished_in_input"]):
        fail("in-window request count conservation failed")
    if m["items_completed"] + m["items_failed"] != m["items_offered"]:
        fail("item count conservation failed")
    if (m["items_started_in_input"] == 0 or
            m["items_started_in_input"] > m["items_offered"] or
            m["items_completed_in_input"] > m["items_completed"] or
            m["items_completed_in_input"] > m["items_started_in_input"]):
        fail("in-window item count conservation failed")
    if (m["docops_completed"] != m["docops_offered"] or
            m["docops_offered_in_input"] > m["docops_offered"] or
            m["docops_completed_in_input"] > m["docops_completed"] or
            m["docops_completed_in_input"] > m["docops_offered_in_input"] or
            m["docops_offered"] > m["items_offered"] or
            m["docops_completed"] > m["items_completed"]):
        fail("docops count conservation failed")
    if (m["queries_completed"] != m["queries_offered"] or
            m["queries_started_in_input"] > m["queries_offered"] or
            m["queries_completed_in_input"] > m["queries_completed"] or
            m["queries_completed_in_input"] > m["queries_started_in_input"]):
        fail("query count conservation failed")
    if any(m[key] != 0 for key in ("request_errors", "client_cancellations", "items_failed", "query_errors_or_timeouts")):
        fail("receipt reports request, item, or query errors")
    checks = (("requests_started_per_second_milli", "requests_started_in_input"),
              ("requests_completed_per_second_milli", "requests_completed_in_input"),
              ("items_started_per_second_milli", "items_started_in_input"),
              ("items_completed_per_second_milli", "items_completed_in_input"),
              ("docops_offered_per_second_milli", "docops_offered_in_input"),
              ("docops_completed_per_second_milli", "docops_completed_in_input"),
              ("queries_started_per_second_milli", "queries_started_in_input"),
              ("queries_completed_per_second_milli", "queries_completed_in_input"))
    for stored, count in checks:
        if m[stored] != rate(m[count], duration):
            fail(f"{stored} does not match the measured input rate")
    if (m["docops_offered_per_second_milli"] < 100000 or
            m["docops_completed_per_second_milli"] < 95000 or
            m["queries_started_per_second_milli"] < 10000 or
            m["queries_completed_per_second_milli"] < 10000):
        fail("measured input rate is below the frozen durable gate")
    if m["docops_completion_percent_milli"] != (m["docops_completed_in_input"] * 100000) // m["docops_offered_in_input"]:
        fail("docops completion percent does not match counts")
    mutation_request_starts = (m["index_requests_started_in_input"] +
                               m["replace_requests_started_in_input"] +
                               m["unindex_requests_started_in_input"])
    if (m["index_requests_started_in_input"] == 0 or
            m["replace_requests_started_in_input"] == 0 or
            m["unindex_requests_started_in_input"] == 0 or
            mutation_request_starts != m["requests_started_in_input"]):
        fail("each mutation endpoint must contribute actual in-window request bodies")
    if (m["hot_queries_started_in_input"] == 0 or
            m["idle_queries_started_in_input"] == 0 or
            m["hot_queries_started_in_input"] + m["idle_queries_started_in_input"] != m["queries_started_in_input"]):
        fail("hot and idle queries must both contribute to the combined in-window QPS")
    if (m["request_latency_p99_ms"] > m["request_latency_max_ms"] or
            m["request_latency_p99_ms"] > 1000 or m["request_latency_max_ms"] > 5000):
        fail("request latency limit exceeded")
    if (m["query_latency_p99_ms"] > m["query_latency_max_ms"] or
            m["query_latency_p99_ms"] > 1000 or m["query_latency_max_ms"] > 5000):
        fail("query latency limit exceeded")
    if m["request_drain_ms"] > 60000 or m["query_drain_ms"] > 60000:
        fail("drain limit exceeded")
    if m["checkpoint_delta"] == 0 or m["merge_delta"] == 0:
        fail("checkpoint and merge are both required within the measurement window")
    if m["checkpoint_bytes"] == 0 or m["merge_read_bytes"] == 0 or m["merge_write_bytes"] == 0:
        fail("checkpoint and merge IO bytes are required")
    if m["segment_disk_bytes"] == 0:
        fail("segment disk bytes are required")
    if not 0 < m["peak_rss_bytes"] <= 12884901888:
        fail("peak RSS is absent or exceeds 12 GiB")
    if not m["restart_recovered"]:
        fail("restart recovery was not observed")
    if m["restart_duration_ms"] > 30000:
        fail("restart duration exceeds the approved 30000ms limit")
    if not m["live_mutation_readback"] or not m["cold_mutation_readback"]:
        fail("live and cold mutation target/content readback are required")


def receipt_paths(receipts_dir):
    root = os.path.abspath(receipts_dir)
    try:
        info = os.lstat(root)
    except OSError as error:
        fail(f"cannot inspect receipts directory: {error}")
    if stat.S_ISLNK(info.st_mode) or not stat.S_ISDIR(info.st_mode):
        fail("receipts path must be a real directory")
    paths = []
    try:
        entries = list(os.scandir(root))
    except OSError as error:
        fail(f"cannot list receipts directory: {error}")
    for entry in entries:
        if entry.name.startswith("."):
            continue
        try:
            info = entry.stat(follow_symlinks=False)
        except OSError as error:
            fail(f"cannot inspect receipt {entry.name!r}: {error}")
        if not stat.S_ISREG(info.st_mode) or not entry.name.endswith(".json"):
            fail(f"receipt directory contains unsafe or unknown entry {entry.name!r}")
        paths.append(os.path.join(root, entry.name))
    return [pathlib.Path(path) for path in sorted(paths)]


def validate_cli_binding(repo, run_id, run_attempt, commit, image):
    binding = {"repository": require_string(repo, "--repo"), "run_id": require_string(run_id, "--run-id"),
               "run_attempt": require_string(run_attempt, "--run-attempt"), "commit": require_string(commit, "--commit"),
               "image_reference": require_string(image, "--image"), "actual_image_id": None}
    if not re.fullmatch(r"[0-9a-f]{40}", binding["commit"]):
        fail("--commit must be a lowercase 40-hex commit")
    if not IMAGE_RE.fullmatch(binding["image_reference"]):
        fail("--image must be a digest-pinned image reference")
    return binding


def expected_summary(receipts_dir, repo, run_id, run_attempt, commit, image):
    paths = receipt_paths(receipts_dir)
    if len(paths) != 16:
        fail(f"expected exactly 16 receipt files, got {len(paths)}")
    expected = validate_cli_binding(repo, run_id, run_attempt, commit, image)
    records = []
    cell_ids = set()
    actual_image_id = None
    for path in paths:
        raw, receipt = load_receipt(path)
        if actual_image_id is None:
            actual_image_id = receipt.get("binding", {}).get("actual_image_id") if isinstance(receipt, dict) else None
        if actual_image_id is None:
            fail("first receipt has no actual image ID")
        expected["actual_image_id"] = actual_image_id
        cell_id = validate_cell(receipt, expected)
        if cell_id in cell_ids:
            fail(f"duplicate cell receipt {cell_id!r}")
        cell_ids.add(cell_id)
        records.append({"cell_id": cell_id, "file": path.name, "sha256": hashlib.sha256(raw).hexdigest()})
    if cell_ids != EXPECTED_CELLS:
        fail(f"cell coverage differs: missing={sorted(EXPECTED_CELLS - cell_ids)!r} extra={sorted(cell_ids - EXPECTED_CELLS)!r}")
    summary = {"schema_version": 1, "kind": SUMMARY_KIND, "binding": expected,
               "receipt_count": len(records), "receipts": sorted(records, key=lambda item: item["cell_id"])}
    return summary


def verify(receipts_dir, repo, run_id, run_attempt, commit, image, output):
    summary = expected_summary(receipts_dir, repo, run_id, run_attempt, commit, image)
    write_output(output, summary)
    return summary


def require_regular_path(path, name):
    try:
        info = os.lstat(path)
    except OSError as error:
        fail(f"cannot inspect {name}: {error}")
    if stat.S_ISLNK(info.st_mode) or not stat.S_ISREG(info.st_mode):
        fail(f"{name} must be a regular file")


def load_json_file(path, name):
    require_regular_path(path, name)
    try:
        raw = pathlib.Path(path).read_bytes()
        value = json.loads(raw.decode("utf-8"), object_pairs_hook=no_duplicate_object,
                           parse_constant=lambda value: fail(f"invalid JSON number {value!r}"))
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        fail(f"cannot read {name}: {error}")
    return raw, value


def verify_existing(receipts_dir, repo, run_id, run_attempt, commit, image, output, summary_sha256):
    if not re.fullmatch(r"[0-9a-f]{64}", summary_sha256):
        fail("--summary-sha256 must be 64 lowercase hexadecimal characters")
    expected = expected_summary(receipts_dir, repo, run_id, run_attempt, commit, image)
    raw, existing = load_json_file(output, "existing aggregate")
    actual_sha256 = hashlib.sha256(raw).hexdigest()
    if actual_sha256 != summary_sha256:
        fail("existing aggregate SHA-256 does not match --summary-sha256")
    sidecar = output + ".sha256"
    require_regular_path(sidecar, "existing aggregate sidecar")
    try:
        sidecar_bytes = pathlib.Path(sidecar).read_bytes()
    except OSError as error:
        fail(f"cannot read existing aggregate sidecar: {error}")
    if sidecar_bytes != (actual_sha256 + "\n").encode("ascii"):
        fail("existing aggregate sidecar does not bind its exact bytes")
    exact_keys(existing, {"schema_version", "kind", "binding", "receipt_count", "receipts"}, "existing aggregate")
    if existing != expected:
        fail("existing aggregate does not match validated raw receipts")
    return expected


def write_output(output, summary):
    target = os.path.abspath(output)
    parent = os.path.dirname(target)
    parent_info = os.lstat(parent)
    if stat.S_ISLNK(parent_info.st_mode) or not stat.S_ISDIR(parent_info.st_mode):
        fail("output parent must be a real directory")
    if os.path.lexists(target):
        ensure_existing_summary_matches(target, summary)
        raise FileExistsError(f"output already exists: {target}")
    if os.path.lexists(target + ".sha256"):
        fail("output sidecar already exists without a replaceable output")
    payload = (json.dumps(summary, sort_keys=True, separators=(",", ":")) + "\n").encode("utf-8")
    digest = hashlib.sha256(payload).hexdigest()
    create_once(parent, os.path.basename(target), payload)
    try:
        create_once(parent, os.path.basename(target) + ".sha256", (digest + "\n").encode("ascii"))
    except Exception:
        # The summary has already been created and remains valid.  Do not replace it.
        raise


def ensure_existing_summary_matches(path, summary):
    info = os.lstat(path)
    if stat.S_ISLNK(info.st_mode) or not stat.S_ISREG(info.st_mode):
        fail("existing output must be a regular file")
    try:
        with open(path, encoding="utf-8") as stream:
            previous = json.load(stream, object_pairs_hook=no_duplicate_object,
                                 parse_constant=lambda value: fail(f"invalid JSON number {value!r}"))
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        fail(f"cannot read existing aggregate: {error}")
    exact_keys(previous, {"schema_version", "kind", "binding", "receipt_count", "receipts"}, "existing aggregate")
    if previous["schema_version"] != 1 or previous["kind"] != SUMMARY_KIND:
        fail("existing aggregate has an unsupported schema")
    if previous["binding"] != summary["binding"]:
        fail("existing aggregate binding differs")
    if previous["receipt_count"] != 16 or previous["receipts"] != summary["receipts"]:
        fail("raw receipt hashes differ from the create-once aggregate")


def create_once(parent, name, payload):
    if not name or name in {".", ".."} or os.path.sep in name:
        fail("output must name a file in its parent directory")
    fd, temporary = tempfile.mkstemp(prefix=".verify-durable-perf-", dir=parent)
    try:
        with os.fdopen(fd, "wb") as stream:
            stream.write(payload)
            stream.flush()
            os.fsync(stream.fileno())
        try:
            os.link(temporary, os.path.join(parent, name))
        except FileExistsError:
            raise
        finally:
            os.unlink(temporary)
        directory_fd = os.open(parent, os.O_RDONLY)
        try:
            os.fsync(directory_fd)
        finally:
            os.close(directory_fd)
    except Exception:
        try:
            os.unlink(temporary)
        except FileNotFoundError:
            pass
        raise


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--receipts-dir", required=True)
    parser.add_argument("--repo", required=True)
    parser.add_argument("--run-id", required=True)
    parser.add_argument("--run-attempt", required=True)
    parser.add_argument("--commit", required=True)
    parser.add_argument("--image", required=True)
    parser.add_argument("--output", required=True)
    parser.add_argument("--verify-existing", action="store_true")
    parser.add_argument("--summary-sha256")
    args = parser.parse_args(argv)
    try:
        if args.verify_existing:
            if args.summary_sha256 is None:
                fail("--verify-existing requires --summary-sha256")
            verify_existing(args.receipts_dir, args.repo, args.run_id, args.run_attempt,
                            args.commit, args.image, args.output, args.summary_sha256)
        else:
            if args.summary_sha256 is not None:
                fail("--summary-sha256 requires --verify-existing")
            verify(args.receipts_dir, args.repo, args.run_id, args.run_attempt, args.commit, args.image, args.output)
    except (OSError, ValueError) as error:
        print(f"refused: {error}", file=sys.stderr)
        return 2
    return 0


if __name__ == "__main__":
    sys.exit(main())
