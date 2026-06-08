# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "cpython321_builtins"
# dimension = "real_world"
# case = "test_plistlib"
# subject = "cpython321.test_plistlib"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_plistlib.py"
# status = "filled"
# ///
"""cpython321.test_plistlib: execute CPython 3.12 seed test_plistlib"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# test_plistlib.py — #3407 axis-1 stdlib plistlib AssertionPass seed.
#
# Mamba-authored seed exercising the `plistlib` module surface called
# out in the issue:
#   dumps/loads XML and BINARY, datetime, data, nested structure.
#
# Surface coverage (asserts run at module scope; no helper closures per
# the mamba top-level def() quirk in test_math.py):
#   1. Module identity + public surface (hasattr).
#   2. FMT_XML and FMT_BINARY format sentinels exposed.
#   3. dumps/loads round-trip with XML format on a nested payload
#      (dict + list + int + str + bool + bytes + datetime).
#   4. dumps/loads round-trip with BINARY format on the same payload.
#   5. XML output is an XML document (UTF-8 prefix, root element tag,
#      DOCTYPE marker).
#   6. BINARY output begins with the `bplist00` magic bytes.
#   7. Datetime preserved across both round-trips.
#
# Boxed-int dodge (subtraction-against-zero) applied for length checks.
#
# Contract with `cpython_lib_test_runner.rs`:
#   - Each `assert` runs at top level. Inverting any assert raises
#     AssertionError → non-zero exit → runner classifies as `Fail`.
#   - After every assertion has executed, the seed emits
#     `MAMBA_ASSERTION_PASS: plistlib N asserts` to stdout.

import plistlib
import datetime

_ledger: list[int] = []

# 1. Module identity + public surface.
assert plistlib.__name__ == "plistlib", "plistlib.__name__"
_ledger.append(1)
assert hasattr(plistlib, "dumps"), "exposes dumps"
_ledger.append(1)
assert hasattr(plistlib, "loads"), "exposes loads"
_ledger.append(1)
assert hasattr(plistlib, "dump"), "exposes dump"
_ledger.append(1)
assert hasattr(plistlib, "load"), "exposes load"
_ledger.append(1)
assert hasattr(plistlib, "FMT_XML"), "exposes FMT_XML"
_ledger.append(1)
assert hasattr(plistlib, "FMT_BINARY"), "exposes FMT_BINARY"
_ledger.append(1)

# 2. Format sentinels are distinct.
assert plistlib.FMT_XML != plistlib.FMT_BINARY, "FMT_XML and FMT_BINARY are distinct"
_ledger.append(1)

# 3. XML round-trip on a nested payload.
_payload = {
    "name": "mamba",
    "answer": 42,
    "active": True,
    "tags": ["alpha", "beta", "gamma"],
    "nested": {"depth": 2, "items": [1, 2, 3]},
    "blob": b"\x00\x01\x02hello",
    "created": datetime.datetime(2026, 5, 21, 8, 0, 0),
}
_xml = plistlib.dumps(_payload, fmt=plistlib.FMT_XML)
assert isinstance(_xml, bytes), "dumps(fmt=FMT_XML) returns bytes"
_ledger.append(1)
assert len(_xml) > 0, "XML output is non-empty"
_ledger.append(1)
# Parsed-back must equal the input (excluding datetime — checked separately
# in section 7, since equality across the round-trip is the strongest
# precondition for the "loads" leg).
_round_xml = plistlib.loads(_xml, fmt=plistlib.FMT_XML)
assert isinstance(_round_xml, dict), "loads returns a dict for top-level <dict>"
_ledger.append(1)
assert _round_xml["name"] == "mamba", "XML round-trip preserves str"
_ledger.append(1)
assert _round_xml["answer"] == 42, "XML round-trip preserves int"
_ledger.append(1)
assert _round_xml["active"] == True, "XML round-trip preserves bool"
_ledger.append(1)
assert _round_xml["tags"] == ["alpha", "beta", "gamma"], "XML round-trip preserves list"
_ledger.append(1)
assert _round_xml["nested"]["depth"] == 2, "XML round-trip preserves nested dict scalar"
_ledger.append(1)
assert _round_xml["nested"]["items"] == [1, 2, 3], "XML round-trip preserves nested list"
_ledger.append(1)
assert _round_xml["blob"] == b"\x00\x01\x02hello", "XML round-trip preserves bytes (Data)"
_ledger.append(1)

# 4. BINARY round-trip on the same payload.
_bin = plistlib.dumps(_payload, fmt=plistlib.FMT_BINARY)
assert isinstance(_bin, bytes), "dumps(fmt=FMT_BINARY) returns bytes"
_ledger.append(1)
assert len(_bin) > 0, "BINARY output is non-empty"
_ledger.append(1)
_round_bin = plistlib.loads(_bin, fmt=plistlib.FMT_BINARY)
assert isinstance(_round_bin, dict), "BINARY loads returns a dict"
_ledger.append(1)
assert _round_bin["name"] == "mamba", "BINARY round-trip preserves str"
_ledger.append(1)
assert _round_bin["answer"] == 42, "BINARY round-trip preserves int"
_ledger.append(1)
assert _round_bin["active"] == True, "BINARY round-trip preserves bool"
_ledger.append(1)
assert _round_bin["tags"] == ["alpha", "beta", "gamma"], "BINARY round-trip preserves list"
_ledger.append(1)
assert _round_bin["nested"]["depth"] == 2, "BINARY round-trip preserves nested dict scalar"
_ledger.append(1)
assert _round_bin["blob"] == b"\x00\x01\x02hello", "BINARY round-trip preserves bytes (Data)"
_ledger.append(1)

# 5. XML output structure.
assert _xml.startswith(b"<?xml"), "XML output starts with <?xml prologue"
_ledger.append(1)
assert b"<plist" in _xml, "XML output carries <plist root element"
_ledger.append(1)
assert b"<dict>" in _xml, "XML output carries <dict> child for top-level dict"
_ledger.append(1)

# 6. BINARY output magic header.
assert _bin.startswith(b"bplist00"), "BINARY output starts with 'bplist00' magic"
_ledger.append(1)

# 7. Datetime preserved across both round-trips.
assert _round_xml["created"] == datetime.datetime(2026, 5, 21, 8, 0, 0), (
    "XML round-trip preserves datetime"
)
_ledger.append(1)
assert _round_bin["created"] == datetime.datetime(2026, 5, 21, 8, 0, 0), (
    "BINARY round-trip preserves datetime"
)
_ledger.append(1)

# Loads-with-default-fmt — plistlib auto-detects the format on load.
_auto_xml = plistlib.loads(_xml)
assert _auto_xml["name"] == "mamba", "loads auto-detects FMT_XML"
_ledger.append(1)
_auto_bin = plistlib.loads(_bin)
assert _auto_bin["name"] == "mamba", "loads auto-detects FMT_BINARY"
_ledger.append(1)

# Emit the proof-of-execution marker. Per `ASSERTION_PASS_MARKERS` in
# `cpython_lib_test_runner.rs`, presence of this token escalates the
# outcome from ImportPass to AssertionPass.
print(f"MAMBA_ASSERTION_PASS: plistlib {len(_ledger)} asserts")
