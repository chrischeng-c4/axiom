use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/types/col_names_tests__test_cursor_description_insert.py`.
#[test]
fn test_gen_behavior_std_libs_types_col_names_tests__test_cursor_description_insert() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "col_names_tests__test_cursor_description_insert"
# subject = "cpython.test_types.ColNamesTests.test_cursor_description_insert"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_types.py"
# status = "filled"
# ///
import datetime
import sqlite3 as sqlite
import sys
self_con = sqlite.connect(':memory:', detect_types=sqlite.PARSE_COLNAMES)
self_cur = self_con.cursor()
self_cur.execute('create table test(x foo)')
sqlite.converters['FOO'] = lambda x: '[%s]' % x.decode('ascii')
sqlite.converters['BAR'] = lambda x: '<%s>' % x.decode('ascii')
sqlite.converters['EXC'] = lambda x: 5 / 0
sqlite.converters['B1B1'] = lambda x: 'MARKER'
self_cur.execute('insert into test values (1)')
assert self_cur.description is None

print("ColNamesTests::test_cursor_description_insert: ok")
"###);
    assert_output(&out, r###"ColNamesTests::test_cursor_description_insert: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/types/col_names_tests__test_none.py`.
#[test]
fn test_gen_behavior_std_libs_types_col_names_tests__test_none() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "col_names_tests__test_none"
# subject = "cpython.test_types.ColNamesTests.test_none"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_sqlite3/test_types.py"
# status = "filled"
# ///
import datetime
import sqlite3 as sqlite
import sys
self_con = sqlite.connect(':memory:', detect_types=sqlite.PARSE_COLNAMES)
self_cur = self_con.cursor()
self_cur.execute('create table test(x foo)')
sqlite.converters['FOO'] = lambda x: '[%s]' % x.decode('ascii')
sqlite.converters['BAR'] = lambda x: '<%s>' % x.decode('ascii')
sqlite.converters['EXC'] = lambda x: 5 / 0
sqlite.converters['B1B1'] = lambda x: 'MARKER'
self_cur.execute('insert into test(x) values (?)', (None,))
self_cur.execute('select x from test')
val = self_cur.fetchone()[0]
assert val == None

print("ColNamesTests::test_none: ok")
"###);
    assert_output(&out, r###"ColNamesTests::test_none: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/types/coroutine_generator_wrapper_protocol.py`.
#[test]
fn test_gen_behavior_std_libs_types_coroutine_generator_wrapper_protocol() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "coroutine_generator_wrapper_protocol"
# subject = "types.coroutine"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
"""types.coroutine: wrapping a function returning a non-coroutine generator yields a generator-wrapper whose repr==str and which exposes the coroutine/generator protocol (__await__/__iter__/send/close/throw)"""
import types


# Wrapping a plain function that returns a NON-coroutine generator yields a
# generator-wrapper: its repr and str agree and it exposes the coroutine /
# generator protocol (a generator that is already coroutine-flagged would be
# returned bare, so use a fresh, unwrapped generator here).
def plain_gen():
    yield


@types.coroutine
def returns_plain_gen():
    return plain_gen()


wrapper = returns_plain_gen()
assert repr(wrapper) == str(wrapper)
expected = {"__await__", "__iter__", "send", "close", "throw"}
assert expected.issubset(set(dir(wrapper)))

print("coroutine_generator_wrapper_protocol OK")
"###);
    assert_output(&out, r###"coroutine_generator_wrapper_protocol OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/types/coroutine_passes_awaitable_through.py`.
#[test]
fn test_gen_behavior_std_libs_types_coroutine_passes_awaitable_through() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "coroutine_passes_awaitable_through"
# subject = "types.coroutine"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
"""types.coroutine: wrapping a function that returns a duck-typed awaitable passes the awaitable through and __await__ stays consistent"""
import types


class CoroLike:
    def send(self):
        pass

    def throw(self):
        pass

    def close(self):
        pass

    def __await__(self):
        return self


duck = CoroLike()


@types.coroutine
def returns_duck():
    return duck


assert returns_duck() is duck
assert returns_duck().__await__() is duck

print("coroutine_passes_awaitable_through OK")
"###);
    assert_output(&out, r###"coroutine_passes_awaitable_through OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/types/coroutine_passthrough_generator_idempotent.py`.
#[test]
fn test_gen_behavior_std_libs_types_coroutine_passthrough_generator_idempotent() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "coroutine_passthrough_generator_idempotent"
# subject = "types.coroutine"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
"""types.coroutine: a generator function passed through coroutine() is returned unchanged, and re-wrapping is idempotent"""
import types


def gen():
    yield


assert types.coroutine(gen) is gen
assert types.coroutine(types.coroutine(gen)) is gen

print("coroutine_passthrough_generator_idempotent OK")
"###);
    assert_output(&out, r###"coroutine_passthrough_generator_idempotent OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/types/coroutine_sets_iterable_flag.py`.
#[test]
fn test_gen_behavior_std_libs_types_coroutine_sets_iterable_flag() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "coroutine_sets_iterable_flag"
# subject = "types.coroutine"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
"""types.coroutine: coroutine() sets the CO_ITERABLE_COROUTINE code flag without marking the generator a native CO_COROUTINE, visible on the function and a live generator's code object"""
import inspect
import types


def gen():
    yield


types.coroutine(gen)

# coroutine() sets the iterable-coroutine code flag without marking it a
# native coroutine. The flag also shows on a live generator's code object.
assert gen.__code__.co_flags & inspect.CO_ITERABLE_COROUTINE
assert not gen.__code__.co_flags & inspect.CO_COROUTINE
running = gen()
assert running.gi_code.co_flags & inspect.CO_ITERABLE_COROUTINE
assert not running.gi_code.co_flags & inspect.CO_COROUTINE

print("coroutine_sets_iterable_flag OK")
"###);
    assert_output(&out, r###"coroutine_sets_iterable_flag OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/types/new_class_basics.py`.
#[test]
fn test_gen_behavior_std_libs_types_new_class_basics() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "new_class_basics"
# subject = "types.new_class"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
"""types.new_class: new_class('C') builds a fresh class deriving from object, and explicit empty bases/kwds/exec_body default to the same result"""
import types

# new_class('C') builds a fresh class deriving from object.
C = types.new_class("C")
assert C.__name__ == "C"
assert C.__bases__ == (object,)

# Explicit empty bases/kwds/exec-body default to the same result.
D = types.new_class("D", (), {}, None)
assert D.__name__ == "D"
assert D.__bases__ == (object,)

print("new_class_basics OK")
"###);
    assert_output(&out, r###"new_class_basics OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/types/new_class_exec_body.py`.
#[test]
fn test_gen_behavior_std_libs_types_new_class_exec_body() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "new_class_exec_body"
# subject = "types.new_class"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
"""types.new_class: an exec_body callback populates the new class namespace before the class is created"""
import types


def body(ns):
    ns["value"] = 42


E = types.new_class("E", (), {}, body)
assert E.value == 42

print("new_class_exec_body OK")
"###);
    assert_output(&out, r###"new_class_exec_body OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/types/new_class_explicit_base.py`.
#[test]
fn test_gen_behavior_std_libs_types_new_class_explicit_base() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "new_class_explicit_base"
# subject = "types.new_class"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
"""types.new_class: new_class with an explicit base honors that base in __bases__ and inherits its attributes"""
import types


class Base:
    tag = "base"


F = types.new_class("F", (Base,))
assert F.__bases__ == (Base,)
assert F.tag == "base"

print("new_class_explicit_base OK")
"###);
    assert_output(&out, r###"new_class_explicit_base OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/types/resolve_bases_mro_entries.py`.
#[test]
fn test_gen_behavior_std_libs_types_resolve_bases_mro_entries() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "resolve_bases_mro_entries"
# subject = "types.resolve_bases"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
"""types.resolve_bases: resolve_bases expands __mro_entries__ but leaves plain bases untouched, returning the same tuple object when nothing needs resolving"""
import types


class P:
    pass


class Q:
    def __mro_entries__(self, bases):
        return () if P in bases else (P,)


q = Q()
assert types.resolve_bases(()) == ()
assert types.resolve_bases((q,)) == (P,)
assert types.resolve_bases((P,)) == (P,)
assert types.resolve_bases((q, P)) == (P,)
unchanged = (P, Q)
assert types.resolve_bases(unchanged) is unchanged

print("resolve_bases_mro_entries OK")
"###);
    assert_output(&out, r###"resolve_bases_mro_entries OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/types/simplenamespace_attr_lifecycle.py`.
#[test]
fn test_gen_behavior_std_libs_types_simplenamespace_attr_lifecycle() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "simplenamespace_attr_lifecycle"
# subject = "types.SimpleNamespace"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
"""types.SimpleNamespace: attribute set / get / del lifecycle on a SimpleNamespace updates vars(), and deleting an absent attribute raises AttributeError"""
import types

ns = types.SimpleNamespace(a=1, b=2, c=3)
assert ns.a == 1
ns.d = "added"
assert ns.d == "added"
del ns.b
assert vars(ns) == {"a": 1, "c": 3, "d": "added"}

# Deleting an absent attribute raises AttributeError.
_raised = False
try:
    del ns.missing
except AttributeError:
    _raised = True
assert _raised, "deleting an absent attribute should raise AttributeError"

print("simplenamespace_attr_lifecycle OK")
"###);
    assert_output(&out, r###"simplenamespace_attr_lifecycle OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/types/simplenamespace_construction_forms.py`.
#[test]
fn test_gen_behavior_std_libs_types_simplenamespace_construction_forms() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "simplenamespace_construction_forms"
# subject = "types.SimpleNamespace"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
"""types.SimpleNamespace: empty(), kwargs, and **dict construction are equivalent forms whose vars()/__dict__ reflect the supplied attributes"""
import types

ns_empty = types.SimpleNamespace()
ns_kw = types.SimpleNamespace(x=1, y=2)
ns_unpack = types.SimpleNamespace(**dict(x=1, y=2))
assert vars(ns_empty) == {}
assert vars(ns_kw) == {"x": 1, "y": 2}
assert ns_unpack.__dict__ == {"x": 1, "y": 2}
assert len(ns_kw.__dict__) == 2

print("simplenamespace_construction_forms OK")
"###);
    assert_output(&out, r###"simplenamespace_construction_forms OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/types/simplenamespace_equality.py`.
#[test]
fn test_gen_behavior_std_libs_types_simplenamespace_equality() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "simplenamespace_equality"
# subject = "types.SimpleNamespace"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
"""types.SimpleNamespace: SimpleNamespace equality compares the underlying __dict__: two namespaces with equal attrs are ==, two empty ones are ==, and an empty differs from a populated one"""
import types

left = types.SimpleNamespace(x=1)
right = types.SimpleNamespace()
right.x = 1
assert left == right
assert types.SimpleNamespace() == types.SimpleNamespace()
assert right != types.SimpleNamespace()

print("simplenamespace_equality OK")
"###);
    assert_output(&out, r###"simplenamespace_equality OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/types/simplenamespace_nested_by_reference.py`.
#[test]
fn test_gen_behavior_std_libs_types_simplenamespace_nested_by_reference() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "simplenamespace_nested_by_reference"
# subject = "types.SimpleNamespace"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
"""types.SimpleNamespace: a nested SimpleNamespace is stored by reference, not copied, so the inner object is reachable via attribute access and identity-equal in vars()"""
import types

inner = types.SimpleNamespace(a=1, b=2)
outer = types.SimpleNamespace(x=inner)
assert outer.x.a == 1
assert vars(outer) == {"x": inner}
assert outer.x is inner

print("simplenamespace_nested_by_reference OK")
"###);
    assert_output(&out, r###"simplenamespace_nested_by_reference OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/types/simplenamespace_not_a_mapping.py`.
#[test]
fn test_gen_behavior_std_libs_types_simplenamespace_not_a_mapping() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "simplenamespace_not_a_mapping"
# subject = "types.SimpleNamespace"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
"""types.SimpleNamespace: SimpleNamespace is not a mapping: len/iter/contains/getitem each raise TypeError"""
import types

plain = types.SimpleNamespace(spam="spamspamspam")
for op in (lambda: len(plain), lambda: iter(plain),
           lambda: "spam" in plain, lambda: plain["spam"]):
    _raised = False
    try:
        op()
    except TypeError:
        _raised = True
    assert _raised, "SimpleNamespace mapping op should raise TypeError"

print("simplenamespace_not_a_mapping OK")
"###);
    assert_output(&out, r###"simplenamespace_not_a_mapping OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/types/simplenamespace_recursive_repr.py`.
#[test]
fn test_gen_behavior_std_libs_types_simplenamespace_recursive_repr() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "simplenamespace_recursive_repr"
# subject = "types.SimpleNamespace"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
"""types.SimpleNamespace: a self-referential SimpleNamespace renders its cycle as 'namespace(...)' instead of recursing forever"""
import types

loop = types.SimpleNamespace(c="cookie")
loop.spam = loop
assert repr(loop) == "namespace(c='cookie', spam=namespace(...))"

print("simplenamespace_recursive_repr OK")
"###);
    assert_output(&out, r###"simplenamespace_recursive_repr OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/types/simplenamespace_repr.py`.
#[test]
fn test_gen_behavior_std_libs_types_simplenamespace_repr() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "simplenamespace_repr"
# subject = "types.SimpleNamespace"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
"""types.SimpleNamespace: repr renders 'namespace(...)' with attributes in insertion order, including underscored names"""
import types

assert repr(types.SimpleNamespace(x=1, y=2, w=3)) == "namespace(x=1, y=2, w=3)"
spammy = types.SimpleNamespace()
spammy.x = "spam"
spammy._y = 5
assert repr(spammy) == "namespace(x='spam', _y=5)"

print("simplenamespace_repr OK")
"###);
    assert_output(&out, r###"simplenamespace_repr OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/types/singleton_type_identities.py`.
#[test]
fn test_gen_behavior_std_libs_types_singleton_type_identities() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "singleton_type_identities"
# subject = "types.NoneType"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
"""types.NoneType: NoneType/NotImplementedType/EllipsisType are exactly the runtime types of None/NotImplemented/Ellipsis (isinstance holds and type(None) is types.NoneType)"""
import types

assert isinstance(None, types.NoneType)
assert isinstance(NotImplemented, types.NotImplementedType)
assert isinstance(Ellipsis, types.EllipsisType)
assert type(None) is types.NoneType
assert type(NotImplemented) is types.NotImplementedType
assert type(Ellipsis) is types.EllipsisType

print("singleton_type_identities OK")
"###);
    assert_output(&out, r###"singleton_type_identities OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/types/type_object_names.py`.
#[test]
fn test_gen_behavior_std_libs_types_type_object_names() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "type_object_names"
# subject = "types.NoneType"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
"""types.NoneType: the named type objects (FunctionType/MethodType/ModuleType/GeneratorType/CoroutineType/NoneType/MappingProxyType/EllipsisType/NotImplementedType) expose POSIX-stable __name__ values matching CPython"""
import types

# Type objects expose __name__ for introspection — POSIX-stable names that
# match CPython exactly.
expected = {
    "FunctionType": "function",
    "MethodType": "method",
    "ModuleType": "module",
    "GeneratorType": "generator",
    "CoroutineType": "coroutine",
    "NoneType": "NoneType",
    "MappingProxyType": "mappingproxy",
    "EllipsisType": "ellipsis",
    "NotImplementedType": "NotImplementedType",
}
for attr, name in expected.items():
    obj = getattr(types, attr)
    assert obj.__name__ == name, (attr, obj.__name__, name)

print("type_object_names OK")
"###);
    assert_output(&out, r###"type_object_names OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/types/union_args.py`.
#[test]
fn test_gen_behavior_std_libs_types_union_args() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "union_args"
# subject = "types.UnionType"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
"""types.UnionType: __args__ exposes the union member types in source order: (int | str).__args__ == (int, str)"""
import types  # noqa: F401

assert (int | str).__args__ == (int, str)
assert (str | int).__args__ == (str, int)

print("union_args OK")
"###);
    assert_output(&out, r###"union_args OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/types/union_construction_and_type.py`.
#[test]
fn test_gen_behavior_std_libs_types_union_construction_and_type() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "union_construction_and_type"
# subject = "types.UnionType"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
"""types.UnionType: int | str produces a UnionType instance: isinstance(u, types.UnionType) and type(u) is types.UnionType"""
import types

u = int | str
assert isinstance(u, types.UnionType)
assert type(u) is types.UnionType

print("union_construction_and_type OK")
"###);
    assert_output(&out, r###"union_construction_and_type OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/types/union_copy_deepcopy.py`.
#[test]
fn test_gen_behavior_std_libs_types_union_copy_deepcopy() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "union_copy_deepcopy"
# subject = "types.UnionType"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
"""types.UnionType: copy.copy and copy.deepcopy of a parameterized union (list[T] | int) preserve its __args__ and __parameters__"""
import copy
import types  # noqa: F401
import typing

T = typing.TypeVar("T")
orig = list[T] | int
for clone in (copy.copy(orig), copy.deepcopy(orig)):
    assert clone == orig
    assert clone.__args__ == orig.__args__
    assert clone.__parameters__ == orig.__parameters__

print("union_copy_deepcopy OK")
"###);
    assert_output(&out, r###"union_copy_deepcopy OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/types/union_hash_order_insensitive.py`.
#[test]
fn test_gen_behavior_std_libs_types_union_hash_order_insensitive() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "union_hash_order_insensitive"
# subject = "types.UnionType"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
"""types.UnionType: unions are order-insensitive for hashing/equality and equal to the typing.Union of the same members"""
import types  # noqa: F401
import typing

assert hash(int | str) == hash(str | int)
assert hash(int | str) == hash(typing.Union[int, str])
assert (int | str) == (str | int)
assert (int | str) == typing.Union[int, str]

print("union_hash_order_insensitive OK")
"###);
    assert_output(&out, r###"union_hash_order_insensitive OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/types/union_isinstance.py`.
#[test]
fn test_gen_behavior_std_libs_types_union_isinstance() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "union_isinstance"
# subject = "types.UnionType"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
"""types.UnionType: isinstance against a runtime union behaves like a tuple of types: isinstance(5, int|str) and isinstance('x', int|str) but not isinstance(1.5, int|str)"""
import types  # noqa: F401

assert isinstance(5, int | str)
assert isinstance("x", int | str)
assert not isinstance(1.5, int | str)

print("union_isinstance OK")
"###);
    assert_output(&out, r###"union_isinstance OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/types/union_parameter_substitution.py`.
#[test]
fn test_gen_behavior_std_libs_types_union_parameter_substitution() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "union_parameter_substitution"
# subject = "types.UnionType"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
"""types.UnionType: free type vars chain through subscription/substitution: (float | list[T])[int], list[T]|list[S] parameters and substitution"""
import types  # noqa: F401
import typing

T = typing.TypeVar("T")
S = typing.TypeVar("S")
assert (float | list[T])[int] == float | list[int]
assert (list[T] | list[S]).__parameters__ == (T, S)
assert (list[T] | list[S])[int, T] == list[int] | list[T]

print("union_parameter_substitution OK")
"###);
    assert_output(&out, r###"union_parameter_substitution OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/types/union_repr_flattens.py`.
#[test]
fn test_gen_behavior_std_libs_types_union_repr_flattens() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "union_repr_flattens"
# subject = "types.UnionType"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
"""types.UnionType: union repr flattens nesting and renders None for NoneType: int|str, int|str|list, int|(str|list), int|None, int|type(None)"""
import types  # noqa: F401

assert repr(int | str) == "int | str"
assert repr(int | str | list) == "int | str | list"
assert repr(int | (str | list)) == "int | str | list"
assert repr(int | None) == "int | None"
assert repr(int | type(None)) == "int | None"

print("union_repr_flattens OK")
"###);
    assert_output(&out, r###"union_repr_flattens OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/types/utility_functions_callable.py`.
#[test]
fn test_gen_behavior_std_libs_types_utility_functions_callable() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "types"
# dimension = "behavior"
# case = "utility_functions_callable"
# subject = "types.new_class"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_types.py"
# status = "filled"
# ///
"""types.new_class: the class-creation utilities new_class / prepare_class / resolve_bases are all callable"""
import types

assert callable(types.new_class)
assert callable(types.prepare_class)
assert callable(types.resolve_bases)

print("utility_functions_callable OK")
"###);
    assert_output(&out, r###"utility_functions_callable OK
"###);
}
