# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_string_prefixes"
# subject = "cpython321.lang_string_prefixes"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/lang_string_prefixes.py"
# status = "filled"
# ///
"""cpython321.lang_string_prefixes: execute CPython 3.12 seed lang_string_prefixes"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for string-literal prefix forms.
# Surface: raw strings r"\n"/r"\t"/r"\\" are length-2/length-2/length-2
# (escape disabled, backslash literal), distinct from plain "\n" which
# is length-1; triple-quoted forms """abc"""/"""xyz""" parse identically
# to single-quoted, and multi-line triple strings carry the embedded
# newline; triple-raw r"""\n""" stays raw; bytes literals b"abc"
# compare equal to bytes("abc", "utf-8"), have integer-element subscript
# (ASCII codepoint), support + concat and * repeat and slice; bytes↔
# str round-trip via .encode/.decode("utf-8"); f-string with format
# specs (`:5` width, `:05` zero-pad, `:>10` / `:<10` align) interpolates
# correctly; PEP 3101 parse-time adjacent-literal concat ("abc" "def")
# fuses at parse time. Companion to lang_fstring_format_spec /
# lang_string_methods / lang_string_slicing.
_ledger: list[int] = []

# Raw string — backslash is literal
assert r"\n" == "\\n"; _ledger.append(1)
assert r"\t" == "\\t"; _ledger.append(1)
assert r"hello\nworld" == "hello\\nworld"; _ledger.append(1)
assert len(r"\n") == 2; _ledger.append(1)
assert len("\n") == 1; _ledger.append(1)
assert r"\\" == "\\\\"; _ledger.append(1)
assert r"abc" == "abc"; _ledger.append(1)

# Triple-quoted parses as plain
assert """abc""" == "abc"; _ledger.append(1)
assert """xyz""" == "xyz"; _ledger.append(1)
assert """line1
line2""" == "line1\nline2"; _ledger.append(1)

# Triple-raw stays raw
assert r"""\n""" == "\\n"; _ledger.append(1)

# Bytes literal — equality, length, byte-as-int subscript
assert b"abc" == b"abc"; _ledger.append(1)
assert b"abc" == bytes("abc", "utf-8"); _ledger.append(1)
assert len(b"abc") == 3; _ledger.append(1)
assert b"" == b""; _ledger.append(1)
assert b"\x00" == b"\x00"; _ledger.append(1)
assert b"hello" == b"hello"; _ledger.append(1)
assert b"abc"[0] == 97; _ledger.append(1)
assert b"abc"[1] == 98; _ledger.append(1)
assert b"abc"[2] == 99; _ledger.append(1)

# Bytes ↔ str round-trip via utf-8
assert b"abc".decode("utf-8") == "abc"; _ledger.append(1)
assert "abc".encode("utf-8") == b"abc"; _ledger.append(1)
assert "héllo".encode("utf-8") != b"hllo"; _ledger.append(1)

# f-string with format specs
x = 42
assert f"{x}" == "42"; _ledger.append(1)
assert f"{x + 1}" == "43"; _ledger.append(1)
assert f"{x:5}" == "   42"; _ledger.append(1)
assert f"{x:05}" == "00042"; _ledger.append(1)
assert f"{x:>10}" == "        42"; _ledger.append(1)
assert f"{x:<10}" == "42        "; _ledger.append(1)

# Adjacent-literal parse-time concat
assert "abc" "def" == "abcdef"; _ledger.append(1)
assert "x" "y" "z" == "xyz"; _ledger.append(1)

# Bytes + concat / * repeat / slice
assert b"abc" + b"def" == b"abcdef"; _ledger.append(1)
assert b"abc" * 2 == b"abcabc"; _ledger.append(1)
assert b"abcdef"[1:4] == b"bcd"; _ledger.append(1)
assert b"abcdef"[:3] == b"abc"; _ledger.append(1)
assert b"abcdef"[3:] == b"def"; _ledger.append(1)

# Empty / repeat edges
assert "" == ""; _ledger.append(1)
assert len("") == 0; _ledger.append(1)
assert "" + "abc" == "abc"; _ledger.append(1)
assert "a" * 5 == "aaaaa"; _ledger.append(1)
assert "x" * 0 == ""; _ledger.append(1)
assert "ab" * 3 == "ababab"; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_string_prefixes {sum(_ledger)} asserts")
