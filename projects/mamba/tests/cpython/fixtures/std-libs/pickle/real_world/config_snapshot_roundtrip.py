# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pickle"
# dimension = "real_world"
# case = "config_snapshot_roundtrip"
# subject = "pickle"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pickle.py"
# status = "filled"
# ///
"""pickle: a settings layer serializes a nested config dict (str keys; int/str/bool/list/tuple/nested-dict values) with pickle.dumps and reloads it with pickle.loads, asserting the reloaded snapshot equals the original and a derived aggregate is stable"""
import pickle

# A realistic application config snapshot: a settings layer that persists its
# state by pickling a nested dict and reloads it on the next run.
config = {
    "service": "api-gateway",
    "version": 3,
    "debug": False,
    "ports": [8080, 8443],
    "limits": {"max_conns": 1024, "timeout_s": 30, "retries": 5},
    "routes": [
        {"path": "/health", "weight": 1},
        {"path": "/v1/users", "weight": 10},
        {"path": "/v1/orders", "weight": 7},
    ],
    "region_quota": ("us-east", 500),
}

snapshot = pickle.dumps(config)
assert isinstance(snapshot, bytes), "snapshot is bytes"

restored = pickle.loads(snapshot)
assert restored == config, f"config snapshot round-trip mismatch: {restored!r}"
assert restored["limits"]["max_conns"] == 1024, "nested int preserved"
assert isinstance(restored["region_quota"], tuple), "tuple value stays a tuple"

# Derived aggregate: total route weight is stable across the round-trip.
weight = sum(r["weight"] for r in restored["routes"])
assert weight == 18, f"aggregate route weight = {weight}"

print("config_snapshot_roundtrip OK")
