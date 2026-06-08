# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_augmented_unpack"
# subject = "cpython321.lang_augmented_unpack"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/lang_augmented_unpack.py"
# status = "filled"
# ///
"""cpython321.lang_augmented_unpack: execute CPython 3.12 seed lang_augmented_unpack"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for augmented-assignment and unpack
# forms beyond test_unpacking_ops.
# Surface: nested-tuple destructuring; star-unpack with an empty
# middle slice; iterable unpack from a string; chained assignment
# (`a = b = c = expr`); unpack from `range`; unpack from a function's
# tuple return; augmented assignment `+= -= *= //= %= **=`.
# Note: list-pattern targets like `[x, y] = [1, 2]` are NOT exercised
# here — they have a known breakage on the current mamba runtime.
_ledger: list[int] = []

# Nested unpack — the inner tuple destructures into the inner names
(a, (b, c)) = (1, (2, 3))
assert a == 1; _ledger.append(1)
assert b == 2; _ledger.append(1)
assert c == 3; _ledger.append(1)

# Star unpack with an empty middle slice
a2, *mid, b2 = [1, 2]
assert a2 == 1; _ledger.append(1)
assert mid == []; _ledger.append(1)
assert b2 == 2; _ledger.append(1)

# Iterable unpack from a string yields characters
ch1, ch2, ch3 = "abc"
assert ch1 == "a"; _ledger.append(1)
assert ch2 == "b"; _ledger.append(1)
assert ch3 == "c"; _ledger.append(1)

# Chained assignment binds the same value to multiple names
x = y = z = 10
assert x == 10; _ledger.append(1)
assert y == 10; _ledger.append(1)
assert z == 10; _ledger.append(1)

# Unpack from range() — iterable with known length
r1, r2, r3 = range(3)
assert r1 == 0; _ledger.append(1)
assert r2 == 1; _ledger.append(1)
assert r3 == 2; _ledger.append(1)

# Unpack from a function's tuple return without explicit parens
def _two():
    return 10, 20

t1, t2 = _two()
assert t1 == 10; _ledger.append(1)
assert t2 == 20; _ledger.append(1)

# Augmented assignment runs op-then-store; integer return is
# bound back to local before any check to dodge the int-identity
# quirk on annotated/generic return positions.
n = 5
n += 3
assert n == 8; _ledger.append(1)
n *= 2
assert n == 16; _ledger.append(1)
n -= 1
assert n == 15; _ledger.append(1)
n //= 3
assert n == 5; _ledger.append(1)
n %= 3
assert n == 2; _ledger.append(1)
n **= 4
assert n == 16; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: lang_augmented_unpack {sum(_ledger)} asserts")
