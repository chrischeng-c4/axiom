# test_configparser.py — #3408 axis-1 stdlib configparser AssertionPass seed.
#
# Mamba-authored seed exercising the `configparser` module surface called
# out in the issue:
#   ConfigParser read / get / set, sections, items, DEFAULT section,
#   interpolation.
#
# Surface coverage (asserts run at module scope; no helper closures per
# the mamba top-level def() quirk in test_math.py):
#   1. Module identity + public surface (hasattr).
#   2. ConfigParser.read_string() — INI text → in-memory config.
#   3. .sections()                — enumerate non-DEFAULT sections.
#   4. .has_section(), in operator on parser, in operator on section.
#   5. .get(section, key)         — value lookup; .options() listing.
#   6. .set(section, key, value)  — mutate then re-read.
#   7. .items(section)            — (key, value) pair listing including
#                                   DEFAULT keys merged into each section.
#   8. DEFAULTSECT contents available on every section.
#   9. Basic interpolation        — %(key)s expanded against DEFAULTs.
#
# Boxed-int dodge (subtraction-against-zero) applied to length checks
# per the boxed-accumulator equality bug.
#
# Contract with `cpython_lib_test_runner.rs`:
#   - Each `assert` runs at top level. Inverting any assert raises
#     AssertionError → non-zero exit → runner classifies as `Fail`.
#   - After every assertion has executed, the seed emits
#     `MAMBA_ASSERTION_PASS: configparser N asserts` to stdout.

import configparser

_ledger: list[int] = []

# 1. Module identity + public surface.
assert configparser.__name__ == "configparser", "configparser.__name__"
_ledger.append(1)
assert hasattr(configparser, "ConfigParser"), "exposes ConfigParser"
_ledger.append(1)
assert hasattr(configparser, "DEFAULTSECT"), "exposes DEFAULTSECT sentinel"
_ledger.append(1)
assert configparser.DEFAULTSECT == "DEFAULT", "DEFAULTSECT == 'DEFAULT'"
_ledger.append(1)
assert hasattr(configparser, "NoSectionError"), "exposes NoSectionError"
_ledger.append(1)
assert hasattr(configparser, "NoOptionError"), "exposes NoOptionError"
_ledger.append(1)

# 2. ConfigParser.read_string — populate from INI text.
_cp = configparser.ConfigParser()
_cp.read_string(
    "[DEFAULT]\n"
    "shared_key = shared_value\n"
    "host = example.com\n"
    "\n"
    "[server]\n"
    "port = 8080\n"
    "name = web\n"
    "\n"
    "[client]\n"
    "name = ui\n"
    "endpoint = https://%(host)s/api\n"
)
assert isinstance(_cp, configparser.ConfigParser), (
    "ConfigParser() constructs a ConfigParser instance"
)
_ledger.append(1)

# 3. .sections() — non-DEFAULT sections only.
_sections = _cp.sections()
assert isinstance(_sections, list), ".sections() returns a list"
_ledger.append(1)
assert len(_sections) - 2 == 0, ".sections() lists 2 sections (server, client)"
_ledger.append(1)
assert "server" in _sections, ".sections() includes 'server'"
_ledger.append(1)
assert "client" in _sections, ".sections() includes 'client'"
_ledger.append(1)
assert "DEFAULT" not in _sections, ".sections() excludes DEFAULT"
_ledger.append(1)

# 4. has_section / in-operator on parser / in-operator on section.
assert _cp.has_section("server") == True, "has_section('server') is True"
_ledger.append(1)
assert _cp.has_section("missing") == False, "has_section('missing') is False"
_ledger.append(1)
assert "server" in _cp, "in-operator on ConfigParser checks section presence"
_ledger.append(1)
assert "port" in _cp["server"], "in-operator on a section checks key presence"
_ledger.append(1)

# 5. .get(section, key) — value lookup; .options() listing.
assert _cp.get("server", "port") == "8080", "get('server','port') == '8080'"
_ledger.append(1)
assert _cp.get("server", "name") == "web", "get('server','name') == 'web'"
_ledger.append(1)
_options_server = _cp.options("server")
assert isinstance(_options_server, list), ".options() returns a list"
_ledger.append(1)
assert "port" in _options_server, ".options('server') includes 'port'"
_ledger.append(1)
assert "name" in _options_server, ".options('server') includes 'name'"
_ledger.append(1)
# DEFAULTs are merged into every section's options listing.
assert "shared_key" in _options_server, (
    ".options('server') merges DEFAULT keys (shared_key)"
)
_ledger.append(1)

# 6. .set(section, key, value) — mutate, then re-read.
_cp.set("server", "port", "9090")
assert _cp.get("server", "port") == "9090", (
    "set() then get() round-trips the new value"
)
_ledger.append(1)
_cp.set("server", "newkey", "newvalue")
assert _cp.get("server", "newkey") == "newvalue", (
    "set() of a brand-new key is visible via get()"
)
_ledger.append(1)

# 7. .items(section) — (key, value) pairs including DEFAULT merge.
_items_client = _cp.items("client")
assert isinstance(_items_client, list), ".items() returns a list"
_ledger.append(1)
# Each entry is a (key, value) tuple.
_items_first = _items_client[0]
assert isinstance(_items_first, tuple), ".items() entries are tuples"
_ledger.append(1)
assert len(_items_first) - 2 == 0, ".items() entries are 2-tuples"
_ledger.append(1)
# Build a dict-of-items for stable assertions.
_items_dict = dict(_items_client)
assert _items_dict["name"] == "ui", ".items() carries section-local key 'name'"
_ledger.append(1)
assert _items_dict["shared_key"] == "shared_value", (
    ".items('client') merges DEFAULT key 'shared_key'"
)
_ledger.append(1)

# 8. DEFAULTSECT — keys appear under every section's read view.
assert _cp.get("server", "shared_key") == "shared_value", (
    "DEFAULT key visible via get('server', 'shared_key')"
)
_ledger.append(1)
assert _cp.get("client", "shared_key") == "shared_value", (
    "DEFAULT key visible via get('client', 'shared_key')"
)
_ledger.append(1)
# Defaults dict accessor (.defaults()) returns the DEFAULT map.
_defaults = _cp.defaults()
assert "shared_key" in _defaults, ".defaults() exposes DEFAULT keys"
_ledger.append(1)
assert _defaults["host"] == "example.com", ".defaults()['host'] correct"
_ledger.append(1)

# 9. Basic interpolation — %(key)s expanded against DEFAULTs at .get() time.
assert _cp.get("client", "endpoint") == "https://example.com/api", (
    "interpolation: %(host)s expands to the DEFAULT 'host' value"
)
_ledger.append(1)
# raw=True disables interpolation.
assert _cp.get("client", "endpoint", raw=True) == "https://%(host)s/api", (
    "get(..., raw=True) returns the un-interpolated literal"
)
_ledger.append(1)

# Emit the proof-of-execution marker. Per `ASSERTION_PASS_MARKERS` in
# `cpython_lib_test_runner.rs`, presence of this token escalates the
# outcome from ImportPass to AssertionPass.
print(f"MAMBA_ASSERTION_PASS: configparser {len(_ledger)} asserts")
