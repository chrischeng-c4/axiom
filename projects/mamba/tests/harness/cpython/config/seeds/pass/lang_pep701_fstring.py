# lang_pep701_fstring.py - axis-1 PEP 701 f-string improvements seed (#3342).
#
# Surface (from #3342):
#   1. Multi-line f-string with embedded expression
#   2. Nested quotes: f"{d[\"k\"]}" using mixed quote styles
#   3. Format spec: f"{x:.2f}", f"{x:>10}"
#   4. f"{x!r}", f"{x!s}", f"{x!a}" conversions
#   5. f-string inside f-string
#
# Contract with cpython_lib_test_runner (#2691): each top-level `assert`
# executes; AssertionError -> non-zero exit -> `Fail`. Emitting
# `MAMBA_ASSERTION_PASS: lang_pep701_fstring N asserts` flips to AssertionPass.

_ledger: list[int] = []

# 1. Multi-line f-string with embedded expression.
x = 3.14159
multi = f"""line1
line2 {x}"""
assert "line1" in multi, "multi-line f-string preserves first line"
_ledger.append(1)

assert "3.14159" in multi, "multi-line f-string interpolates expression on second line"
_ledger.append(1)

# 2. Nested quotes via swapped quote style: f'...{d["k"]}...'.
d = {"k": "v", "other": "w"}
nested = f'{d["k"]}'
assert nested == "v", "nested-quote f-string indexes dict with double-quoted key"
_ledger.append(1)

nested2 = f'<{d["k"]}|{d["other"]}>'
assert nested2 == "<v|w>", "nested-quote f-string handles two indexed lookups"
_ledger.append(1)

# 3. Format spec: precision and alignment.
assert f"{x:.2f}" == "3.14", "format spec :.2f truncates float"
_ledger.append(1)

s = "hello"
assert f"{s:>10}" == "     hello", "format spec :>10 right-aligns within width 10"
_ledger.append(1)

assert f"{s:<10}" == "hello     ", "format spec :<10 left-aligns within width 10"
_ledger.append(1)

# 4. Conversions: !r, !s, !a.
assert f"{s!r}" == "'hello'", "!r conversion uses repr()"
_ledger.append(1)

assert f"{s!s}" == "hello", "!s conversion uses str()"
_ledger.append(1)

assert f"{s!a}" == "'hello'", "!a conversion uses ascii() (ascii-safe repr)"
_ledger.append(1)

# 5. f-string inside f-string (PEP 701 nesting; same quote style allowed).
inner = f"{f'{x:.1f}'}"
assert inner == "3.1", "f-string nested inside another f-string formats precision"
_ledger.append(1)

depth2 = f"<{f'[{f"{x:.0f}"}]'}>"
assert depth2 == "<[3]>", "PEP 701 allows reusing same quote style across nested f-strings"
_ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_pep701_fstring {sum(_ledger)} asserts")
