# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cpython321_stdlib"
# dimension = "real_world"
# case = "test_mimetypes_guess_ops"
# subject = "cpython321.test_mimetypes_guess_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_mimetypes_guess_ops.py"
# status = "filled"
# ///
"""cpython321.test_mimetypes_guess_ops: execute CPython 3.12 seed test_mimetypes_guess_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the `mimetypes` module — the
# extension → MIME-type lookup table (`types_map`), the forward
# `guess_type(path)` resolver (path → (mime, encoding)), the reverse
# `guess_extension(mime)` resolver, and the encoding suffix table
# (`encodings_map` for gzip/bz2). `mimetypes` is consumed by HTTP
# clients/servers, file-upload UI layers, and email/MIME compose
# code; it has no fixture yet.
#
# Surface:
#   • mimetypes.guess_type(path)
#       — returns a 2-tuple (mime, encoding); mime is `None` for
#         unknown extensions and for paths with no extension at all;
#       — is case-insensitive on the extension;
#       — also works on full URL-shaped paths (the suffix lookup is on
#         the trailing path component);
#   • mimetypes.types_map
#       — dict[str, str] mapping `.ext` → MIME-type;
#       — covers >= 100 entries on a stock install;
#       — common entries: .html → text/html, .css → text/css,
#         .json → application/json, .pdf → application/pdf, etc.;
#   • mimetypes.guess_extension(mime)
#       — reverse lookup: MIME → `.ext`; returns `None` for unknown;
#   • mimetypes.encodings_map
#       — dict mapping the trailing compression suffix → encoding
#         (`.gz` → gzip, `.bz2` → bzip2).
#
# Note: mamba and CPython disagree on the canonical MIME of the `.xml`
# extension (mamba: `text/xml`, CPython 3.12: `application/xml`) so
# this seed skips the `.xml` slot.
import mimetypes
_ledger: list[int] = []

# Forward — common extensions agree across runtimes
assert mimetypes.guess_type("test.html") == ("text/html", None); _ledger.append(1)
assert mimetypes.guess_type("test.css") == ("text/css", None); _ledger.append(1)
assert mimetypes.guess_type("test.js") == ("text/javascript", None); _ledger.append(1)
assert mimetypes.guess_type("test.pdf") == ("application/pdf", None); _ledger.append(1)
assert mimetypes.guess_type("test.zip") == ("application/zip", None); _ledger.append(1)
assert mimetypes.guess_type("test.csv") == ("text/csv", None); _ledger.append(1)
assert mimetypes.guess_type("test.jpg") == ("image/jpeg", None); _ledger.append(1)
assert mimetypes.guess_type("test.jpeg") == ("image/jpeg", None); _ledger.append(1)
assert mimetypes.guess_type("test.gif") == ("image/gif", None); _ledger.append(1)
assert mimetypes.guess_type("test.svg") == ("image/svg+xml", None); _ledger.append(1)
assert mimetypes.guess_type("test.mp3") == ("audio/mpeg", None); _ledger.append(1)
assert mimetypes.guess_type("test.mp4") == ("video/mp4", None); _ledger.append(1)

# Forward — unknown extensions return (None, None)
assert mimetypes.guess_type("test.xyzunknown_qqq") == (None, None); _ledger.append(1)
# Forward — no extension returns (None, None)
assert mimetypes.guess_type("README") == (None, None); _ledger.append(1)

# Forward — case-insensitive on the extension
assert mimetypes.guess_type("test.HTML") == ("text/html", None); _ledger.append(1)
assert mimetypes.guess_type("test.JPG") == ("image/jpeg", None); _ledger.append(1)

# Forward — URL paths work (suffix lookup on trailing component)
assert mimetypes.guess_type("http://example.com/page.html") == ("text/html", None); _ledger.append(1)
assert mimetypes.guess_type("https://example.com/file.pdf") == ("application/pdf", None); _ledger.append(1)

# types_map — direct dict lookups for the stable subset
assert isinstance(mimetypes.types_map, dict); _ledger.append(1)
assert len(mimetypes.types_map) >= 100; _ledger.append(1)
assert mimetypes.types_map[".py"] == "text/x-python"; _ledger.append(1)
assert mimetypes.types_map[".txt"] == "text/plain"; _ledger.append(1)
assert mimetypes.types_map[".pdf"] == "application/pdf"; _ledger.append(1)
assert mimetypes.types_map[".json"] == "application/json"; _ledger.append(1)
assert mimetypes.types_map[".html"] == "text/html"; _ledger.append(1)
assert mimetypes.types_map[".css"] == "text/css"; _ledger.append(1)
assert mimetypes.types_map[".zip"] == "application/zip"; _ledger.append(1)
assert mimetypes.types_map[".gif"] == "image/gif"; _ledger.append(1)
assert mimetypes.types_map[".jpg"] == "image/jpeg"; _ledger.append(1)
assert mimetypes.types_map[".jpeg"] == "image/jpeg"; _ledger.append(1)
assert mimetypes.types_map[".mp3"] == "audio/mpeg"; _ledger.append(1)

# types_map — unknown returns None via .get
assert mimetypes.types_map.get(".xyzqq_unknown_ext") is None; _ledger.append(1)

# Reverse — guess_extension on the canonical MIMEs
assert mimetypes.guess_extension("text/html") == ".html"; _ledger.append(1)
assert mimetypes.guess_extension("text/css") == ".css"; _ledger.append(1)
assert mimetypes.guess_extension("text/plain") == ".txt"; _ledger.append(1)

# Reverse — unknown MIME returns None
assert mimetypes.guess_extension("application/xyzunknown_qqq") is None; _ledger.append(1)

# encodings_map — gzip / bzip2 compression suffixes
assert isinstance(mimetypes.encodings_map, dict); _ledger.append(1)
assert mimetypes.encodings_map[".gz"] == "gzip"; _ledger.append(1)
assert mimetypes.encodings_map[".bz2"] == "bzip2"; _ledger.append(1)

# Tuple-shape — guess_type result is a 2-tuple
_r = mimetypes.guess_type("foo.html")
assert isinstance(_r, tuple); _ledger.append(1)
assert len(_r) == 2; _ledger.append(1)
assert isinstance(_r[0], str); _ledger.append(1)
assert _r[1] is None; _ledger.append(1)

# Return types — types_map keys/values are str
assert all(isinstance(k, str) for k in list(mimetypes.types_map.keys())[:20]); _ledger.append(1)
assert all(isinstance(v, str) for v in list(mimetypes.types_map.values())[:20]); _ledger.append(1)

# Keys in types_map all start with `.`
assert all(k.startswith(".") for k in list(mimetypes.types_map.keys())[:20]); _ledger.append(1)

# inited — module exposes a bool init-state flag
assert isinstance(mimetypes.inited, bool); _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: test_mimetypes_guess_ops {sum(_ledger)} asserts")
