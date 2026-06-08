# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "functools"
# dimension = "behavior"
# case = "singledispatch_dispatches_on_first_arg_type"
# subject = "functools.singledispatch"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_functools.py"
# status = "filled"
# ///
"""functools.singledispatch: singledispatch picks the implementation by the first argument's type via explicit register, decorator register, .dispatch, MRO, ABC, and annotation/union registration"""
import collections.abc
import functools
import typing


# Explicit register(type, impl): unregistered types fall back to base.
@functools.singledispatch
def describe(obj):
    return "base"


def _describe_int(i):
    return "integer"


describe.register(int, _describe_int)
assert describe("str") == "base", "str -> base"
assert describe(1) == "integer", "int -> integer"
assert describe([1, 2]) == "base", "list -> base"


# Decorator form @g.register(type) plus .dispatch lookups.
@functools.singledispatch
def kind(obj):
    return "default"


@kind.register(int)
def _kind_int(i):
    return f"int {i}"


assert kind("") == "default", "empty str default"
assert kind(12) == "int 12", "int dispatched"
assert kind.dispatch(int) is _kind_int, "dispatch(int)"
assert kind.dispatch(object) is kind.dispatch(str), "unregistered -> base impl"


# MRO resolution: D(C, B) with A and B registered prefers B over A.
@functools.singledispatch
def label(obj):
    return "base"


class A:
    pass


class C(A):
    pass


class B(A):
    pass


class D(C, B):
    pass


label.register(A, lambda o: "A")
label.register(B, lambda o: "B")
assert label(A()) == "A", "A -> A"
assert label(B()) == "B", "B -> B"
assert label(C()) == "A", "C inherits A"
assert label(D()) == "B", "D(C,B) prefers B"


# ABC registration: concrete types match the most specific abstract base.
@functools.singledispatch
def abc_kind(obj):
    return "base"


abc_kind.register(collections.abc.Sequence, lambda o: "sequence")
abc_kind.register(collections.abc.MutableSequence, lambda o: "mutableseq")
assert abc_kind((1, 2)) == "sequence", "tuple -> sequence"
assert abc_kind([1, 2]) == "mutableseq", "list -> mutableseq"


# Annotation-based register: the type comes from the parameter annotation.
@functools.singledispatch
def via_ann(arg):
    return "base"


@via_ann.register
def _(arg: collections.abc.Mapping):
    return "mapping"


assert via_ann(None) == "base", "None -> base"
assert via_ann({"a": 1}) == "mapping", "dict -> mapping"


# Union annotations (typing.Union and X | Y) both register.
@functools.singledispatch
def uni(arg):
    return "default"


@uni.register
def _(arg: typing.Union[str, bytes]):
    return "union"


@uni.register
def _(arg: int | float):
    return "uniontype"


assert uni([]) == "default", "list default"
assert uni("") == "union", "str union"
assert uni(b"") == "union", "bytes union"
assert uni(1) == "uniontype", "int uniontype"
assert uni(1.0) == "uniontype", "float uniontype"

print("singledispatch_dispatches_on_first_arg_type OK")
