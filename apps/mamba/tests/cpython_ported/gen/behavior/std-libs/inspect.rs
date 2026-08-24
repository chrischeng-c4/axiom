use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/inspect/bind_captures_args_in_order.py`.
#[test]
fn test_gen_behavior_std_libs_inspect_bind_captures_args_in_order() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "bind_captures_args_in_order"
# subject = "inspect.Signature"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.Signature: Signature.bind() captures args in declaration order, folding extras into *args/**kwargs, and BoundArguments.args/.kwargs split them"""
import inspect

def mixed(a, *args, b, z=100, **kwargs):
    pass

sig = inspect.signature(mixed)
ba = sig.bind(10, 20, b=30, c=40)
assert tuple(ba.arguments.items()) == (
    ("a", 10),
    ("args", (20,)),
    ("b", 30),
    ("kwargs", {"c": 40}),
), f"arguments = {tuple(ba.arguments.items())!r}"
assert ba.args == (10, 20), f"ba.args = {ba.args!r}"
assert ba.kwargs == {"b": 30, "c": 40}, f"ba.kwargs = {ba.kwargs!r}"

print("bind_captures_args_in_order OK")
"###);
    assert_output(&out, r###"bind_captures_args_in_order OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/inspect/bind_positional_only_and_same_named_keyword.py`.
#[test]
fn test_gen_behavior_std_libs_inspect_bind_positional_only_and_same_named_keyword() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "bind_positional_only_and_same_named_keyword"
# subject = "inspect.Signature"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.Signature: a positional-only parameter and a same-named keyword do not collide: the positional binds the param, the keyword lands in **kwargs"""
import inspect

def posonly(bar, /, **kwargs):
    pass

sig3 = inspect.signature(posonly)
res = sig3.bind("pos-only", bar="keyword")
assert ("bar", "pos-only") in res.arguments.items(), "posonly captured positionally"
assert res.kwargs == {"bar": "keyword"}, f"posonly kwargs = {res.kwargs!r}"

print("bind_positional_only_and_same_named_keyword OK")
"###);
    assert_output(&out, r###"bind_positional_only_and_same_named_keyword OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/inspect/bound_arguments_apply_defaults.py`.
#[test]
fn test_gen_behavior_std_libs_inspect_bound_arguments_apply_defaults() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "bound_arguments_apply_defaults"
# subject = "inspect.Signature"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.Signature: BoundArguments.apply_defaults() fills missing defaults including empty *args and **kwargs"""
import inspect

def defs(a, b=1, *args, c=7, **kw):
    pass

sig2 = inspect.signature(defs)
ba2 = sig2.bind(20)
ba2.apply_defaults()
assert list(ba2.arguments.items()) == [
    ("a", 20),
    ("b", 1),
    ("args", ()),
    ("c", 7),
    ("kw", {}),
], f"defaults = {list(ba2.arguments.items())!r}"

print("bound_arguments_apply_defaults OK")
"###);
    assert_output(&out, r###"bound_arguments_apply_defaults OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/inspect/bound_arguments_equality_ignores_kwarg_order.py`.
#[test]
fn test_gen_behavior_std_libs_inspect_bound_arguments_equality_ignores_kwarg_order() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "bound_arguments_equality_ignores_kwarg_order"
# subject = "inspect.Signature"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.Signature: BoundArguments equality is by (signature, arguments) and ignores kwarg order; differing values are unequal"""
import inspect

def kw(*, a, b):
    pass

sigk = inspect.signature(kw)
b1 = sigk.bind(a=1, b=2)
b2 = sigk.bind(b=2, a=1)
assert b1 == b2, "kwarg-order-independent equality"

b3 = sigk.bind(a=1, b=3)
assert b1 != b3, "differing values not equal"

print("bound_arguments_equality_ignores_kwarg_order OK")
"###);
    assert_output(&out, r###"bound_arguments_equality_ignores_kwarg_order OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/inspect/cleandoc_dedents_uniform_indentation.py`.
#[test]
fn test_gen_behavior_std_libs_inspect_cleandoc_dedents_uniform_indentation() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "cleandoc_dedents_uniform_indentation"
# subject = "inspect.cleandoc"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.cleandoc: cleandoc() strips uniform leading indentation and the leading blank line"""
import inspect

assert (
    inspect.cleandoc("An\n    indented\n    docstring.")
    == "An\nindented\ndocstring."
), "cleandoc dedent"

print("cleandoc_dedents_uniform_indentation OK")
"###);
    assert_output(&out, r###"cleandoc_dedents_uniform_indentation OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/inspect/currentframe_returns_frame_object.py`.
#[test]
fn test_gen_behavior_std_libs_inspect_currentframe_returns_frame_object() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "currentframe_returns_frame_object"
# subject = "inspect.currentframe"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.currentframe: currentframe() returns a frame object exposing f_code, f_locals, and f_lineno"""
import inspect

_frame = inspect.currentframe()
assert _frame is not None, "currentframe not None"
assert hasattr(_frame, "f_code"), "frame has f_code"
assert hasattr(_frame, "f_locals"), "frame has f_locals"
assert hasattr(_frame, "f_lineno"), "frame has f_lineno"

print("currentframe_returns_frame_object OK")
"###);
    assert_output(&out, r###"currentframe_returns_frame_object OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/inspect/gen_state_constants_distinct_repr.py`.
#[test]
fn test_gen_behavior_std_libs_inspect_gen_state_constants_distinct_repr() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "gen_state_constants_distinct_repr"
# subject = "inspect"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect: generator-state constants GEN_CREATED/GEN_RUNNING/GEN_SUSPENDED/GEN_CLOSED are distinct and self-describing in repr"""
import inspect

for name in ("GEN_CREATED", "GEN_RUNNING", "GEN_SUSPENDED", "GEN_CLOSED"):
    state = getattr(inspect, name)
    assert name in repr(state), f"{name} missing from repr"

print("gen_state_constants_distinct_repr OK")
"###);
    assert_output(&out, r###"gen_state_constants_distinct_repr OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/inspect/get_annotations_raw_and_eval_str.py`.
#[test]
fn test_gen_behavior_std_libs_inspect_get_annotations_raw_and_eval_str() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "get_annotations_raw_and_eval_str"
# subject = "inspect.get_annotations"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.get_annotations: get_annotations() returns raw annotations by default and resolves stringized ones only with eval_str=True; an unannotated object yields {}"""
import inspect

def fn(a: int, b: str) -> bool:
    return True

assert inspect.get_annotations(fn) == {"a": int, "b": str, "return": bool}, "raw annos"

fn.__annotations__ = {"a": "int", "b": "str"}
assert inspect.get_annotations(fn) == {"a": "int", "b": "str"}, "stringized raw"
assert inspect.get_annotations(fn, eval_str=True) == {"a": int, "b": str}, "stringized eval"

def plain(x):
    return x

assert inspect.get_annotations(plain) == {}, "no annotations"

print("get_annotations_raw_and_eval_str OK")
"###);
    assert_output(&out, r###"get_annotations_raw_and_eval_str OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/inspect/getattr_static_does_not_fire_descriptors.py`.
#[test]
fn test_gen_behavior_std_libs_inspect_getattr_static_does_not_fire_descriptors() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "getattr_static_does_not_fire_descriptors"
# subject = "inspect.getattr_static"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.getattr_static: getattr_static() reads an attribute without firing descriptors: a property returns the property object, a slot returns the member descriptor, a default applies when missing"""
import inspect

class _Desc:
    cls_attr = object()

    @property
    def prop(self):
        return 42

_d = _Desc()
# Property: static access returns the descriptor itself, not 42.
_static = inspect.getattr_static(_d, "prop")
assert isinstance(_static, property), f"static prop = {type(_static)!r}"
# Plain class attribute: same identity.
assert inspect.getattr_static(_d, "cls_attr") is _Desc.cls_attr, "static class attr"
# Missing attribute: default applies, otherwise AttributeError.
assert inspect.getattr_static(_d, "missing", "fallback") == "fallback", "static default"
_raised = False
try:
    inspect.getattr_static(_d, "missing")
except AttributeError:
    _raised = True
assert _raised, "expected AttributeError without default"

# Slot: dynamic read returns the value, static read returns the member descriptor.
class Slotted:
    __slots__ = ("x",)

s = Slotted()
s.x = 7
assert getattr(s, "x") == 7, "dynamic slot value"
assert inspect.isdatadescriptor(inspect.getattr_static(s, "x")), "static slot is descriptor"

print("getattr_static_does_not_fire_descriptors OK")
"###);
    assert_output(&out, r###"getattr_static_does_not_fire_descriptors OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/inspect/getclasstree_builds_nested_hierarchy.py`.
#[test]
fn test_gen_behavior_std_libs_inspect_getclasstree_builds_nested_hierarchy() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "getclasstree_builds_nested_hierarchy"
# subject = "inspect.getclasstree"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.getclasstree: getclasstree() builds a nested inheritance hierarchy with object at the top and subclasses nested under their base"""
import inspect

class A:
    pass

class B(A):
    pass

class C(A):
    pass

tree = inspect.getclasstree([A, B, C])
assert tree[0] == (object, ()), f"tree root = {tree[0]!r}"
under_object = tree[1]
assert (A, (object,)) in under_object, "A nested under object"
under_a = under_object[1]
assert (B, (A,)) in under_a, "B nested under A"
assert (C, (A,)) in under_a, "C nested under A"

print("getclasstree_builds_nested_hierarchy OK")
"###);
    assert_output(&out, r###"getclasstree_builds_nested_hierarchy OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/inspect/getclosurevars_reports_nonlocals_and_builtins.py`.
#[test]
fn test_gen_behavior_std_libs_inspect_getclosurevars_reports_nonlocals_and_builtins() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "getclosurevars_reports_nonlocals_and_builtins"
# subject = "inspect.getclosurevars"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.getclosurevars: getclosurevars() reports nonlocal and builtin names referenced by a closure; an empty closure yields all-empty ClosureVars"""
import inspect

def make_closure():
    captured = 0

    def inner():
        return len(str(captured))

    return inner

cv = inspect.getclosurevars(make_closure())
assert cv.nonlocals == {"captured": 0}, f"nonlocals = {cv.nonlocals!r}"
assert "len" in cv.builtins, f"builtins = {cv.builtins!r}"

# Empty closure -> all-empty ClosureVars.
empty = inspect.ClosureVars({}, {}, {}, set())
assert inspect.getclosurevars(lambda: True) == empty, "empty closure"

print("getclosurevars_reports_nonlocals_and_builtins OK")
"###);
    assert_output(&out, r###"getclosurevars_reports_nonlocals_and_builtins OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/inspect/getgeneratorlocals_reflects_live_frame.py`.
#[test]
fn test_gen_behavior_std_libs_inspect_getgeneratorlocals_reflects_live_frame() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "getgeneratorlocals_reflects_live_frame"
# subject = "inspect.getgeneratorlocals"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.getgeneratorlocals: getgeneratorlocals() reflects the generator's live frame locals before and after stepping it once"""
import inspect

def gen(seq, a=None):
    for v in seq:
        yield v

g = gen([1, 2])
assert inspect.getgeneratorlocals(g) == {"a": None, "seq": [1, 2]}, "locals before run"
next(g)
assert inspect.getgeneratorlocals(g) == {"a": None, "seq": [1, 2], "v": 1}, "locals after step"

print("getgeneratorlocals_reflects_live_frame OK")
"###);
    assert_output(&out, r###"getgeneratorlocals_reflects_live_frame OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/inspect/getmembers_predicate_distinguishes_bound_methods.py`.
#[test]
fn test_gen_behavior_std_libs_inspect_getmembers_predicate_distinguishes_bound_methods() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "getmembers_predicate_distinguishes_bound_methods"
# subject = "inspect.getmembers"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.getmembers: getmembers(predicate=ismethod) yields bound methods on an instance but not the plain function on the class"""
import inspect

class _Holder:
    def m(self):
        pass

# On the class, m is a plain function (not a bound method).
assert ("m", _Holder.m) in inspect.getmembers(_Holder), "function in class members"
assert ("m", _Holder.m) not in inspect.getmembers(_Holder, inspect.ismethod), (
    "class function is not ismethod"
)
# On an instance, m is a bound method.
_h = _Holder()
assert ("m", _h.m) in inspect.getmembers(_h, inspect.ismethod), "instance method ismethod"

print("getmembers_predicate_distinguishes_bound_methods OK")
"###);
    assert_output(&out, r###"getmembers_predicate_distinguishes_bound_methods OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/inspect/isabstract_true_only_for_unimplemented_abc.py`.
#[test]
fn test_gen_behavior_std_libs_inspect_isabstract_true_only_for_unimplemented_abc() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "isabstract_true_only_for_unimplemented_abc"
# subject = "inspect.isabstract"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.isabstract: isabstract is True only for an ABC with an unimplemented abstractmethod; False for a concrete subclass, an instance, and a builtin"""
import inspect
from abc import ABCMeta, abstractmethod

class AbstractBase(metaclass=ABCMeta):
    @abstractmethod
    def foo(self):
        pass

class Concrete(AbstractBase):
    def foo(self):
        pass

assert inspect.isabstract(AbstractBase), "abstract class"
assert not inspect.isabstract(Concrete), "concrete subclass"
assert not inspect.isabstract(Concrete()), "instance is not abstract"
assert not inspect.isabstract(int), "builtin not abstract"

print("isabstract_true_only_for_unimplemented_abc OK")
"###);
    assert_output(&out, r###"isabstract_true_only_for_unimplemented_abc OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/inspect/isbuiltin_true_for_len.py`.
#[test]
fn test_gen_behavior_std_libs_inspect_isbuiltin_true_for_len() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "isbuiltin_true_for_len"
# subject = "inspect.isbuiltin"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.isbuiltin: isbuiltin is True for a builtin like len"""
import inspect

assert inspect.isbuiltin(len), "isbuiltin(len)"

print("isbuiltin_true_for_len OK")
"###);
    assert_output(&out, r###"isbuiltin_true_for_len OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/inspect/isclass_true_for_class_false_for_instance.py`.
#[test]
fn test_gen_behavior_std_libs_inspect_isclass_true_for_class_false_for_instance() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "isclass_true_for_class_false_for_instance"
# subject = "inspect.isclass"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.isclass: isclass is True for a class object, False for an instance of it and for a function"""
import inspect

class _MyClass:
    pass

def _f():
    pass

assert inspect.isclass(_MyClass), "isclass(class)"
assert not inspect.isclass(_MyClass()), "not isclass(instance)"
assert not inspect.isclass(_f), "function is not class"

print("isclass_true_for_class_false_for_instance OK")
"###);
    assert_output(&out, r###"isclass_true_for_class_false_for_instance OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/inspect/isdatadescriptor_property_and_slot.py`.
#[test]
fn test_gen_behavior_std_libs_inspect_isdatadescriptor_property_and_slot() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "isdatadescriptor_property_and_slot"
# subject = "inspect.isdatadescriptor"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.isdatadescriptor: isdatadescriptor is True for a property and a __slots__ member descriptor, False for a class and a plain function"""
import inspect

class WithProp:
    @property
    def a_property(self):
        return 1

class Slotted:
    __slots__ = ("x",)

assert inspect.isdatadescriptor(WithProp.a_property), "property is data descriptor"
assert inspect.isdatadescriptor(Slotted.x), "slot is data descriptor"
assert not inspect.isdatadescriptor(WithProp), "class is not data descriptor"
assert not inspect.isdatadescriptor(lambda: 0), "function is not data descriptor"

print("isdatadescriptor_property_and_slot OK")
"###);
    assert_output(&out, r###"isdatadescriptor_property_and_slot OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/inspect/isfunction_true_for_def_false_for_int.py`.
#[test]
fn test_gen_behavior_std_libs_inspect_isfunction_true_for_def_false_for_int() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "isfunction_true_for_def_false_for_int"
# subject = "inspect.isfunction"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.isfunction: isfunction is True for a def, False for a non-function (int) and for a class"""
import inspect

def _myfunc(x):
    return x

class _MyClass:
    pass

assert inspect.isfunction(_myfunc), "isfunction(func)"
assert not inspect.isfunction(42), "not isfunction(int)"
assert not inspect.isfunction(_MyClass), "class is not function"

print("isfunction_true_for_def_false_for_int OK")
"###);
    assert_output(&out, r###"isfunction_true_for_def_false_for_int OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/inspect/ismethod_bound_vs_unbound.py`.
#[test]
fn test_gen_behavior_std_libs_inspect_ismethod_bound_vs_unbound() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "ismethod_bound_vs_unbound"
# subject = "inspect.ismethod"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.ismethod: ismethod is True for a bound method on an instance, False for the plain function accessed on the class"""
import inspect

class _Owner:
    def method(self):
        pass

_o = _Owner()
assert inspect.ismethod(_o.method), "ismethod(bound method)"
assert not inspect.ismethod(_Owner.method), "not ismethod(unbound)"

print("ismethod_bound_vs_unbound OK")
"###);
    assert_output(&out, r###"ismethod_bound_vs_unbound OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/inspect/ismodule_true_for_module_false_for_func.py`.
#[test]
fn test_gen_behavior_std_libs_inspect_ismodule_true_for_module_false_for_func() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "ismodule_true_for_module_false_for_func"
# subject = "inspect.ismodule"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.ismodule: ismodule is True for an imported module, False for a function"""
import inspect
import math

def _myfunc(x):
    return x

assert inspect.ismodule(math), "ismodule(math)"
assert not inspect.ismodule(_myfunc), "not ismodule(func)"

print("ismodule_true_for_module_false_for_func OK")
"###);
    assert_output(&out, r###"ismodule_true_for_module_false_for_func OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/inspect/isroutine_function_builtin_singledispatch.py`.
#[test]
fn test_gen_behavior_std_libs_inspect_isroutine_function_builtin_singledispatch() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "isroutine_function_builtin_singledispatch"
# subject = "inspect.isroutine"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.isroutine: isroutine is True for a function, a builtin, and a functools.singledispatch wrapper, False for an int"""
import functools
import inspect

def f():
    pass

assert inspect.isroutine(f), "function is routine"
assert inspect.isroutine(len), "builtin is routine"
assert inspect.isroutine(functools.singledispatch(f)), "singledispatch is routine"
assert not inspect.isroutine(42), "int is not routine"

print("isroutine_function_builtin_singledispatch OK")
"###);
    assert_output(&out, r###"isroutine_function_builtin_singledispatch OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/inspect/parameter_attributes_reflect_constructor.py`.
#[test]
fn test_gen_behavior_std_libs_inspect_parameter_attributes_reflect_constructor() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "parameter_attributes_reflect_constructor"
# subject = "inspect.Parameter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.Parameter: Parameter attributes (name, default, kind) reflect the constructor; a missing annotation is the empty sentinel"""
import inspect

P = inspect.Parameter

p = P("foo", default=10, kind=P.POSITIONAL_ONLY)
assert p.name == "foo", f"name = {p.name!r}"
assert p.default == 10, f"default = {p.default!r}"
assert p.kind == P.POSITIONAL_ONLY, f"kind = {p.kind!r}"
assert p.annotation is P.empty, "annotation is empty sentinel"

print("parameter_attributes_reflect_constructor OK")
"###);
    assert_output(&out, r###"parameter_attributes_reflect_constructor OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/inspect/parameter_default_and_empty_sentinel.py`.
#[test]
fn test_gen_behavior_std_libs_inspect_parameter_default_and_empty_sentinel() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "parameter_default_and_empty_sentinel"
# subject = "inspect.Parameter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.Parameter: Parameter.default reflects the declared default; a parameter without one reports inspect.Parameter.empty"""
import inspect

def _func(a, b, c=3):
    return a + b + c

_params = inspect.signature(_func).parameters
assert _params["c"].default == 3, f"default c = {_params['c'].default!r}"
assert _params["a"].default is inspect.Parameter.empty, "a has no default"

print("parameter_default_and_empty_sentinel OK")
"###);
    assert_output(&out, r###"parameter_default_and_empty_sentinel OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/inspect/parameter_immutable_attribute_assignment.py`.
#[test]
fn test_gen_behavior_std_libs_inspect_parameter_immutable_attribute_assignment() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "parameter_immutable_attribute_assignment"
# subject = "inspect.Parameter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.Parameter: Parameter instances are immutable: assigning an attribute raises AttributeError"""
import inspect

P = inspect.Parameter

imm = P("spam", kind=P.KEYWORD_ONLY)
_raised = False
try:
    imm.foo = "bar"
except AttributeError:
    _raised = True
assert _raised, "expected AttributeError on attribute set"

print("parameter_immutable_attribute_assignment OK")
"###);
    assert_output(&out, r###"parameter_immutable_attribute_assignment OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/inspect/parameter_kind_ordering_and_str.py`.
#[test]
fn test_gen_behavior_std_libs_inspect_parameter_kind_ordering_and_str() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "parameter_kind_ordering_and_str"
# subject = "inspect.Parameter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.Parameter: Parameter kinds have a defined ordering POSITIONAL_ONLY < POSITIONAL_OR_KEYWORD < VAR_POSITIONAL < KEYWORD_ONLY < VAR_KEYWORD and a readable str form"""
import inspect

P = inspect.Parameter

assert P.POSITIONAL_ONLY < P.POSITIONAL_OR_KEYWORD < P.VAR_POSITIONAL, "kind order lo"
assert P.VAR_POSITIONAL < P.KEYWORD_ONLY < P.VAR_KEYWORD, "kind order hi"
assert str(P.POSITIONAL_ONLY) == "POSITIONAL_ONLY", "kind str"

print("parameter_kind_ordering_and_str OK")
"###);
    assert_output(&out, r###"parameter_kind_ordering_and_str OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/inspect/parameter_replace_is_non_mutating.py`.
#[test]
fn test_gen_behavior_std_libs_inspect_parameter_replace_is_non_mutating() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "parameter_replace_is_non_mutating"
# subject = "inspect.Parameter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.Parameter: Parameter.replace() returns a new object: no-arg replace is equal, a renamed one differs"""
import inspect

P = inspect.Parameter

q = P("foo", default=42, kind=P.KEYWORD_ONLY)
assert q is not q.replace(), "replace returns a new object"
assert q == q.replace(), "replace() with no args is equal"
assert q.replace(name="bar").name == "bar", "replace name"
assert q.replace(name="bar") != q, "renamed parameter not equal"

print("parameter_replace_is_non_mutating OK")
"###);
    assert_output(&out, r###"parameter_replace_is_non_mutating OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/inspect/parameter_repr_embeds_name_default.py`.
#[test]
fn test_gen_behavior_std_libs_inspect_parameter_repr_embeds_name_default() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "parameter_repr_embeds_name_default"
# subject = "inspect.Parameter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.Parameter: repr(Parameter) starts with '<Parameter' and embeds the rendered name=default form"""
import inspect

P = inspect.Parameter

r = P("a", default=42, kind=P.POSITIONAL_OR_KEYWORD)
assert repr(r).startswith("<Parameter"), f"repr = {repr(r)!r}"
assert "a=42" in repr(r), f"repr lacks a=42: {repr(r)!r}"

print("parameter_repr_embeds_name_default OK")
"###);
    assert_output(&out, r###"parameter_repr_embeds_name_default OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/inspect/signature_captures_parameter_names_in_order.py`.
#[test]
fn test_gen_behavior_std_libs_inspect_signature_captures_parameter_names_in_order() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "signature_captures_parameter_names_in_order"
# subject = "inspect.signature"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.signature: signature() lists positional/keyword parameter names in declaration order"""
import inspect

def _func(a, b, c=3):
    return a + b + c

_sig = inspect.signature(_func)
_names = list(_sig.parameters.keys())
assert _names == ["a", "b", "c"], f"param names = {_names!r}"

print("signature_captures_parameter_names_in_order OK")
"###);
    assert_output(&out, r###"signature_captures_parameter_names_in_order OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/inspect/signature_empty_renders_parens.py`.
#[test]
fn test_gen_behavior_std_libs_inspect_signature_empty_renders_parens() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "signature_empty_renders_parens"
# subject = "inspect.Signature"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.Signature: an empty Signature() and the signature of a no-arg lambda both render as '()'"""
import inspect

S = inspect.Signature
assert str(S()) == "()", f"empty sig = {str(S())!r}"
assert str(inspect.signature(lambda: None)) == "()", "empty lambda sig"

print("signature_empty_renders_parens OK")
"###);
    assert_output(&out, r###"signature_empty_renders_parens OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/inspect/signature_equality_and_hash.py`.
#[test]
fn test_gen_behavior_std_libs_inspect_signature_equality_and_hash() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "signature_equality_and_hash"
# subject = "inspect.signature"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.signature: two signatures with identical parameters/annotations are equal and hash-equal; a differing return annotation makes them unequal"""
import inspect

def g1(a, *, b: int) -> float:
    pass

def g2(a, *, b: int) -> float:
    pass

def g3(a, *, b: int) -> int:  # different return annotation
    pass

assert inspect.signature(g1) == inspect.signature(g2), "same sig equal"
assert hash(inspect.signature(g1)) == hash(inspect.signature(g2)), "same sig hash"
assert inspect.signature(g1) != inspect.signature(g3), "return anno differs"

print("signature_equality_and_hash OK")
"###);
    assert_output(&out, r###"signature_equality_and_hash OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/inspect/signature_keyword_only_after_star.py`.
#[test]
fn test_gen_behavior_std_libs_inspect_signature_keyword_only_after_star() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "signature_keyword_only_after_star"
# subject = "inspect.signature"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.signature: parameters declared after a bare * are KEYWORD_ONLY and keep their defaults"""
import inspect

def _kwonly(a, *, b, c=10):
    pass

_kp = inspect.signature(_kwonly).parameters
assert _kp["b"].kind == inspect.Parameter.KEYWORD_ONLY, "b is KEYWORD_ONLY"
assert _kp["c"].kind == inspect.Parameter.KEYWORD_ONLY, "c is KEYWORD_ONLY"
assert _kp["c"].default == 10, "c default = 10"

print("signature_keyword_only_after_star OK")
"###);
    assert_output(&out, r###"signature_keyword_only_after_star OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/inspect/signature_manual_from_parameter_list.py`.
#[test]
fn test_gen_behavior_std_libs_inspect_signature_manual_from_parameter_list() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "signature_manual_from_parameter_list"
# subject = "inspect.Signature"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.Signature: a Signature assembled from a Parameter list renders the same as a parsed function signature"""
import inspect

S = inspect.Signature
P = inspect.Parameter

assert str(S(parameters=[P("foo", P.POSITIONAL_ONLY)])) == "(foo, /)", "manual posonly"
assert (
    str(S(parameters=[P("foo", P.POSITIONAL_ONLY), P("bar", P.VAR_KEYWORD)]))
    == "(foo, /, **bar)"
), "manual posonly + **kw"

print("signature_manual_from_parameter_list OK")
"###);
    assert_output(&out, r###"signature_manual_from_parameter_list OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/inspect/signature_str_positional_only_marker.py`.
#[test]
fn test_gen_behavior_std_libs_inspect_signature_str_positional_only_marker() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "signature_str_positional_only_marker"
# subject = "inspect.signature"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.signature: str(signature) renders the positional-only '/' marker correctly"""
import inspect

def f2(a_po, /, *, b, **kwargs):
    pass

assert str(inspect.signature(f2)) == "(a_po, /, *, b, **kwargs)", "posonly str"

print("signature_str_positional_only_marker OK")
"###);
    assert_output(&out, r###"signature_str_positional_only_marker OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/inspect/signature_str_renders_defaults_kwonly_varargs_return.py`.
#[test]
fn test_gen_behavior_std_libs_inspect_signature_str_renders_defaults_kwonly_varargs_return() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "signature_str_renders_defaults_kwonly_varargs_return"
# subject = "inspect.signature"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.signature: str(signature) renders defaults, the keyword-only marker, *args/**kwargs, and the return annotation"""
import inspect

def f1(a: int = 1, *, b, c=None, **kwargs) -> 42:
    pass

assert str(inspect.signature(f1)) == "(a: int = 1, *, b, c=None, **kwargs) -> 42", (
    f"f1 str = {str(inspect.signature(f1))!r}"
)

print("signature_str_renders_defaults_kwonly_varargs_return OK")
"###);
    assert_output(&out, r###"signature_str_renders_defaults_kwonly_varargs_return OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/inspect/signature_var_positional_and_var_keyword_kinds.py`.
#[test]
fn test_gen_behavior_std_libs_inspect_signature_var_positional_and_var_keyword_kinds() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "inspect"
# dimension = "behavior"
# case = "signature_var_positional_and_var_keyword_kinds"
# subject = "inspect.signature"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""inspect.signature: signature() classifies *args as VAR_POSITIONAL and **kwargs as VAR_KEYWORD"""
import inspect

def _variadic(*args, **kwargs):
    pass

_vp = inspect.signature(_variadic).parameters
assert "args" in _vp, "args in variadic"
assert "kwargs" in _vp, "kwargs in variadic"
assert _vp["args"].kind == inspect.Parameter.VAR_POSITIONAL, "VAR_POSITIONAL"
assert _vp["kwargs"].kind == inspect.Parameter.VAR_KEYWORD, "VAR_KEYWORD"

print("signature_var_positional_and_var_keyword_kinds OK")
"###);
    assert_output(&out, r###"signature_var_positional_and_var_keyword_kinds OK
"###);
}
