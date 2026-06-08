# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_fstring_padding_conversion"
# subject = "cpython321.lang_fstring_padding_conversion"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/lang_fstring_padding_conversion.py"
# status = "filled"
# ///
"""cpython321.lang_fstring_padding_conversion: execute CPython 3.12 seed lang_fstring_padding_conversion"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for f-string padding and conversion
# specifiers. Surface: width specs `{x:5}` right-align integers and
# left-align strings to the given column width; `{x:05}` zero-pads on
# the left; explicit alignment flags `>`, `<`, `^` force right, left,
# and centre with space fill; the sign flag `+` shows `+` for positive
# and `-` for negative integers; integer base conversions `b`/`o`/`x`/
# `X` produce binary/octal/lowercase-hex/uppercase-hex with the `#`
# alternate form adding the `0b`/`0x` prefix; float specs `.2f`/`.4f`
# round to the requested precision and `10.2f` combines width with
# precision; the exponent spec `e` formats in scientific notation;
# arbitrary fill characters before the alignment flag (`*>5`, `0>5`,
# `->5`) replace the default padding; string operands accept the same
# width/align spec; conversion flags `!r` and `!s` apply repr() and
# str() before formatting; nested expressions inside `{}` evaluate
# arbitrary Python including method calls, subscripts, conditionals,
# arithmetic, and PEP 448 list/tuple displays.
_ledger: list[int] = []
x = 42
# Basic substitution and arithmetic in braces
assert f"{x}" == "42"; _ledger.append(1)
assert f"{x + 1}" == "43"; _ledger.append(1)
# Width + alignment
assert f"{x:5}" == "   42"; _ledger.append(1)
assert f"{x:05}" == "00042"; _ledger.append(1)
assert f"{x:>5}" == "   42"; _ledger.append(1)
assert f"{x:<5}" == "42   "; _ledger.append(1)
assert f"{x:^5}" == " 42  "; _ledger.append(1)
# Sign flag
assert f"{x:+}" == "+42"; _ledger.append(1)
assert f"{-x:+}" == "-42"; _ledger.append(1)
# Integer base conversions
assert f"{x:b}" == "101010"; _ledger.append(1)
assert f"{x:o}" == "52"; _ledger.append(1)
assert f"{x:x}" == "2a"; _ledger.append(1)
assert f"{x:X}" == "2A"; _ledger.append(1)
# Alt-form prefix
assert f"{x:#b}" == "0b101010"; _ledger.append(1)
assert f"{x:#x}" == "0x2a"; _ledger.append(1)
# Float specs
f = 3.14159
assert f"{f:.2f}" == "3.14"; _ledger.append(1)
assert f"{f:.4f}" == "3.1416"; _ledger.append(1)
assert f"{f:10.2f}" == "      3.14"; _ledger.append(1)
assert f"{f:>10.2f}" == "      3.14"; _ledger.append(1)
assert f"{f:e}" == "3.141590e+00"; _ledger.append(1)
# Custom fill characters
assert f"{x:*>5}" == "***42"; _ledger.append(1)
assert f"{x:0>5}" == "00042"; _ledger.append(1)
assert f"{x:->5}" == "---42"; _ledger.append(1)
# String operand width/align
s = "hi"
assert f"{s:10}" == "hi        "; _ledger.append(1)
assert f"{s:>10}" == "        hi"; _ledger.append(1)
assert f"{s:^10}" == "    hi    "; _ledger.append(1)
assert f"{s:*^10}" == "****hi****"; _ledger.append(1)
# Adjacent literals
name = "Alice"
assert f"Hello, {name}!" == "Hello, Alice!"; _ledger.append(1)
assert f"{name}{name}" == "AliceAlice"; _ledger.append(1)
assert f"a{x}b{x}c" == "a42b42c"; _ledger.append(1)
# Nested expressions
assert f"{1 + 2 * 3}" == "7"; _ledger.append(1)
assert f"{[1, 2, 3]}" == "[1, 2, 3]"; _ledger.append(1)
assert f"{(1, 2)}" == "(1, 2)"; _ledger.append(1)
# Method call inside expression
assert f"{s.upper()}" == "HI"; _ledger.append(1)
# Subscript inside expression
L = [10, 20, 30]
assert f"{L[1]}" == "20"; _ledger.append(1)
# Conditional inside expression
n = 5
assert f"{'big' if n > 3 else 'small'}" == "big"; _ledger.append(1)
# Conversion flags !r, !s
assert f"{s!r}" == "'hi'"; _ledger.append(1)
assert f"{s!s}" == "hi"; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: lang_fstring_padding_conversion {sum(_ledger)} asserts")
