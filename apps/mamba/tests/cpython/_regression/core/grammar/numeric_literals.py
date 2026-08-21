# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""core/grammar: numeric literal token forms (CPython 3.12 oracle).

Distilled from CPython TokenTests integer/float/underscore literal tests:
the runtime VALUE each literal form evaluates to, plus the syntax-error
forms that the tokenizer rejects.
"""

# Integer bases all evaluate to the same value.
assert 0xFF == 255
assert 0o777 == 511
assert 0b101010 == 42
assert 0xFFFF_FFFF == 4294967295
print("bases: ok")

# Underscore digit grouping is cosmetic; value matches the stripped form.
assert 1_000_000 == 1000000
assert 0b1001_0100 == 0b10010100
assert 1_00_00.5 == 10000.5
assert 1e1_0 == 1e10
assert .1_4 == 0.14
assert 1_00_00j == 10000j
print("underscores: ok")

# Leading zeros are fine for floats/complex; only bare decimal int 0NNN is bad.
assert 0777. == 777.0
assert 0e0 == 0
assert 00j == 0j
assert 09.5 == 9.5
print("leading_zeros: ok")

# Float exponents and complex.
assert 2e3 == 2000.0
assert 1.5e-2 == 0.015
assert (1 + 2j).imag == 2.0
assert (1 + 2j).real == 1.0
print("float_complex: ok")

# Tokenizer rejects malformed literals as SyntaxError (not ValueError).
_bad = ["077787", "0xj", "0e", "0b42", "0o123456789", "2e", "1e-", "0777"]
for src in _bad:
    try:
        eval(src)
        raise AssertionError("expected SyntaxError for %r" % src)
    except SyntaxError:
        pass
print("syntax_errors: ok")

# Underscores must sit between digits; trailing/double underscores are invalid.
for src in ["0_", "1_.4", "1__0", "0x_", "1e_1"]:
    try:
        eval(src)
        raise AssertionError("expected SyntaxError for %r" % src)
    except SyntaxError:
        pass
print("underscore_errors: ok")

# Ellipsis literal is the singleton.
e = ...
assert e is Ellipsis
print("numeric_literals OK")
