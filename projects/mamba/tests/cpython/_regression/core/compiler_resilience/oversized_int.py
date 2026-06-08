# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""core/compiler_resilience: oversized integer literal > 2^63 (CPython 3.12 oracle)."""
# mamba-xfail: compiler has no parser bounds — hostile source crashes/diverges (WI #3929)
#
# The literal is built at RUNTIME ("9"*100) and fed to compile()/eval(). CPython
# has arbitrary-precision ints, so a 100-digit literal is a normal value. mamba's
# parser caps integer literals at 64-bit and rejects this as "unexpected token"
# (project_mamba_int_literal_64bit_cap), so the fixture is xfail for mamba.

src = "9" * 100
print("digit_count:", len(src))

# CPython 3.12: a 100-digit decimal literal compiles to a bignum constant.
# (Note: int()-from-STRING has a 4300-digit conversion cap since 3.11, but the
# COMPILER's handling of an integer LITERAL is not subject to that limit.)
code = compile(src, "<bignum>", "eval")
value = eval(code)
assert isinstance(value, int)
assert value == int(src)            # exact arbitrary-precision value
assert value == 10 ** 100 - 1       # 100 nines == 10^100 - 1
assert value.bit_length() > 63      # well beyond a 64-bit int
print("bit_length:", value.bit_length())
print("exceeds_2_63:", value > 2 ** 63)

# Arithmetic on the bignum stays exact (no overflow / truncation).
assert value + 1 == 10 ** 100
assert (value * 2) // 2 == value
print("bignum_arith_exact: True")

# Document mamba's divergence explicitly.
mamba_note = (
    "mamba parser caps int literals at 64-bit and rejects '9'*100 as "
    "'unexpected token' (project_mamba_int_literal_64bit_cap)"
)
print("mamba_note:", mamba_note)

print("oversized_int: CPython bignum literal OK, no crash — OK")
