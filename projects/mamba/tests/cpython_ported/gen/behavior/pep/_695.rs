use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/pep/695/explicit_base_keeps_generic_appended.py`.
#[test]
fn test_gen_behavior_pep_695_explicit_base_keeps_generic_appended() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "695"
# dimension = "behavior"
# case = "explicit_base_keeps_generic_appended"
# subject = "typing.Generic"
# kind = "semantic"
# xfail = "Child.__bases__ returns None on mamba so the (Base, Generic) ordering can't be checked (probed 2026-05-29)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.Generic: an explicit base is preserved with Generic appended after it; keyword args and **dict expansion flow to __init_subclass__ (Child[T](Base, a=1, b=2, **extra))"""
from typing import Generic


# An explicit base is preserved, with Generic appended after it.
class Base:
    def __init_subclass__(cls, **kwargs):
        cls.kwargs = kwargs


# Keyword args and **dict expansion flow to __init_subclass__ as usual.
extra = {"c": 3}
class Child[T](Base, a=1, b=2, **extra):
    pass


assert Child.__bases__ == (Base, Generic)
assert Child.kwargs == {"a": 1, "b": 2, "c": 3}

print("explicit_base_keeps_generic_appended OK")
"###);
    assert_output(&out, r###"explicit_base_keeps_generic_appended OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/695/generic_class_stores_values.py`.
#[test]
fn test_gen_behavior_pep_695_generic_class_stores_values() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "695"
# dimension = "behavior"
# case = "generic_class_stores_values"
# subject = "typing.Generic"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.Generic: a generic class Box[T] instantiates and stores any value regardless of the declared T: Box(42).value==42 and Box('hi').value=='hi'"""


# A generic class instantiates and stores values regardless of the declared T.
class Box[T]:
    def __init__(self, value: T) -> None:
        self.value = value


assert Box(42).value == 42
assert Box("hi").value == "hi"

print("generic_class_stores_values OK")
"###);
    assert_output(&out, r###"generic_class_stores_values OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/695/generic_function_polymorphic.py`.
#[test]
fn test_gen_behavior_pep_695_generic_function_polymorphic() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "695"
# dimension = "behavior"
# case = "generic_function_polymorphic"
# subject = "typing.TypeVar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.TypeVar: a generic function def first[T](xs)->T is fully polymorphic at runtime: first([1,2,3]) is 1 and first(['a','b']) is 'a' (the type param is erased)"""


# A generic function is fully polymorphic at runtime; the type param is erased.
def first[T](xs: list[T]) -> T:
    return xs[0]


assert first([1, 2, 3]) == 1
assert first(["a", "b"]) == "a"

print("generic_function_polymorphic OK")
"###);
    assert_output(&out, r###"generic_function_polymorphic OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/695/generic_method_zero_arg_super.py`.
#[test]
fn test_gen_behavior_pep_695_generic_method_zero_arg_super() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "695"
# dimension = "behavior"
# case = "generic_method_zero_arg_super"
# subject = "typing.Generic"
# kind = "semantic"
# xfail = "zero-arg super() inside a def greet[T] generic method diverges on mamba (PEP 695 method type-param path; probed 2026-05-29)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.Generic: a generic method def greet[T](self, tag) can still use zero-arg super() to reach its base: Sub().greet(1) == 'parent-sub'"""


# A generic method can still use zero-arg super() to reach its base.
class Parent:
    def greet(self):
        return "parent"


class Sub(Parent):
    def greet[T](self, tag: int) -> str:
        return super().greet() + "-sub"


assert Sub().greet(1) == "parent-sub"

print("generic_method_zero_arg_super OK")
"###);
    assert_output(&out, r###"generic_method_zero_arg_super OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/695/star_unpacked_base_lists.py`.
#[test]
fn test_gen_behavior_pep_695_star_unpacked_base_lists() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "695"
# dimension = "behavior"
# case = "star_unpacked_base_lists"
# subject = "typing.Generic"
# kind = "semantic"
# xfail = "Empty/Starred.__bases__ return None on mamba (probed 2026-05-29)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.Generic: star-unpacked base lists work for generic classes: class Empty[T](*()) yields just (Generic,) and class Starred[T](*[Base]) yields (Base, Generic)"""
from typing import Generic


class Base:
    pass


# Star-unpacked base lists work too: an empty one yields just Generic.
class Empty[T](*()):
    pass


assert Empty.__bases__ == (Generic,)

bases = [Base]
class Starred[T](*bases):
    pass


assert Starred.__bases__ == (Base, Generic)

print("star_unpacked_base_lists OK")
"###);
    assert_output(&out, r###"star_unpacked_base_lists OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/695/type_params_empty_on_plain.py`.
#[test]
fn test_gen_behavior_pep_695_type_params_empty_on_plain() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "695"
# dimension = "behavior"
# case = "type_params_empty_on_plain"
# subject = "typing.TypeVar"
# kind = "semantic"
# xfail = "plain.__type_params__ returns None on mamba, not () (probed 2026-05-29)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.TypeVar: non-generic callables, classes, and builtin types carry an empty-but-present __type_params__ == () (plain fn, plain class, type, object)"""


# Non-generic callables and classes have an empty (but present) __type_params__.
def plain():
    pass


class Plain:
    pass


assert plain.__type_params__ == ()
assert Plain.__type_params__ == ()
# Even builtin types carry the attribute.
assert type.__type_params__ == ()
assert object.__type_params__ == ()

print("type_params_empty_on_plain OK")
"###);
    assert_output(&out, r###"type_params_empty_on_plain OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/695/type_params_writable.py`.
#[test]
fn test_gen_behavior_pep_695_type_params_writable() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "695"
# dimension = "behavior"
# case = "type_params_writable"
# subject = "typing.TypeVar"
# kind = "semantic"
# xfail = "cls/fn.__type_params__ returns None on mamba and the attribute is not writable (probed 2026-05-29)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.TypeVar: __type_params__ is writable on both a generic function and a generic class: assigning () is observed back"""


# __type_params__ is a writable attribute on both functions and classes.
def gen_fn[A]():
    pass


class GenCls[A]:
    pass


gen_fn.__type_params__ = ()
GenCls.__type_params__ = ()
assert gen_fn.__type_params__ == ()
assert GenCls.__type_params__ == ()

print("type_params_writable OK")
"###);
    assert_output(&out, r###"type_params_writable OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/695/typeparam_default_args.py`.
#[test]
fn test_gen_behavior_pep_695_typeparam_default_args() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "695"
# dimension = "behavior"
# case = "typeparam_default_args"
# subject = "typing.TypeVar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.TypeVar: default arguments annotated with a type param work normally: def defaulted[T](a=..., *, b=...) yields ('a','b'), (1,'b'), ('a',2)"""


# Default arguments annotated with a type param work normally.
def defaulted[T](a: T = "a", *, b: T = "b"):
    return (a, b)


assert defaulted() == ("a", "b")
assert defaulted(1) == (1, "b")
assert defaulted(b=2) == ("a", 2)

print("typeparam_default_args OK")
"###);
    assert_output(&out, r###"typeparam_default_args OK
"###);
}
