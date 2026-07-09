#!/usr/bin/env python3
"""Polyglot integration test (#106): a Python producer + worker drive loom using
*only* HTTP + stdlib — no bespoke SDK, no third-party deps. Proves loom's core
claim: any language can participate over plain HTTP.

- producer: PUTs inputs to keep (claim-check), submits a DAG to the loom
  controller, polls /runs/{id} to completion, and reads results from keep.
- worker (consumer): leases tasks from a relay subject, fetches each task's input
  from keep, runs a handler keyed by task_name (logic loom never sees), writes
  the result to keep, publishes a completion to the loom completions subject, and
  acks relay. Run with several worker threads to exercise the work queue.

Endpoints come from env (set by scripts/integration-polyglot.sh):
  LOOM (controller), RELAY, KEEP. Assumes LOOM_COMPLETION_SHARDS=1 so completions
  go to the single `loom.completions` subject (no shard hashing needed here).
"""
import json
import os
import sys
import threading
import time
import urllib.error
import urllib.request

LOOM = os.environ["LOOM"]
RELAY = os.environ["RELAY"]
KEEP = os.environ["KEEP"]
WORKER_SUBJECT = "resident"  # default runner route the controller dispatches to
COMPLETIONS = "loom.completions"


# ---- tiny HTTP helpers (stdlib only) ---------------------------------------
def _req(method, url, body=None, json_body=None, ctype=None):
    data = None
    headers = {}
    if json_body is not None:
        data = json.dumps(json_body).encode()
        headers["content-type"] = "application/json"
    elif body is not None:
        data = body if isinstance(body, bytes) else body.encode()
        headers["content-type"] = ctype or "application/octet-stream"
    r = urllib.request.Request(url, data=data, headers=headers, method=method)
    try:
        with urllib.request.urlopen(r, timeout=15) as resp:
            return resp.status, resp.read()
    except urllib.error.HTTPError as e:
        return e.code, e.read()


def post_json(url, obj):
    s, b = _req("POST", url, json_body=obj)
    return s, (json.loads(b) if b else {})


def get_bytes(url):
    s, b = _req("GET", url)
    return s, b


def put_bytes(url, data):
    s, _ = _req("PUT", url, body=data)
    return s


# ---- keep (claim-check store) ----------------------------------------------
def keep_put_input(key, data):
    assert put_bytes(f"{KEEP}/v1/inputs/{key}", data) == 200, f"keep PUT input {key}"


def keep_get_input(key):
    # A claim-check ref names either a producer input or an upstream node's
    # result — try inputs, then fall back to results (inter-node data flow).
    s, b = get_bytes(f"{KEEP}/v1/inputs/{key}")
    if s == 200:
        return b
    s, b = get_bytes(f"{KEEP}/v1/results/{key}")
    return b if s == 200 else b""


def keep_put_result(key, data):
    assert put_bytes(f"{KEEP}/v1/results/{key}", data) == 200, f"keep PUT result {key}"


def keep_get_result(key):
    s, b = get_bytes(f"{KEEP}/v1/results/{key}")
    return b if s == 200 else None


# ---- the Python worker: task logic loom never sees -------------------------
HANDLERS = {
    "upper": lambda b: b.upper(),
    "exclaim": lambda b: b + b"!",
    "reverse": lambda b: b[::-1],
}

_stop = threading.Event()


def worker_loop(worker_id):
    while not _stop.is_set():
        _, body = post_json(f"{RELAY}/v1/{WORKER_SUBJECT}/lease", {"consumer_id": worker_id})
        lease, entry = body.get("lease"), body.get("entry")
        if not lease or not entry:
            time.sleep(0.05)
            continue
        msg = entry["payload"]  # the loom TaskMessage
        run, node, attempt = msg["run_id"], msg["node_id"], msg["attempt"]
        refs = msg.get("input_refs") or []
        inp = keep_get_input(refs[0]) if refs else b""
        handler = HANDLERS.get(msg["task_name"])
        result_key = f"{run}:{node}:result"
        if handler is None:
            _report(run, node, attempt, None, failed=True)
        else:
            keep_put_result(result_key, handler(inp))
            _report(run, node, attempt, result_key, failed=False)
        # ack the lease
        post_json(f"{RELAY}/v1/{WORKER_SUBJECT}/ack",
                  {"lease_id": lease["lease_id"], "epoch": lease["epoch"]})


def _report(run, node, attempt, result_key, failed):
    # Completions are published to RELAY (the broker); the loom controller leases
    # `loom.completions` from relay and folds them — loom has no publish endpoint.
    post_json(f"{RELAY}/v1/{COMPLETIONS}/publish", {
        "message_id": f"{run}:{node}:{attempt}:done",
        "payload": {
            "run_id": run, "node_id": node, "attempt": attempt,
            "result_ref": result_key, "failed": failed, "fan_out": [],
        },
    })


# ---- the Python producer ----------------------------------------------------
def submit(run_id, nodes):
    s, body = post_json(f"{LOOM}/runs", {"run_id": run_id, "nodes": nodes})
    assert s in (200, 201), f"submit {run_id} -> {s} {body}"


def wait_succeeded(run_id, timeout=30):
    deadline = time.time() + timeout
    while time.time() < deadline:
        s, body = _req("GET", f"{LOOM}/runs/{run_id}")
        st = json.loads(body).get("status") if s == 200 else None
        if st == "succeeded":
            return True
        if st == "failed":
            return False
        time.sleep(0.2)
    return False


# ---- the scenarios ----------------------------------------------------------
def test_chain():
    """up(upper) -> bang(exclaim): data flows upstream-result -> downstream-input
    through keep (claim-check)."""
    run = "py-chain"
    keep_put_input("greeting", b"hello world")
    submit(run, [
        {"id": "up", "task_name": "upper", "input_refs": ["greeting"]},
        {"id": "bang", "task_name": "exclaim", "deps": ["up"],
         "input_refs": [f"{run}:up:result"]},
    ])
    assert wait_succeeded(run), "chain run did not succeed"
    out = keep_get_result(f"{run}:bang:result")
    assert out == b"HELLO WORLD!", f"chain result = {out!r}"
    print("  [chain]  up->bang  =>", (out or b"").decode())


def test_fanout():
    """3 independent upper nodes run in parallel across the worker pool; the run
    succeeds only when all leaves complete (fan-in barrier)."""
    run = "py-fanout"
    words = {"a": b"alpha", "b": b"beta", "c": b"gamma"}
    nodes = []
    for nid, w in words.items():
        keep_put_input(f"w-{nid}", w)
        nodes.append({"id": nid, "task_name": "upper", "input_refs": [f"w-{nid}"]})
    submit(run, nodes)
    assert wait_succeeded(run), "fan-out run did not succeed"
    got = {nid: keep_get_result(f"{run}:{nid}:result") for nid in words}
    for nid, w in words.items():
        assert got[nid] == w.upper(), f"fanout {nid} = {got[nid]!r}"
    print("  [fanout] 3 parallel upper =>", {k: v.decode() for k, v in got.items()})


def main():
    workers = [threading.Thread(target=worker_loop, args=(f"py-w{i}",), daemon=True)
               for i in range(3)]
    for w in workers:
        w.start()
    print("started 3 Python workers (stdlib HTTP, no SDK)")
    try:
        test_chain()
        test_fanout()
    finally:
        _stop.set()
    print("PASS: Python producer + workers drove loom over plain HTTP")


if __name__ == "__main__":
    try:
        main()
    except AssertionError as e:
        print(f"FAIL: {e}", file=sys.stderr)
        sys.exit(1)
