# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "turtle"
# dimension = "behavior"
# case = "test_vec2d__test_constructor"
# subject = "cpython.test_turtle.TestVec2D.test_constructor"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_turtle.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_turtle.py::TestVec2D::test_constructor
"""Auto-ported test: TestVec2D::test_constructor (CPython 3.12 oracle)."""


try:
    import turtle
except ImportError:
    print("TestVec2D::test_constructor: skipped, turtle unavailable")
    raise SystemExit(0)


Vec2D = turtle.Vec2D
vec = Vec2D(0.5, 2)
assert vec[0] == 0.5, vec
assert vec[1] == 2, vec
assert isinstance(vec, Vec2D)

for args in ((), (0,), ((0, 1),), (vec,), (0, 1, 2)):
    try:
        Vec2D(*args)
    except TypeError:
        pass
    else:
        raise AssertionError(f"Vec2D{args!r} did not raise TypeError")

print("TestVec2D::test_constructor: ok")
