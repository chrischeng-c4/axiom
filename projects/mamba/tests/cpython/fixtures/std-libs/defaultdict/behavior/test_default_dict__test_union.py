# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "defaultdict"
# dimension = "behavior"
# case = "test_default_dict__test_union"
# subject = "cpython.test_defaultdict.TestDefaultDict.test_union"
# kind = "semantic"
# xfail = "auto-ported CPython test; mamba promotion pending"
# mem_carveout = ""
# source = "Lib/test/test_defaultdict.py"
# status = "filled"
# ///
# mamba-xfail: auto-ported CPython test; mamba promotion pending
# Auto-ported from CPython 3.12 test_defaultdict.py::TestDefaultDict::test_union
"""Auto-ported test: TestDefaultDict::test_union (CPython 3.12 oracle)."""


from collections import defaultdict


def assert_raises(exc_type, fn, *args):
    try:
        fn(*args)
    except exc_type:
        return
    raise AssertionError(f"expected {exc_type.__name__}")


i = defaultdict(int, {1: 1, 2: 2})
s = defaultdict(str, {0: "zero", 1: "one"})

i_s = i | s
assert i_s.default_factory is int
assert dict(i_s) == {1: "one", 2: 2, 0: "zero"}
assert list(i_s) == [1, 2, 0]

s_i = s | i
assert s_i.default_factory is str
assert dict(s_i) == {0: "zero", 1: 1, 2: 2}
assert list(s_i) == [0, 1, 2]

i_ds = i | dict(s)
assert i_ds.default_factory is int
assert dict(i_ds) == {1: "one", 2: 2, 0: "zero"}
assert list(i_ds) == [1, 2, 0]

ds_i = dict(s) | i
assert ds_i.default_factory is int
assert dict(ds_i) == {0: "zero", 1: 1, 2: 2}
assert list(ds_i) == [0, 1, 2]

assert_raises(TypeError, lambda: i | list(s.items()))
assert_raises(TypeError, lambda: list(s.items()) | i)

i |= list(s.items())
assert i.default_factory is int
assert dict(i) == {1: "one", 2: 2, 0: "zero"}
assert list(i) == [1, 2, 0]

assert_raises(TypeError, lambda: i.__ior__(None))

print("TestDefaultDict::test_union: ok")
