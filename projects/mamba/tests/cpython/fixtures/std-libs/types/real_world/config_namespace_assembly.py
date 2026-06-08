# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "real_world"
# case = "config_namespace_assembly"
# subject = "types.SimpleNamespace"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
"""types.SimpleNamespace: a settings layer assembles a SimpleNamespace config from defaults plus overrides, validates each field against a runtime int|str union, mutates and snapshots it, then dynamically builds a record type with types.new_class — one realistic end-to-end flow over the types surface"""
import types

# 1. Defaults + overrides merge into a single config namespace.
defaults = {"host": "localhost", "port": 8080, "scheme": "http"}
overrides = {"port": 9090, "scheme": "https"}
merged = dict(defaults)
merged.update(overrides)
config = types.SimpleNamespace(**merged)
assert config.host == "localhost"
assert config.port == 9090
assert config.scheme == "https"
assert vars(config) == {"host": "localhost", "port": 9090, "scheme": "https"}

# 2. Validate every field against a runtime int|str union (tuple-of-types).
scalar = int | str
for name, value in vars(config).items():
    assert isinstance(value, scalar), (name, value)

# 3. Mutate then snapshot the namespace; the snapshot is decoupled by value.
config.timeout = 30
snapshot = types.SimpleNamespace(**vars(config))
config.timeout = 60
assert snapshot.timeout == 30
assert config.timeout == 60

# 4. repr renders the config in insertion order for logging.
view = types.SimpleNamespace(a=1, b="two")
assert repr(view) == "namespace(a=1, b='two')"

# 5. Dynamically build a record type whose namespace is populated by an
#    exec_body callback, then instantiate it from the config.
def record_body(ns):
    ns["origin"] = f"{config.scheme}://{config.host}:{config.port}"
    ns["describe"] = lambda self: self.origin

Endpoint = types.new_class("Endpoint", (), {}, record_body)
ep = Endpoint()
assert Endpoint.__bases__ == (object,)
assert ep.origin == "https://localhost:9090"
assert ep.describe() == "https://localhost:9090"

print("config_namespace_assembly OK")
