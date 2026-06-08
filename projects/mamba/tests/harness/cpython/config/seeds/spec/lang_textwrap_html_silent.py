# Operational AssertionPass seed for SILENT divergences across the
# text-formatting / HTML-escape pair pinned by atomic 162:
# `textwrap` (the documented `wrap` width-enforcement +
# `fill` newline-joining + `shorten` placeholder-elision +
# `indent` trailing-newline-preservation contracts) and `html`
# (the documented `escape(quote=False)` single-quote-preservation
# contract + `unescape` numeric-entity expansion contract).
#
# The matching subset (shlex.split / quote / join chain on string
# literals, struct.pack / unpack / calcsize on big-endian ints
# and floats, calendar.isleap / month_name / day_name /
# monthrange tuple / weekday / module hasattr surface, secrets
# module hasattr surface + token_bytes / token_hex / token_urlsafe
# / choice / randbelow / compare_digest output type+length
# contracts, queue.Queue single-thread put / get_nowait / qsize /
# empty chain + LifoQueue / PriorityQueue / Empty hasattr
# surface) is covered by
# `test_shlex_struct_calendar_secrets_queue_value_ops`; this
# fixture pins the CPython-only contracts that mamba currently
# elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • textwrap.wrap("a b c d e f g h i j", width=5) returns a
#     multi-element list of width-bounded chunks (mamba: returns
#     `["a b c d e f g h i j"]` — single-element list, width
#     bound silently ignored);
#   • textwrap.fill("a b c d e f g h i j", width=5) returns a
#     newline-joined string of width-bounded chunks (mamba:
#     returns the input unchanged with no newlines);
#   • textwrap.shorten("Hello world this is a test", width=15)
#     returns a width-bounded string ending with `[...]`
#     placeholder (mamba: returns the input unchanged);
#   • textwrap.indent("hi\nbye\n", "> ") preserves the trailing
#     newline (mamba: returns `"> hi\n> bye"` — trailing
#     newline silently dropped);
#   • html.escape("<a href='x'>&hi</a>", quote=False) preserves
#     single-quotes — `quote=False` MUST suppress quote escaping
#     (mamba: still escapes single-quotes as `&#x27;`, parameter
#     ignored);
#   • html.unescape("&#65;&#66;&#67;") == "ABC" — numeric
#     entity expansion contract (mamba: returns the raw
#     `"&#65;&#66;&#67;"` literal, numeric entities not
#     expanded).
import textwrap as _textwrap_mod
import html as _html_mod
from typing import Any

# Module bindings retyped as `Any` to bypass Pyright stub-driven
# narrowing — every spec contract below probes documented public
# module-level helpers that mamba's bundled type stubs do not
# surface accurately.
textwrap: Any = _textwrap_mod
html: Any = _html_mod


_ledger: list[int] = []

# 1) textwrap.wrap — width-enforced chunking
assert textwrap.wrap("a b c d e f g h i j", width=5) == ["a b c", "d e f", "g h i", "j"]; _ledger.append(1)

# 2) textwrap.fill — newline-joined chunking
assert textwrap.fill("a b c d e f g h i j", width=5) == "a b c\nd e f\ng h i\nj"; _ledger.append(1)

# 3) textwrap.shorten — placeholder-elision contract
assert textwrap.shorten("Hello world this is a test", width=15) == "Hello [...]"; _ledger.append(1)

# 4) textwrap.indent — trailing-newline preservation
assert textwrap.indent("hi\nbye\n", "> ") == "> hi\n> bye\n"; _ledger.append(1)

# 5) html.escape — quote=False single-quote preservation
assert html.escape("<a href='x'>&hi</a>", quote=False) == "&lt;a href='x'&gt;&amp;hi&lt;/a&gt;"; _ledger.append(1)

# 6) html.unescape — numeric entity expansion
assert html.unescape("&#65;&#66;&#67;") == "ABC"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_textwrap_html_silent {sum(_ledger)} asserts")
