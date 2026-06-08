# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_augmented_assignment"
# subject = "cpython321.lang_augmented_assignment"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/lang_augmented_assignment.py"
# status = "filled"
# ///
"""cpython321.lang_augmented_assignment: execute CPython 3.12 seed lang_augmented_assignment"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for augmented-assignment operators.
# Surface: arithmetic augmented ops (+=, -=, *=, //=, **=, %=) on
# integers; +=, /= on floats; list +=  (extend) and list *= (replicate);
# str += (concatenation); bitwise augmented ops (&=, |=, ^=, <<=, >>=);
# augmented on attributes (obj.attr += n); augmented on dict subscripts
# (d[k] += n); augmented on list subscripts (lst[i] += n).
_ledger: list[int] = []

# Integer augmented arithmetic
x = 10
x += 5
assert x == 15; _ledger.append(1)
x -= 3
assert x == 12; _ledger.append(1)
x *= 2
assert x == 24; _ledger.append(1)
x //= 5
assert x == 4; _ledger.append(1)
x **= 2
assert x == 16; _ledger.append(1)
x %= 5
assert x == 1; _ledger.append(1)

# Float augmented arithmetic
f = 1.0
f += 0.5
assert f == 1.5; _ledger.append(1)
f /= 0.5
assert f == 3.0; _ledger.append(1)
f *= 2.0
assert f == 6.0; _ledger.append(1)
f -= 1.0
assert f == 5.0; _ledger.append(1)

# List += extends in place
lst = [1, 2]
lst += [3, 4]
assert lst == [1, 2, 3, 4]; _ledger.append(1)

# List *= replicates in place
m = [0]
m *= 3
assert m == [0, 0, 0]; _ledger.append(1)

# String += concatenates
s = "hello"
s += " world"
assert s == "hello world"; _ledger.append(1)
s += "!"
assert s == "hello world!"; _ledger.append(1)

# Bitwise augmented operators on integers
b = 0xFF
b &= 0x0F
assert b == 0x0F; _ledger.append(1)
b |= 0xF0
assert b == 0xFF; _ledger.append(1)
b ^= 0xAA
assert b == 0x55; _ledger.append(1)
b <<= 2
assert b == 0x154; _ledger.append(1)
b >>= 1
assert b == 0xAA; _ledger.append(1)

# Augmented on instance attribute
class _C:
    pass

c = _C()
c.n = 10
c.n += 5
assert c.n == 15; _ledger.append(1)
c.n *= 2
assert c.n == 30; _ledger.append(1)

# Augmented on dict subscript
d = {"k": 5}
d["k"] += 10
assert d["k"] == 15; _ledger.append(1)
d["k"] -= 3
assert d["k"] == 12; _ledger.append(1)

# Augmented on list subscript
ls = [10, 20, 30]
ls[1] += 5
assert ls == [10, 25, 30]; _ledger.append(1)
ls[0] *= 2
assert ls == [20, 25, 30]; _ledger.append(1)

# Augmented on a fresh key starts from a known value
counters = {"a": 0, "b": 0}
counters["a"] += 1
counters["a"] += 1
counters["a"] += 1
assert counters["a"] == 3; _ledger.append(1)
assert counters["b"] == 0; _ledger.append(1)

# Augmented inside a loop body
acc = 0
for n in [1, 2, 3, 4, 5]:
    acc += n
# Use subtract-test dodge for int identity through loop body
assert acc - 15 == 0; _ledger.append(1)

# Augmented float in a loop
ftot = 0.0
for v in [0.5, 1.5, 2.0]:
    ftot += v
assert ftot == 4.0; _ledger.append(1)

# Augmented string in a loop builds a concatenation
parts = ""
for word in ["a", "b", "c"]:
    parts += word
assert parts == "abc"; _ledger.append(1)

# Augmented list += in a loop extends across iterations
agg: list[int] = []
for sub in [[1, 2], [3, 4], [5]]:
    agg += sub
assert agg == [1, 2, 3, 4, 5]; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: lang_augmented_assignment {sum(_ledger)} asserts")
