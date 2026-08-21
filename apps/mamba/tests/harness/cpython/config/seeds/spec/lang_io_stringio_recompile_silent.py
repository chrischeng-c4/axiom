# Operational AssertionPass seed for SILENT divergences across the
# in-memory IO / compiled-regex pair pinned by atomic 173: `io`
# (the documented `StringIO(...)` / `BytesIO(...)` instance
# constructor + the documented `.write` / `.getvalue` round-
# trip + the documented `.read` / `.seek` random-access surface)
# and `re` (the documented `re.compile(pattern).search(text)`
# compiled-Pattern instance method surface).
#
# The matching subset (re module-level helper layer (match /
# search / sub / subn / findall / split / escape), Match
# object instance method layer when produced by the module-
# level helpers (group / groups / start / end), integer flag-
# value layer (IGNORECASE / MULTILINE / DOTALL), full re
# module hasattr surface) is covered by
# `test_re_module_full_value_ops`; this fixture pins the
# CPython-only contracts that mamba currently elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • io.StringIO().write("hello"); .getvalue() == "hello" —
#     documented in-memory write-then-read-back contract
#     (mamba: returns "" — io.StringIO() returns a `dict` and
#     the documented round-trip is broken even though
#     hasattr(...) reports write / getvalue as True);
#   • io.StringIO("hello world").read() == "hello world" —
#     documented constructor + read contract (mamba: returns
#     "");
#   • io.StringIO("hello").seek(0) — documented Random-access
#     method (mamba: AttributeError 'dict' object has no
#     attribute 'seek');
#   • io.BytesIO().write(b"abc"); .getvalue() == b"abc" —
#     documented in-memory write-then-read-back contract for
#     bytes (mamba: returns b"");
#   • io.BytesIO(b"hello world").read() == b"hello world" —
#     documented constructor + read contract (mamba:
#     AttributeError 'dict' object has no attribute 'read');
#   • re.compile(r"hello", re.IGNORECASE).search("HELLO
#     world").group(0) == "HELLO" — documented compiled-
#     Pattern instance method (mamba: .search returns None on
#     a pattern that should match — compiled-Pattern
#     instance surface broken).
import io as _io_mod
import re as _re_mod
from typing import Any

# Module bindings retyped as `Any` to bypass Pyright stub-driven
# narrowing — every spec contract below probes documented public
# class identifiers / instance methods / compiled-Pattern
# methods that mamba's bundled type stubs do not surface
# accurately.
io: Any = _io_mod
re: Any = _re_mod


_ledger: list[int] = []

# 1) io.StringIO — write + getvalue round-trip
_sb = io.StringIO()
_sb.write("hello")
_sb.write(" world")
assert _sb.getvalue() == "hello world"; _ledger.append(1)

# 2) io.StringIO — ctor + read
_sb2 = io.StringIO("hello world")
assert _sb2.read() == "hello world"; _ledger.append(1)

# 3) io.StringIO — seek
_sb3 = io.StringIO("hello world")
_sb3.seek(0)
assert _sb3.read() == "hello world"; _ledger.append(1)

# 4) io.BytesIO — write + getvalue round-trip
_bb = io.BytesIO()
_bb.write(b"abc")
_bb.write(b"def")
assert _bb.getvalue() == b"abcdef"; _ledger.append(1)

# 5) io.BytesIO — ctor + read
_bb2 = io.BytesIO(b"hello world")
assert _bb2.read() == b"hello world"; _ledger.append(1)

# 6) re.compile + .search — compiled-Pattern instance method
_pat = re.compile(r"hello", re.IGNORECASE)
_m = _pat.search("HELLO world")
assert _m.group(0) == "HELLO"; _ledger.append(1)

# 7) re.compile + .findall — compiled-Pattern instance method
_pat2 = re.compile(r"\d+")
assert _pat2.findall("a 1 b 22 c 333") == ["1", "22", "333"]; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_io_stringio_recompile_silent {sum(_ledger)} asserts")
