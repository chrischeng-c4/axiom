# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""core/comprehension_scope: closure capture and class-cell interaction."""


# Closures capture the loop variable by REFERENCE, not by value. Every
# lambda created in the comprehension sees the final value of `i`.
late = [lambda: i for i in range(5)]
assert [fn() for fn in late] == [4, 4, 4, 4, 4]
print("[class-scope] late-binding:", [fn() for fn in late])


# Same late-binding rule holds for a generator expression of closures.
gen_funcs = list(lambda: j for j in range(3))
assert [fn() for fn in gen_funcs] == [2, 2, 2]
print("[class-scope] genexpr late-binding:", [fn() for fn in gen_funcs])


# A default argument is evaluated eagerly, so it captures each value early.
early = [lambda v=i: v for i in range(5)]
assert [fn() for fn in early] == [0, 1, 2, 3, 4]
print("[class-scope] default-arg early-binding:", [fn() for fn in early])


# A comprehension inside a class body coexists with the implicit __class__
# cell used by zero-argument super(). The comprehension's lambdas still
# follow the closure late-binding rule, and __class__ resolves correctly.
class C:
    def method(self):
        super()  # forces creation of the __class__ closure cell
        return __class__

    items = [lambda: i for i in range(5)]
    y = [fn() for fn in items]


assert C.y == [4, 4, 4, 4, 4]
assert C().method() is C
print("[class-scope] class-cell y:", C.y)
print("[class-scope] __class__ resolves:", C().method() is C)
