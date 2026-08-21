# Operational AssertionPass seed for SILENT divergences in `string`
# (Template / Formatter class identity + substitute, the documented
# `printable` superset constant), `textwrap` (fill / wrap actually
# wrapping at the requested width, shorten producing the documented
# "[...]" sentinel, TextWrapper class identity), `io` (StringIO /
# BytesIO class identity + write/seek/read round-trip + DEFAULT_BUFFER
# _SIZE + SEEK_* integer constants), and `argparse` (ArgumentParser
# class identity + parse_args, Namespace, the documented SUPPRESS /
# OPTIONAL / REMAINDER / ZERO_OR_MORE / ONE_OR_MORE sentinel strings).
#
# The matching subset (every per-character-class string constant,
# capwords, dedent, indent on non-empty lines) is covered by
# `test_string_textwrap_constants_dedent_ops`; this fixture pins the
# CPython-only contracts that mamba currently elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • string.printable — documented superset of digits + ascii_letters
#     + punctuation + whitespace (str of length 100)
#     (mamba: hasattr returns False, value None);
#   • string.Template.__name__ == "Template" — class identity
#     (mamba: returns None);
#   • string.Template("$x").substitute(x="hi") == "hi"
#     (mamba: returns a plain dict, AttributeError on .substitute);
#   • string.Formatter.__name__ == "Formatter" — class identity
#     (mamba: returns None);
#   • textwrap.fill("hello world", width=8) == "hello\\nworld"
#     (mamba: returns the unwrapped input "hello world");
#   • textwrap.wrap("hello world", width=8) == ["hello", "world"]
#     (mamba: returns the unsplit list ["hello world"]);
#   • textwrap.shorten("hello world", width=10) == "[...]"
#     (mamba: returns "hell [...]" — width handled but placeholder
#     placement diverges);
#   • textwrap.TextWrapper.__name__ == "TextWrapper" — class identity
#     (mamba: hasattr returns False, value None);
#   • io.StringIO.__name__ == "StringIO" — class identity
#     (mamba: returns None);
#   • io.BytesIO.__name__ == "BytesIO" — class identity
#     (mamba: returns None);
#   • io.StringIO write/seek/read round-trip
#     (mamba: write returns 0, read fails);
#   • io.BytesIO write/seek/read round-trip
#     (mamba: write returns 0, read fails);
#   • io.DEFAULT_BUFFER_SIZE == 8192 — documented buffer size sentinel
#     (mamba: returns None);
#   • io.SEEK_SET == 0 / SEEK_CUR == 1 / SEEK_END == 2 — documented
#     seek-mode integer constants (mamba: all return None);
#   • argparse.ArgumentParser.__name__ == "ArgumentParser"
#     (mamba: returns None);
#   • ArgumentParser().add_argument + parse_args round-trip
#     (mamba: AttributeError on .add_argument);
#   • argparse.SUPPRESS == "==SUPPRESS==" (mamba: returns None);
#   • argparse.OPTIONAL == "?" (mamba: returns None);
#   • argparse.REMAINDER == "..." (mamba: returns None);
#   • argparse.ZERO_OR_MORE == "*" (mamba: returns None);
#   • argparse.ONE_OR_MORE == "+" (mamba: returns None).
import string as _string_mod
import textwrap as _textwrap_mod
import io as _io_mod
import argparse as _argparse_mod
from typing import Any

# Module bindings retyped as `Any` to bypass Pyright stub-driven
# narrowing — every spec contract below probes documented public
# class objects, instance methods, or module-level sentinels that
# mamba's bundled type stubs do not surface.
string: Any = _string_mod
textwrap: Any = _textwrap_mod
io: Any = _io_mod
argparse: Any = _argparse_mod

_ledger: list[int] = []

# 1) string.printable — documented superset of length 100
assert isinstance(string.printable, str); _ledger.append(1)
assert len(string.printable) == 100; _ledger.append(1)
assert "0" in string.printable; _ledger.append(1)
assert "a" in string.printable; _ledger.append(1)

# 2) string.Template — class identity + substitute
assert string.Template.__name__ == "Template"; _ledger.append(1)
_t: Any = string.Template("$x")
assert _t.substitute(x="hi") == "hi"; _ledger.append(1)
_t2: Any = string.Template("$greet, $who!")
assert _t2.substitute(greet="hello", who="world") == "hello, world!"; _ledger.append(1)

# 3) string.Formatter — class identity
assert string.Formatter.__name__ == "Formatter"; _ledger.append(1)

# 4) textwrap.fill / wrap — actually wrap at the requested width
assert textwrap.fill("hello world", width=8) == "hello\nworld"; _ledger.append(1)
assert textwrap.wrap("hello world", width=8) == ["hello", "world"]; _ledger.append(1)

# 5) textwrap.shorten — documented placeholder layout
assert textwrap.shorten("hello world", width=10) == "[...]"; _ledger.append(1)

# 6) textwrap.TextWrapper — class identity
assert textwrap.TextWrapper.__name__ == "TextWrapper"; _ledger.append(1)

# 7) io.StringIO — class identity + write/seek/read round-trip
assert io.StringIO.__name__ == "StringIO"; _ledger.append(1)
_s: Any = io.StringIO()
_n: int = _s.write("hello")
assert _n == 5; _ledger.append(1)
_s.seek(0)
assert _s.read() == "hello"; _ledger.append(1)

# 8) io.BytesIO — class identity + write/seek/read round-trip
assert io.BytesIO.__name__ == "BytesIO"; _ledger.append(1)
_b: Any = io.BytesIO()
_nb: int = _b.write(b"hello-bytes")
assert _nb == 11; _ledger.append(1)
_b.seek(0)
assert _b.read() == b"hello-bytes"; _ledger.append(1)

# 9) io.DEFAULT_BUFFER_SIZE / SEEK_* integer sentinels
assert io.DEFAULT_BUFFER_SIZE == 8192; _ledger.append(1)
assert io.SEEK_SET == 0; _ledger.append(1)
assert io.SEEK_CUR == 1; _ledger.append(1)
assert io.SEEK_END == 2; _ledger.append(1)

# 10) argparse.ArgumentParser — class identity + parse round-trip
assert argparse.ArgumentParser.__name__ == "ArgumentParser"; _ledger.append(1)
_p: Any = argparse.ArgumentParser()
_p.add_argument("--name")
_ns: Any = _p.parse_args(["--name", "alice"])
assert _ns.name == "alice"; _ledger.append(1)

# 11) argparse sentinel strings
assert argparse.SUPPRESS == "==SUPPRESS=="; _ledger.append(1)
assert argparse.OPTIONAL == "?"; _ledger.append(1)
assert argparse.REMAINDER == "..."; _ledger.append(1)
assert argparse.ZERO_OR_MORE == "*"; _ledger.append(1)
assert argparse.ONE_OR_MORE == "+"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_string_template_textwrap_wrap_io_argparse_silent {sum(_ledger)} asserts")
