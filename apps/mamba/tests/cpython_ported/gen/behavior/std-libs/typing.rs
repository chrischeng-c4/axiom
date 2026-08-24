use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/typing/cast_returns_value_unchanged.py`.
#[test]
fn test_gen_behavior_std_libs_typing_cast_returns_value_unchanged() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "behavior"
# case = "cast_returns_value_unchanged"
# subject = "typing.cast"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_typing.py"
# status = "filled"
# ///
"""typing.cast: cast is a runtime no-op: cast(int, 'not an int') returns the original string object unchanged, doing no conversion or validation"""
import typing

result = typing.cast(int, "not an int")
assert result == "not an int", "cast must return the value unchanged"
assert type(result) is str, "cast does no conversion: the object is still a str"
print("cast_returns_value_unchanged OK")
"###);
    assert_output(&out, r###"cast_returns_value_unchanged OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/typing/generic_subclass_instantiates.py`.
#[test]
fn test_gen_behavior_std_libs_typing_generic_subclass_instantiates() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "behavior"
# case = "generic_subclass_instantiates"
# subject = "typing.Generic"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_typing.py"
# status = "filled"
# ///
"""typing.Generic: a Generic[T] subclass instantiates and stores its value at runtime; the type parameter is erased so behavior is plain Python"""
import typing

T = typing.TypeVar("T")


class Box(typing.Generic[T]):
    def __init__(self, value: T) -> None:
        self.value = value


b = Box(5)
assert b.value == 5, "Generic subclass should store its value"
s = Box("hello")
assert s.value == "hello", "the same Generic subclass works for any erased type"
print("generic_subclass_instantiates OK")
"###);
    assert_output(&out, r###"generic_subclass_instantiates OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/typing/get_args_unpacks_parameters.py`.
#[test]
fn test_gen_behavior_std_libs_typing_get_args_unpacks_parameters() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "behavior"
# case = "get_args_unpacks_parameters"
# subject = "typing.get_args"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_typing.py"
# status = "filled"
# ///
"""typing.get_args: get_args unpacks the type parameters: List[int] -> (int,), Dict[str, int] -> (str, int), Union[int, str] -> (int, str)"""
import typing

assert typing.get_args(typing.List[int]) == (int,), "get_args(List[int])"
assert typing.get_args(typing.Dict[str, int]) == (str, int), "get_args(Dict[str, int])"
assert typing.get_args(typing.Union[int, str]) == (int, str), "get_args(Union[int, str])"
print("get_args_unpacks_parameters OK")
"###);
    assert_output(&out, r###"get_args_unpacks_parameters OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/typing/get_origin_list_is_list.py`.
#[test]
fn test_gen_behavior_std_libs_typing_get_origin_list_is_list() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "behavior"
# case = "get_origin_list_is_list"
# subject = "typing.get_origin"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_typing.py"
# status = "filled"
# ///
"""typing.get_origin: get_origin(List[int]) is the runtime class list and get_origin(Dict[str, int]) is dict"""
import typing

assert typing.get_origin(typing.List[int]) is list, "get_origin(List[int]) should be list"
assert typing.get_origin(typing.Dict[str, int]) is dict, "get_origin(Dict[str, int]) should be dict"
print("get_origin_list_is_list OK")
"###);
    assert_output(&out, r###"get_origin_list_is_list OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/typing/get_type_hints_resolves_function.py`.
#[test]
fn test_gen_behavior_std_libs_typing_get_type_hints_resolves_function() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "behavior"
# case = "get_type_hints_resolves_function"
# subject = "typing.get_type_hints"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_typing.py"
# status = "filled"
# ///
"""typing.get_type_hints: get_type_hints on a function returns a dict mapping each parameter and 'return' to its resolved runtime type (a: int, b: str, return: bool)"""
import typing


def annotated(a: int, b: str) -> bool:
    return bool(a) and bool(b)


hints = typing.get_type_hints(annotated)
assert hints == {"a": int, "b": str, "return": bool}, f"resolved hints = {hints!r}"
print("get_type_hints_resolves_function OK")
"###);
    assert_output(&out, r###"get_type_hints_resolves_function OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/typing/namedtuple_fields_and_tuple_shape.py`.
#[test]
fn test_gen_behavior_std_libs_typing_namedtuple_fields_and_tuple_shape() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "behavior"
# case = "namedtuple_fields_and_tuple_shape"
# subject = "typing.NamedTuple"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_typing.py"
# status = "filled"
# ///
"""typing.NamedTuple: a class-syntax NamedTuple (Point: x, y) builds an instance addressable by field name and by index, equal to the plain tuple of its values"""
import typing


class Point(typing.NamedTuple):
    x: int
    y: int


p = Point(1, 2)
assert p.x == 1 and p.y == 2, "fields addressable by name"
assert p[0] == 1 and p[1] == 2, "fields addressable by index"
assert tuple(p) == (1, 2), "a NamedTuple is the plain tuple of its values"
assert p == (1, 2), "a NamedTuple compares equal to the plain tuple"
print("namedtuple_fields_and_tuple_shape OK")
"###);
    assert_output(&out, r###"namedtuple_fields_and_tuple_shape OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/typing/newtype_is_identity_at_runtime.py`.
#[test]
fn test_gen_behavior_std_libs_typing_newtype_is_identity_at_runtime() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "behavior"
# case = "newtype_is_identity_at_runtime"
# subject = "typing.NewType"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_typing.py"
# status = "filled"
# ///
"""typing.NewType: NewType('UserId', int) is the identity function at runtime: UserId(5) == 5 and is a plain int; the name is exposed via __name__"""
import typing

UserId = typing.NewType("UserId", int)
value = UserId(5)
assert value == 5, "NewType call is the identity function at runtime"
assert type(value) is int, "NewType('UserId', int)(5) is a plain int"
assert UserId.__name__ == "UserId", "NewType exposes its name via __name__"
print("newtype_is_identity_at_runtime OK")
"###);
    assert_output(&out, r###"newtype_is_identity_at_runtime OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/typing/optional_is_union_with_none.py`.
#[test]
fn test_gen_behavior_std_libs_typing_optional_is_union_with_none() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "behavior"
# case = "optional_is_union_with_none"
# subject = "typing.Optional"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_typing.py"
# status = "filled"
# ///
"""typing.Optional: Optional[int] is exactly Union[int, None]; get_origin reports typing.Union and get_args reports (int, NoneType)"""
import typing

assert typing.Optional[int] == typing.Union[int, None], "Optional[int] == Union[int, None]"
assert typing.get_origin(typing.Optional[int]) is typing.Union, "get_origin(Optional[int]) is typing.Union"
assert typing.get_args(typing.Optional[int]) == (int, type(None)), "get_args(Optional[int]) == (int, NoneType)"
print("optional_is_union_with_none OK")
"###);
    assert_output(&out, r###"optional_is_union_with_none OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/typing/parameterized_alias_round_trip.py`.
#[test]
fn test_gen_behavior_std_libs_typing_parameterized_alias_round_trip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "behavior"
# case = "parameterized_alias_round_trip"
# subject = "typing.List"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_typing.py"
# status = "filled"
# ///
"""typing.List: List[int] and Final[int] are subscriptable special forms; List[int] round-trips through get_origin (list) and get_args ((int,))"""
import typing

list_alias = typing.List[int]
assert typing.get_origin(list_alias) is list, "get_origin(List[int]) is list"
assert typing.get_args(list_alias) == (int,), "get_args(List[int]) is (int,)"

# Final[int] is a subscriptable special form too; subscription must not raise.
final_alias = typing.Final[int]
assert final_alias is not None, "Final[int] should be a usable special form"
print("parameterized_alias_round_trip OK")
"###);
    assert_output(&out, r###"parameterized_alias_round_trip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/typing/typevar_bound_recorded.py`.
#[test]
fn test_gen_behavior_std_libs_typing_typevar_bound_recorded() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "behavior"
# case = "typevar_bound_recorded"
# subject = "typing.TypeVar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_typing.py"
# status = "filled"
# ///
"""typing.TypeVar: TypeVar('T', bound=int) records the upper bound on __bound__ and leaves __constraints__ empty"""
import typing

T = typing.TypeVar("T", bound=int)
assert T.__bound__ is int, "TypeVar(bound=int).__bound__ should be int"
assert T.__constraints__ == (), "a bounded TypeVar has no constraints"
print("typevar_bound_recorded OK")
"###);
    assert_output(&out, r###"typevar_bound_recorded OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/typing/typevar_constraints_recorded.py`.
#[test]
fn test_gen_behavior_std_libs_typing_typevar_constraints_recorded() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "behavior"
# case = "typevar_constraints_recorded"
# subject = "typing.TypeVar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_typing.py"
# status = "filled"
# ///
"""typing.TypeVar: TypeVar('T', int, str) records the constraint set (int, str) on __constraints__ and leaves __bound__ unset"""
import typing

T = typing.TypeVar("T", int, str)
assert T.__constraints__ == (int, str), "TypeVar('T', int, str).__constraints__"
assert T.__bound__ is None, "a constrained TypeVar has no bound"
print("typevar_constraints_recorded OK")
"###);
    assert_output(&out, r###"typevar_constraints_recorded OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/typing/typevar_name_and_repr.py`.
#[test]
fn test_gen_behavior_std_libs_typing_typevar_name_and_repr() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "typing"
# dimension = "behavior"
# case = "typevar_name_and_repr"
# subject = "typing.TypeVar"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_typing.py"
# status = "filled"
# ///
"""typing.TypeVar: a bare TypeVar('T') records its name in __name__ and has neither bound nor constraints"""
import typing

T = typing.TypeVar("T")
assert T.__name__ == "T", "TypeVar.__name__ should be 'T'"
assert T.__bound__ is None, "a bare TypeVar has no bound"
assert T.__constraints__ == (), "a bare TypeVar has no constraints"
print("typevar_name_and_repr OK")
"###);
    assert_output(&out, r###"typevar_name_and_repr OK
"###);
}
