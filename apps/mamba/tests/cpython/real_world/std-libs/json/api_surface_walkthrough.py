# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "json"
# dimension = "real_world"
# case = "api_surface_walkthrough"
# subject = "json"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/json/test_pass1.py"
# status = "filled"
# ///
"""json: a downstream consumer drives dumps/loads/JSONEncoder/sort_keys/indent/separators together over a realistic config record, asserting each result and a deterministic round-trip"""
import json


# A service config record with mixed scalars, a list, and a nested object.
config = {
    "name": "mamba-api",
    "version": 3,
    "enabled": True,
    "timeout": 5.0,
    "retries": None,
    "tags": ["prod", "us-east-1"],
    "limits": {"rps": 1000, "burst": 50},
}

# 1. dumps -> loads round-trips the whole record unchanged.
wire = json.dumps(config)
assert json.loads(wire) == config, "config round-trip"

# 2. sort_keys gives a deterministic, byte-stable serialization.
canon = json.dumps(config, sort_keys=True)
assert canon == json.dumps(config, sort_keys=True), "sort_keys deterministic"
assert canon.index('"enabled"') < canon.index('"name"'), "keys sorted ascending"

# 3. compact separators drop whitespace; still round-trips.
compact = json.dumps(config, separators=(",", ":"), sort_keys=True)
assert ", " not in compact and ": " not in compact, f"compact = {compact!r}"
assert json.loads(compact) == config, "compact round-trip"

# 4. indent pretty-prints for human-readable config files.
pretty = json.dumps(config, indent=2, sort_keys=True)
assert "\n" in pretty and "  " in pretty, "indent adds structure"
assert json.loads(pretty) == config, "pretty round-trip"

# 5. A custom encoder serializes a domain type the stdlib can't.
class ConfigEncoder(json.JSONEncoder):
    def default(self, obj):
        if isinstance(obj, frozenset):
            return sorted(obj)
        return super().default(obj)

extended = json.dumps({"perms": frozenset({"read", "write"})}, cls=ConfigEncoder)
assert json.loads(extended) == {"perms": ["read", "write"]}, extended

# 6. Parse an externally-supplied config blob and read nested fields.
blob = '{"service": {"port": 8080, "hosts": ["a", "b"]}, "debug": false}'
parsed = json.loads(blob)
assert parsed["service"]["port"] == 8080, parsed
assert parsed["service"]["hosts"] == ["a", "b"], parsed
assert parsed["debug"] is False, parsed

print("api_surface_walkthrough OK")
