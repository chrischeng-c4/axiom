use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/pep/484/assert_type_returns_value_identity.py`.
#[test]
fn test_gen_behavior_pep_484_assert_type_returns_value_identity() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "484"
# dimension = "behavior"
# case = "assert_type_returns_value_identity"
# subject = "typing.assert_type"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.assert_type: assert_type returns the value object itself (identity), never a copy: assert_type(arg,object) is arg, assert_type(arg,str|float) is arg, assert_type(arg,None) is arg"""
from typing import assert_type

# assert_type returns the value object itself (identity), never a copy.
arg = object()
assert assert_type(arg, object) is arg
assert assert_type(arg, str | float) is arg
assert assert_type(arg, None) is arg

print("assert_type_returns_value_identity OK")
"###);
    assert_output(&out, r###"assert_type_returns_value_identity OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/484/bare_special_form_repr_stable.py`.
#[test]
fn test_gen_behavior_pep_484_bare_special_form_repr_stable() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "484"
# dimension = "behavior"
# case = "bare_special_form_repr_stable"
# subject = "typing.Any"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.Any: the bare special forms have a stable, module-qualified repr: repr(typing.Any)=='typing.Any', repr(typing.NoReturn)=='typing.NoReturn', repr(typing.Never)=='typing.Never'"""
import typing

# repr of the bare special forms is stable and module-qualified.
assert repr(typing.Any) == "typing.Any"
assert repr(typing.NoReturn) == "typing.NoReturn"
assert repr(typing.Never) == "typing.Never"

print("bare_special_form_repr_stable OK")
"###);
    assert_output(&out, r###"bare_special_form_repr_stable OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/484/cast_is_runtime_noop.py`.
#[test]
fn test_gen_behavior_pep_484_cast_is_runtime_noop() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "484"
# dimension = "behavior"
# case = "cast_is_runtime_noop"
# subject = "typing.cast"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.cast: cast is a runtime no-op that returns its argument unchanged with no coercion for any type form: cast(int,42)==42, cast(float,42) stays an int, cast(Any,'x')=='x', cast(Union[str,float],42)==42, cast(None,42)==42"""
from typing import Any, AnyStr, Union, cast

# cast is a runtime no-op: it returns its argument unchanged for any form.
assert cast(int, 42) == 42
assert cast(float, 42) == 42
assert type(cast(float, 42)) is int  # no coercion happens
assert cast(Any, "x") == "x"
assert cast(list, 42) == 42
assert cast(Union[str, float], 42) == 42
assert cast(AnyStr, 42) == 42
assert cast(None, 42) == 42

print("cast_is_runtime_noop OK")
"###);
    assert_output(&out, r###"cast_is_runtime_noop OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/484/generic_subclass_holds_value.py`.
#[test]
fn test_gen_behavior_pep_484_generic_subclass_holds_value() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "484"
# dimension = "behavior"
# case = "generic_subclass_holds_value"
# subject = "typing.Generic"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.Generic: a Generic[T] subclass is an ordinary class at runtime: Container(typing.Generic[T]) stores and returns its value, Container(42).value==42"""
import typing

T = typing.TypeVar("T")


class Container(typing.Generic[T]):
    def __init__(self, value: T) -> None:
        self.value = value


# A Generic subclass is an ordinary class at runtime.
assert Container(42).value == 42

print("generic_subclass_holds_value OK")
"###);
    assert_output(&out, r###"generic_subclass_holds_value OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/484/get_args_yields_param_tuple.py`.
#[test]
fn test_gen_behavior_pep_484_get_args_yields_param_tuple() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "484"
# dimension = "behavior"
# case = "get_args_yields_param_tuple"
# subject = "typing.get_args"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.get_args: get_args yields the parameter tuple in declaration order: (int,) for List[int], (int,str) for Union[int,str], (1,2,3) for Literal[1,2,3], (int,Ellipsis) for Tuple[int,...], () for a bare int, and ([int,str],bool) for Callable[[int,str],bool]"""
from typing import Callable, List, Literal, Tuple, Union, get_args

# get_args yields the parameter tuple in declaration order.
assert get_args(List[int]) == (int,)
assert get_args(Union[int, str]) == (int, str)
assert get_args(Literal[1, 2, 3]) == (1, 2, 3)
assert get_args(Tuple[int, ...]) == (int, Ellipsis)
assert get_args(int) == ()
assert get_args(Callable[[int, str], bool]) == ([int, str], bool)

print("get_args_yields_param_tuple OK")
"###);
    assert_output(&out, r###"get_args_yields_param_tuple OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/484/literal_value_and_type_identity.py`.
#[test]
fn test_gen_behavior_pep_484_literal_value_and_type_identity() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "484"
# dimension = "behavior"
# case = "literal_value_and_type_identity"
# subject = "typing.Literal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.Literal: Literal compares by value-and-type, dedups, ignores order, flattens nesting, and keeps bool/int distinct: Literal[1,2]==Literal[2,1], Literal[1,2,3]==Literal[1,2,3,3], Literal[True]!=Literal[1], Literal[0]!=Literal[False], and Literal[Literal[1,2],3].__args__==(1,2,3)"""
from typing import Literal

# Literal compares by value-and-type, dedups, and ignores order.
assert Literal[1] == Literal[1]
assert Literal[1, 2] == Literal[2, 1]
assert Literal[1, 2, 3] == Literal[1, 2, 3, 3]
assert Literal[1] != Literal[2]
# bool and int literals are kept distinct by type.
assert Literal[True] != Literal[1]
assert Literal[0] != Literal[False]
# Nested Literals flatten.
flat = Literal[Literal[1, 2], 3]
assert flat == Literal[1, 2, 3]
assert flat.__args__ == (1, 2, 3)

print("literal_value_and_type_identity OK")
"###);
    assert_output(&out, r###"literal_value_and_type_identity OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/484/namedtuple_class_form.py`.
#[test]
fn test_gen_behavior_pep_484_namedtuple_class_form() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "484"
# dimension = "behavior"
# case = "namedtuple_class_form"
# subject = "typing.NamedTuple"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.NamedTuple: a class-form NamedTuple is a real tuple subclass with named fields, defaults, and helpers: Point(NamedTuple) with x:int, y:int=0 gives Point(1)==(1,0), isinstance tuple, _fields==('x','y'), _field_defaults=={'y':0}, __annotations__=={'x':int,'y':int}, _replace(y=5)==(1,5), _asdict()=={'x':1,'y':0}"""
from typing import NamedTuple


# Class-form NamedTuple: a real tuple subclass with named fields and defaults.
class Point(NamedTuple):
    x: int
    y: int = 0


p = Point(1)
assert isinstance(p, tuple)
assert (p.x, p.y) == (1, 0)
assert p == (1, 0)
assert Point._fields == ("x", "y")
assert Point._field_defaults == {"y": 0}
assert Point.__annotations__ == {"x": int, "y": int}
assert p._replace(y=5) == (1, 5)
assert p._asdict() == {"x": 1, "y": 0}

print("namedtuple_class_form OK")
"###);
    assert_output(&out, r###"namedtuple_class_form OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/484/namedtuple_functional_form.py`.
#[test]
fn test_gen_behavior_pep_484_namedtuple_functional_form() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "484"
# dimension = "behavior"
# case = "namedtuple_functional_form"
# subject = "typing.NamedTuple"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.NamedTuple: the functional NamedTuple('Employee',[('name',str),('age',int)]) syntax builds a tuple subclass with named access and metadata: Employee('Nick',25).name=='Nick', __name__=='Employee', _fields==('name','age'); mixing the list form with keywords raises TypeError"""
from typing import NamedTuple

# Functional NamedTuple via the (name, [(field, type), ...]) syntax.
Employee = NamedTuple("Employee", [("name", str), ("age", int)])
e = Employee("Nick", 25)
assert isinstance(e, tuple)
assert e.name == "Nick"
assert Employee.__name__ == "Employee"
assert Employee._fields == ("name", "age")

# Mixing the list form with keywords is rejected.
try:
    NamedTuple("Bad", [("x", int)], y=str)
    raise AssertionError("expected TypeError")
except TypeError:
    pass

print("namedtuple_functional_form OK")
"###);
    assert_output(&out, r###"namedtuple_functional_form OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/484/newtype_identity_with_metadata.py`.
#[test]
fn test_gen_behavior_pep_484_newtype_identity_with_metadata() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "484"
# dimension = "behavior"
# case = "newtype_identity_with_metadata"
# subject = "typing.NewType"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.NewType: NewType produces a callable identity function with introspectable metadata: UserId=NewType('UserId',int) gives UserId(5)==5, UserId.__name__=='UserId', UserId.__supertype__ is int"""
from typing import NewType

# NewType produces a callable identity function with introspectable metadata.
UserId = NewType("UserId", int)
assert UserId(5) == 5
assert UserId.__name__ == "UserId"
assert UserId.__supertype__ is int

print("newtype_identity_with_metadata OK")
"###);
    assert_output(&out, r###"newtype_identity_with_metadata OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/484/optional_is_union_with_none.py`.
#[test]
fn test_gen_behavior_pep_484_optional_is_union_with_none() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "484"
# dimension = "behavior"
# case = "optional_is_union_with_none"
# subject = "typing.Optional"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.Optional: Optional[X] is exactly Union[X,None]: typing.Optional[int]==Union[int,None] and get_args(Optional[int])==(int,type(None))"""
import typing
from typing import Union, get_args

# Optional[X] is exactly Union[X, None].
assert typing.Optional[int] == Union[int, None]
assert get_args(typing.Optional[int]) == (int, type(None))

print("optional_is_union_with_none OK")
"###);
    assert_output(&out, r###"optional_is_union_with_none OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/484/overload_stubs_and_get_overloads.py`.
#[test]
fn test_gen_behavior_pep_484_overload_stubs_and_get_overloads() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "484"
# dimension = "behavior"
# case = "overload_stubs_and_get_overloads"
# subject = "typing.overload"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.overload: a function with only @overload stubs raises NotImplementedError when called; once a concrete impl is defined the function works and get_overloads returns its two stubs"""
from typing import get_overloads, overload


# A function with only @overload stubs raises NotImplementedError when called.
@overload
def f(x: int) -> int: ...
@overload
def f(x: str) -> str: ...


try:
    f(1)
    raise AssertionError("expected NotImplementedError")
except NotImplementedError:
    pass


# Once a concrete implementation is defined, the function is usable and its
# overload stubs are retrievable via get_overloads (keyed by the impl).
@overload
def g(x: int) -> int: ...
@overload
def g(x: str) -> str: ...
def g(x):
    return x * 2


assert g(3) == 6
assert g("a") == "aa"
assert len(get_overloads(g)) == 2

print("overload_stubs_and_get_overloads OK")
"###);
    assert_output(&out, r###"overload_stubs_and_get_overloads OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/484/override_sets_runtime_flag.py`.
#[test]
fn test_gen_behavior_pep_484_override_sets_runtime_flag() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "484"
# dimension = "behavior"
# case = "override_sets_runtime_flag"
# subject = "typing.override"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.override: @override is a pass-through that sets the __override__ flag at runtime: a Child.run decorated with @override returns 2, Child.run.__override__ is True, and the undecorated Base.run has no __override__ attribute"""
from typing import override


# @override marks a method with the __override__ flag at runtime (a no-op pass-through).
class Base:
    def run(self) -> int:
        return 1


class Child(Base):
    @override
    def run(self) -> int:
        return 2


assert Child().run() == 2
assert Child.run.__override__ is True
assert not hasattr(Base.run, "__override__")

print("override_sets_runtime_flag OK")
"###);
    assert_output(&out, r###"override_sets_runtime_flag OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/484/protocol_concrete_subclass_inherits.py`.
#[test]
fn test_gen_behavior_pep_484_protocol_concrete_subclass_inherits() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "484"
# dimension = "behavior"
# case = "protocol_concrete_subclass_inherits"
# subject = "typing.Protocol"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.Protocol: a concrete class may subclass a Protocol to inherit its interface and is then an ordinary class: Square(Drawable) implements draw() and Square().draw()=='square'"""
from typing import Protocol


class Drawable(Protocol):
    def draw(self) -> None: ...


# A concrete class may subclass a Protocol to inherit its interface.
class Square(Drawable):
    def draw(self):
        return "square"


assert Square().draw() == "square"

print("protocol_concrete_subclass_inherits OK")
"###);
    assert_output(&out, r###"protocol_concrete_subclass_inherits OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/484/union_dedups_and_flattens.py`.
#[test]
fn test_gen_behavior_pep_484_union_dedups_and_flattens() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "484"
# dimension = "behavior"
# case = "union_dedups_and_flattens"
# subject = "typing.Union"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.Union: Union deduplicates members, ignores order, collapses a single member, and flattens nested unions: Union[int,int] is int, Union[int,str]==Union[str,int], Union[int,str,int]==Union[int,str], Union[Union[int,str],float]==Union[int,str,float], and Union[int,float]!=Union"""
from typing import Optional, Union

# Union deduplicates members and ignores order.
assert Union[int, int] is int
assert Union[int, str] == Union[str, int]
assert Union[int, str, int] == Union[int, str]
# Nested unions flatten into a single union.
assert Union[Union[int, str], float] == Union[int, str, float]
# Optional[X] is Union[X, None]; the bare special form is not a union value.
assert Optional[int] == Union[int, None]
assert Union[int, float] != Union

print("union_dedups_and_flattens OK")
"###);
    assert_output(&out, r###"union_dedups_and_flattens OK
"###);
}
