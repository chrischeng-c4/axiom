# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "real_world"
# case = "config_model_roundtrip"
# subject = "dataclasses"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dataclasses.py"
# status = "filled"
# ///
"""dataclasses: an end-user config-model flow: define a nested @dataclass schema with field defaults/default_factory, construct instances, serialize via asdict/astuple, immutably update via replace, and compare via the synthesized __eq__"""
import dataclasses


# A realistic nested config model: a service config that owns a retry policy.
@dataclasses.dataclass
class RetryPolicy:
    attempts: int = 3
    backoff: float = 0.5


@dataclasses.dataclass
class ServiceConfig:
    name: str
    host: str = "localhost"
    port: int = dataclasses.field(default=8080)
    tags: list = dataclasses.field(default_factory=list)
    retry: RetryPolicy = dataclasses.field(default_factory=RetryPolicy)


# 1. Construct from defaults + overrides.
cfg = ServiceConfig("api")
assert cfg.name == "api", f"name = {cfg.name!r}"
assert cfg.host == "localhost", f"host = {cfg.host!r}"
assert cfg.port == 8080, f"port = {cfg.port!r}"
assert cfg.tags == [], f"tags = {cfg.tags!r}"
assert cfg.retry == RetryPolicy(3, 0.5), f"retry = {cfg.retry!r}"

# 2. default_factory builds an independent retry policy per instance.
other = ServiceConfig("worker")
cfg.tags.append("prod")
assert other.tags == [], f"independent tags: {other.tags!r}"
assert other.retry is not cfg.retry, "independent retry instances"

# 3. Serialize the whole nested model via asdict (recurses into RetryPolicy).
d = dataclasses.asdict(cfg)
assert d == {
    "name": "api",
    "host": "localhost",
    "port": 8080,
    "tags": ["prod"],
    "retry": {"attempts": 3, "backoff": 0.5},
}, f"asdict = {d!r}"

# 4. astuple flattens the model field-order (nested model becomes a tuple).
t = dataclasses.astuple(cfg)
assert t == ("api", "localhost", 8080, ["prod"], (3, 0.5)), f"astuple = {t!r}"

# 5. Immutable update: replace() yields a new config; original untouched.
prod = dataclasses.replace(cfg, host="prod.example.com", port=443)
assert prod.host == "prod.example.com", f"replaced host = {prod.host!r}"
assert prod.port == 443, f"replaced port = {prod.port!r}"
assert cfg.host == "localhost", "original host unchanged"
assert cfg.port == 8080, "original port unchanged"

# 6. Synthesized __eq__ compares field-wise across the model.
assert ServiceConfig("api", tags=["prod"]) == cfg, "equal configs compare equal"
assert prod != cfg, "differing configs compare unequal"

print("config_model_roundtrip OK")
