# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_mimetypes_advanced_ops"
# subject = "cpython321.test_mimetypes_advanced_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_mimetypes_advanced_ops.py"
# status = "filled"
# ///
"""cpython321.test_mimetypes_advanced_ops: execute CPython 3.12 seed test_mimetypes_advanced_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for mimetypes advanced surface.
# Surface: `mimetypes.guess_type(path)` returns a `(type, encoding)`
# 2-tuple where `type` is the MIME type string inferred from the
# extension or `None` if the extension is unknown. The common
# extensions resolve to canonical types: `.txt` → text/plain, `.html`
# → text/html, `.json` → application/json, `.png` → image/png, `.jpg`
# → image/jpeg, `.css` → text/css, `.js` → application/javascript or
# text/javascript, `.pdf` → application/pdf, `.xml` →
# application/xml or text/xml, `.zip` → application/zip. The
# inverse `guess_extension(mimetype)` returns the canonical extension
# string for a MIME type, e.g. `.txt` for text/plain. `add_type(type,
# ext)` registers a new mapping that subsequent `guess_type` calls
# observe. `mimetypes.init()` initialises the internal tables and
# sets `mimetypes.inited` to True. The module exposes `types_map` (a
# dict keyed by extension → MIME) and `encodings_map` (a dict of
# encoding suffixes like .gz / .bz2 → encoding name).
import mimetypes
_ledger: list[int] = []

# Common extensions resolve to canonical types
assert mimetypes.guess_type("file.txt")[0] == "text/plain"; _ledger.append(1)
assert mimetypes.guess_type("file.txt")[1] is None; _ledger.append(1)
assert mimetypes.guess_type("file.html")[0] == "text/html"; _ledger.append(1)
assert mimetypes.guess_type("file.json")[0] == "application/json"; _ledger.append(1)
assert mimetypes.guess_type("image.png")[0] == "image/png"; _ledger.append(1)
assert mimetypes.guess_type("image.jpg")[0] in ("image/jpeg", "image/jpg"); _ledger.append(1)
assert mimetypes.guess_type("file.css")[0] == "text/css"; _ledger.append(1)
assert mimetypes.guess_type("file.js")[0] in ("application/javascript", "text/javascript"); _ledger.append(1)
assert mimetypes.guess_type("file.pdf")[0] == "application/pdf"; _ledger.append(1)
assert mimetypes.guess_type("file.xml")[0] in ("application/xml", "text/xml"); _ledger.append(1)
assert mimetypes.guess_type("file.zip")[0] == "application/zip"; _ledger.append(1)

# Unknown extension yields (None, None)
assert mimetypes.guess_type("noext")[0] is None; _ledger.append(1)

# Inverse: guess_extension returns the canonical extension string
assert mimetypes.guess_extension("text/plain") == ".txt"; _ledger.append(1)
assert mimetypes.guess_extension("text/html") in (".htm", ".html"); _ledger.append(1)

# add_type registers a fresh mapping observable by guess_type
mimetypes.add_type("application/x-test", ".tst")
assert mimetypes.guess_type("foo.tst")[0] == "application/x-test"; _ledger.append(1)

# init() initialises tables and sets `inited` to a bool
mimetypes.init()
assert isinstance(mimetypes.inited, bool); _ledger.append(1)

# types_map exposes the extension → MIME registry
assert isinstance(mimetypes.types_map, dict); _ledger.append(1)
assert ".txt" in mimetypes.types_map; _ledger.append(1)
assert mimetypes.types_map[".txt"] == "text/plain"; _ledger.append(1)
assert ".html" in mimetypes.types_map; _ledger.append(1)

# encodings_map exposes the .gz/.bz2/... → encoding registry
assert isinstance(mimetypes.encodings_map, dict); _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_mimetypes_advanced_ops {sum(_ledger)} asserts")
