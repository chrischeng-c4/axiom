# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_multiple_inheritance"
# subject = "cpython321.lang_multiple_inheritance"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/lang_multiple_inheritance.py"
# status = "filled"
# ///
"""cpython321.lang_multiple_inheritance: execute CPython 3.12 seed lang_multiple_inheritance"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for multiple inheritance and
# method-resolution order (MRO).
# Surface: linear MRO with C3 linearization, base-order determines
# which method wins on conflict, diamond inheritance through
# super().__init__() cooperative chaining, __mro__ attribute lists
# the resolution order including the implicit object root.
_ledger: list[int] = []

class A:
    def foo(self):
        return "A"

class B:
    def foo(self):
        return "B"

class C(A, B):
    pass

class D(B, A):
    pass

# Base order matters: C(A, B) → A.foo wins
c = C()
assert c.foo() == "A"; _ledger.append(1)
# D(B, A) → B.foo wins
d = D()
assert d.foo() == "B"; _ledger.append(1)

# __mro__ exposes the linearized chain (terminating in object)
c_mro = [cls.__name__ for cls in C.__mro__]
assert c_mro == ["C", "A", "B", "object"]; _ledger.append(1)
d_mro = [cls.__name__ for cls in D.__mro__]
assert d_mro == ["D", "B", "A", "object"]; _ledger.append(1)

# Diamond inheritance — super() chains cooperatively through the MRO
class Top:
    def __init__(self):
        self.history = ["Top"]

class Left(Top):
    def __init__(self):
        super().__init__()
        self.history.append("Left")

class Right(Top):
    def __init__(self):
        super().__init__()
        self.history.append("Right")

class Bottom(Left, Right):
    def __init__(self):
        super().__init__()
        self.history.append("Bottom")

b = Bottom()
# C3 linearization gives Bottom → Left → Right → Top → object
b_mro = [cls.__name__ for cls in Bottom.__mro__]
assert b_mro == ["Bottom", "Left", "Right", "Top", "object"]; _ledger.append(1)

# super().__init__() chain runs through the MRO in order: each
# __init__ executes its super() FIRST, so Top runs first, then Right,
# then Left, then Bottom (postorder traversal)
assert b.history == ["Top", "Right", "Left", "Bottom"]; _ledger.append(1)

# isinstance walks across multiple bases, not just the first
assert isinstance(b, Left); _ledger.append(1)
assert isinstance(b, Right); _ledger.append(1)
assert isinstance(b, Top); _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: lang_multiple_inheritance {sum(_ledger)} asserts")
