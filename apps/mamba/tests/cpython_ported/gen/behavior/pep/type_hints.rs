use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/pep/type_hints/annotated_final_classvar_exist.py`.
#[test]
fn test_gen_behavior_pep_type_hints_annotated_final_classvar_exist() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "type_hints"
# dimension = "behavior"
# case = "annotated_final_classvar_exist"
# subject = "typing"
# kind = "semantic"
# xfail = "mamba diverges on the typing special-form surface (project_mamba_class_machinery_silent_divergences)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing: the documented PEP 484/586/591 special forms are present on the typing module: hasattr(typing,'Annotated'), hasattr(typing,'Final') and hasattr(typing,'ClassVar') are all True"""
import typing

assert hasattr(typing, "Annotated"), "typing.Annotated exists"
assert hasattr(typing, "Final"), "typing.Final exists"
assert hasattr(typing, "ClassVar"), "typing.ClassVar exists"

print("annotated_final_classvar_exist OK")
"###);
    assert_output(&out, r###"annotated_final_classvar_exist OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/type_hints/annotated_function_runs_and_records_annotations.py`.
#[test]
fn test_gen_behavior_pep_type_hints_annotated_function_runs_and_records_annotations() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "type_hints"
# dimension = "behavior"
# case = "annotated_function_runs_and_records_annotations"
# subject = "typing.get_type_hints"
# kind = "semantic"
# xfail = "mamba returns None for a function's __annotations__ (project_mamba_pep_silent_divergences_2026_05_27)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.get_type_hints: an int->int annotated function runs normally and stores its hints: _add(a:int,b:int)->int returns 5 for (2,3), is callable, and 'return' is in _add.__annotations__"""
import typing


def _add(a: int, b: int) -> int:
    return a + b


assert callable(_add), "_add callable"
assert _add(2, 3) == 5, f"add = {_add(2, 3)!r}"
assert "return" in _add.__annotations__, "return in annotations"

print("annotated_function_runs_and_records_annotations OK")
"###);
    assert_output(&out, r###"annotated_function_runs_and_records_annotations OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/type_hints/callable_annotation_applies_function.py`.
#[test]
fn test_gen_behavior_pep_type_hints_callable_annotation_applies_function() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "type_hints"
# dimension = "behavior"
# case = "callable_annotation_applies_function"
# subject = "typing.Callable"
# kind = "semantic"
# xfail = "mamba diverges on the typing Callable runtime machinery (project_mamba_class_machinery_silent_divergences)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.Callable: a Callable[[int],int]-annotated parameter is just a function at runtime: _apply(lambda x: x*2, 5)==10"""
import typing
from typing import Callable


def _apply(fn: Callable[[int], int], v: int) -> int:
    return fn(v)


assert _apply(lambda x: x * 2, 5) == 10, f"apply = {_apply(lambda x: x * 2, 5)!r}"

print("callable_annotation_applies_function OK")
"###);
    assert_output(&out, r###"callable_annotation_applies_function OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/type_hints/cast_is_runtime_noop.py`.
#[test]
fn test_gen_behavior_pep_type_hints_cast_is_runtime_noop() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "type_hints"
# dimension = "behavior"
# case = "cast_is_runtime_noop"
# subject = "typing.cast"
# kind = "semantic"
# xfail = "mamba diverges on the typing cast runtime machinery (project_mamba_class_machinery_silent_divergences)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.cast: cast is a runtime no-op returning its argument unchanged with no coercion: cast(int,'hello')=='hello' and cast(int,'still a str')=='still a str'"""
import typing
from typing import cast

# cast is a no-op at runtime: the value passes through unchanged, uncoerced.
assert cast(int, "hello") == "hello", f"cast no-op = {cast(int, 'hello')!r}"
assert cast(int, "still a str") == "still a str", "cast no-op str"

print("cast_is_runtime_noop OK")
"###);
    assert_output(&out, r###"cast_is_runtime_noop OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/type_hints/dict_annotated_word_count.py`.
#[test]
fn test_gen_behavior_pep_type_hints_dict_annotated_word_count() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "type_hints"
# dimension = "behavior"
# case = "dict_annotated_word_count"
# subject = "typing.Dict"
# kind = "semantic"
# xfail = "mamba diverges on the typing generic-alias runtime machinery (project_mamba_class_machinery_silent_divergences)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.Dict: a Dict[str,int]-annotated function returns a plain dict: _count_words('a b a') returns {'a':2,'b':1}"""
import typing
from typing import Dict


def _count_words(text: str) -> Dict[str, int]:
    result: Dict[str, int] = {}
    for w in text.split():
        result[w] = result.get(w, 0) + 1
    return result


assert _count_words("a b a") == {"a": 2, "b": 1}, f"word count = {_count_words('a b a')!r}"

print("dict_annotated_word_count OK")
"###);
    assert_output(&out, r###"dict_annotated_word_count OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/type_hints/generic_box_holds_value.py`.
#[test]
fn test_gen_behavior_pep_type_hints_generic_box_holds_value() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "type_hints"
# dimension = "behavior"
# case = "generic_box_holds_value"
# subject = "typing.Generic"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.Generic: a Generic[T] subclass is an ordinary class at runtime: _Box(Generic[T]) stores and returns any value, _Box(42).get()==42 and _Box('hello').get()=='hello'"""
import typing
from typing import Generic, TypeVar

T = TypeVar("T")


class _Box(Generic[T]):
    def __init__(self, value: T):
        self.value = value

    def get(self) -> T:
        return self.value


assert _Box(42).get() == 42, f"generic get = {_Box(42).get()!r}"
assert _Box("hello").get() == "hello", f"generic str = {_Box('hello').get()!r}"

print("generic_box_holds_value OK")
"###);
    assert_output(&out, r###"generic_box_holds_value OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/type_hints/get_type_hints_returns_param_dict.py`.
#[test]
fn test_gen_behavior_pep_type_hints_get_type_hints_returns_param_dict() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "type_hints"
# dimension = "behavior"
# case = "get_type_hints_returns_param_dict"
# subject = "typing.get_type_hints"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.get_type_hints: get_type_hints returns a dict of resolved annotations: for _typed(a:int,b:Optional[str]=None)->List[int] it is a dict containing 'a', 'b' and 'return'"""
import typing
from typing import List, Optional


def _typed(a: int, b: Optional[str] = None) -> List[int]:
    return []


_hints = typing.get_type_hints(_typed)
assert isinstance(_hints, dict), f"hints type = {type(_hints)!r}"
assert "a" in _hints, "a in hints"
assert "b" in _hints, "b in hints"
assert "return" in _hints, "return in hints"

print("get_type_hints_returns_param_dict OK")
"###);
    assert_output(&out, r###"get_type_hints_returns_param_dict OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/type_hints/list_int_alias_origin_is_list.py`.
#[test]
fn test_gen_behavior_pep_type_hints_list_int_alias_origin_is_list() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "type_hints"
# dimension = "behavior"
# case = "list_int_alias_origin_is_list"
# subject = "typing.List"
# kind = "semantic"
# xfail = "mamba diverges on the typing generic-alias runtime machinery (project_mamba_class_machinery_silent_divergences)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.List: List[int] is a generic alias exposing its origin: hasattr(List[int],'__origin__') is True and List[int].__origin__ is the builtin list"""
import typing
from typing import List

_IntList = List[int]
assert hasattr(_IntList, "__origin__"), "List[int] has __origin__"
assert _IntList.__origin__ is list, f"__origin__ = {_IntList.__origin__!r}"

print("list_int_alias_origin_is_list OK")
"###);
    assert_output(&out, r###"list_int_alias_origin_is_list OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/type_hints/list_int_annotated_sum.py`.
#[test]
fn test_gen_behavior_pep_type_hints_list_int_annotated_sum() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "type_hints"
# dimension = "behavior"
# case = "list_int_annotated_sum"
# subject = "typing.List"
# kind = "semantic"
# xfail = "mamba diverges on the typing generic-alias runtime machinery (project_mamba_class_machinery_silent_divergences)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.List: a List[int]-annotated function works on a normal list: _sum_list(items:List[int])->int returns 6 for [1,2,3]"""
import typing
from typing import List


def _sum_list(items: List[int]) -> int:
    return sum(items)


assert _sum_list([1, 2, 3]) == 6, f"sum_list = {_sum_list([1, 2, 3])!r}"

print("list_int_annotated_sum OK")
"###);
    assert_output(&out, r###"list_int_annotated_sum OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/type_hints/optional_equals_union_with_none.py`.
#[test]
fn test_gen_behavior_pep_type_hints_optional_equals_union_with_none() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "type_hints"
# dimension = "behavior"
# case = "optional_equals_union_with_none"
# subject = "typing.Optional"
# kind = "semantic"
# xfail = "mamba diverges on the typing union/| machinery (project_mamba_pep_silent_divergences_2026_05_27)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.Optional: Optional[X] is exactly Union[X,None]: typing.Optional[int]==Union[int,None] and typing.Optional[str]==Union[str,None]"""
import typing
from typing import Optional, Union

assert Optional[int] == Union[int, None], "Optional[int] == Union[int, None]"
assert Optional[str] == Union[str, None], "Optional[str] == Union[str, None]"

print("optional_equals_union_with_none OK")
"###);
    assert_output(&out, r###"optional_equals_union_with_none OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/type_hints/tuple_annotated_fixed_arity.py`.
#[test]
fn test_gen_behavior_pep_type_hints_tuple_annotated_fixed_arity() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "type_hints"
# dimension = "behavior"
# case = "tuple_annotated_fixed_arity"
# subject = "typing.Tuple"
# kind = "semantic"
# xfail = "mamba diverges on the typing generic-alias runtime machinery (project_mamba_class_machinery_silent_divergences)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.Tuple: a Tuple[int,int]-annotated function returns a real 2-tuple: _divmod2(17,5) unpacks to q==3, r==2"""
import typing
from typing import Tuple


def _divmod2(a: int, b: int) -> Tuple[int, int]:
    return divmod(a, b)


_q, _r = _divmod2(17, 5)
assert _q == 3 and _r == 2, f"divmod = {_q},{_r}"

print("tuple_annotated_fixed_arity OK")
"###);
    assert_output(&out, r###"tuple_annotated_fixed_arity OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/type_hints/typevar_identity_not_enforced.py`.
#[test]
fn test_gen_behavior_pep_type_hints_typevar_identity_not_enforced() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "type_hints"
# dimension = "behavior"
# case = "typevar_identity_not_enforced"
# subject = "typing.TypeVar"
# kind = "semantic"
# xfail = "mamba diverges on the typing TypeVar/runtime machinery (project_mamba_class_machinery_silent_divergences)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.TypeVar: a TypeVar is an unenforced placeholder: T=TypeVar('T'); _identity(x:T)->T returns 42 for 42 and 'hi' for 'hi', and isinstance(T, TypeVar) is True"""
import typing
from typing import TypeVar

T = TypeVar("T")


def _identity(x: T) -> T:
    return x


assert _identity(42) == 42, f"identity int = {_identity(42)!r}"
assert _identity("hi") == "hi", f"identity str = {_identity('hi')!r}"
assert isinstance(T, TypeVar), f"T is TypeVar = {type(T)!r}"

print("typevar_identity_not_enforced OK")
"###);
    assert_output(&out, r###"typevar_identity_not_enforced OK
"###);
}

/// Ported from `tests/cpython/behavior/pep/type_hints/union_annotation_accepts_either_member.py`.
#[test]
fn test_gen_behavior_pep_type_hints_union_annotation_accepts_either_member() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "type_hints"
# dimension = "behavior"
# case = "union_annotation_accepts_either_member"
# subject = "typing.Union"
# kind = "semantic"
# xfail = "mamba diverges on the typing union/| machinery (project_mamba_pep_silent_divergences_2026_05_27)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""typing.Union: a Union[int,str]-annotated function accepts either member at runtime: _first(1)=='1' and _first('a')=='a' (annotation is advisory)"""
import typing
from typing import Union


def _first(v: Union[int, str]) -> str:
    return str(v)


assert _first(1) == "1", f"first(1) = {_first(1)!r}"
assert _first("a") == "a", f"first(a) = {_first('a')!r}"

print("union_annotation_accepts_either_member OK")
"###);
    assert_output(&out, r###"union_annotation_accepts_either_member OK
"###);
}
