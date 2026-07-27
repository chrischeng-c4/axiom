use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/collections/chainmap_pop_from_back_map_raises.py`.
#[test]
fn test_gen_errors_std_libs_collections_chainmap_pop_from_back_map_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "errors"
# case = "chainmap_pop_from_back_map_raises"
# subject = "collections.ChainMap"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.ChainMap: chainmap_pop_from_back_map_raises (errors)."""
import collections

_raised = False
try:
    collections.ChainMap({'a': 1}, {'b': 2}).pop('b')
except KeyError:
    _raised = True
assert _raised, "chainmap_pop_from_back_map_raises: expected KeyError"
print("chainmap_pop_from_back_map_raises OK")
"###);
    assert_output(&out, r###"chainmap_pop_from_back_map_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/collections/counter_fromkeys_raises.py`.
#[test]
fn test_gen_errors_std_libs_collections_counter_fromkeys_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "errors"
# case = "counter_fromkeys_raises"
# subject = "collections.Counter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.Counter: counter_fromkeys_raises (errors)."""
import collections

_raised = False
try:
    collections.Counter.fromkeys('abc')
except NotImplementedError:
    _raised = True
assert _raised, "counter_fromkeys_raises: expected NotImplementedError"
print("counter_fromkeys_raises OK")
"###);
    assert_output(&out, r###"counter_fromkeys_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/collections/counter_non_iterable_init_raises.py`.
#[test]
fn test_gen_errors_std_libs_collections_counter_non_iterable_init_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "errors"
# case = "counter_non_iterable_init_raises"
# subject = "collections.Counter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.Counter: counter_non_iterable_init_raises (errors)."""
import collections

_raised = False
try:
    collections.Counter(123)
except TypeError:
    _raised = True
assert _raised, "counter_non_iterable_init_raises: expected TypeError"
print("counter_non_iterable_init_raises OK")
"###);
    assert_output(&out, r###"counter_non_iterable_init_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/collections/counter_unhashable_raises.py`.
#[test]
fn test_gen_errors_std_libs_collections_counter_unhashable_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "errors"
# case = "counter_unhashable_raises"
# subject = "collections.Counter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.Counter: counter_unhashable_raises (errors)."""
import collections

_raised = False
try:
    hash(collections.Counter(a=1))
except TypeError:
    _raised = True
assert _raised, "counter_unhashable_raises: expected TypeError"
print("counter_unhashable_raises OK")
"###);
    assert_output(&out, r###"counter_unhashable_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/collections/defaultdict_none_factory_missing_raises.py`.
#[test]
fn test_gen_errors_std_libs_collections_defaultdict_none_factory_missing_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "errors"
# case = "defaultdict_none_factory_missing_raises"
# subject = "collections.defaultdict"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.defaultdict: defaultdict_none_factory_missing_raises (errors)."""
import collections

_raised = False
try:
    collections.defaultdict(None)['x']
except KeyError:
    _raised = True
assert _raised, "defaultdict_none_factory_missing_raises: expected KeyError"
print("defaultdict_none_factory_missing_raises OK")
"###);
    assert_output(&out, r###"defaultdict_none_factory_missing_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/collections/deque_pop_empty_raises.py`.
#[test]
fn test_gen_errors_std_libs_collections_deque_pop_empty_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "errors"
# case = "deque_pop_empty_raises"
# subject = "collections.deque"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.deque: deque_pop_empty_raises (errors)."""
import collections

_raised = False
try:
    collections.deque().pop()
except IndexError:
    _raised = True
assert _raised, "deque_pop_empty_raises: expected IndexError"
print("deque_pop_empty_raises OK")
"###);
    assert_output(&out, r###"deque_pop_empty_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/collections/deque_popleft_empty_raises.py`.
#[test]
fn test_gen_errors_std_libs_collections_deque_popleft_empty_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "errors"
# case = "deque_popleft_empty_raises"
# subject = "collections.deque"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.deque: deque_popleft_empty_raises (errors)."""
import collections

_raised = False
try:
    collections.deque().popleft()
except IndexError:
    _raised = True
assert _raised, "deque_popleft_empty_raises: expected IndexError"
print("deque_popleft_empty_raises OK")
"###);
    assert_output(&out, r###"deque_popleft_empty_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/collections/namedtuple_bad_field_name_raises.py`.
#[test]
fn test_gen_errors_std_libs_collections_namedtuple_bad_field_name_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "errors"
# case = "namedtuple_bad_field_name_raises"
# subject = "collections.namedtuple"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.namedtuple: namedtuple_bad_field_name_raises (errors)."""
import collections

_raised = False
try:
    collections.namedtuple('Bad', 'x class')
except ValueError:
    _raised = True
assert _raised, "namedtuple_bad_field_name_raises: expected ValueError"
print("namedtuple_bad_field_name_raises OK")
"###);
    assert_output(&out, r###"namedtuple_bad_field_name_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/collections/namedtuple_field_readonly_raises.py`.
#[test]
fn test_gen_errors_std_libs_collections_namedtuple_field_readonly_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "errors"
# case = "namedtuple_field_readonly_raises"
# subject = "collections.namedtuple"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.namedtuple: namedtuple_field_readonly_raises (errors)."""
import collections

_raised = False
try:
    setattr(collections.namedtuple('Point', 'x y')(1, 2), 'x', 3)
except AttributeError:
    _raised = True
assert _raised, "namedtuple_field_readonly_raises: expected AttributeError"
print("namedtuple_field_readonly_raises OK")
"###);
    assert_output(&out, r###"namedtuple_field_readonly_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/collections/namedtuple_wrong_arity_raises.py`.
#[test]
fn test_gen_errors_std_libs_collections_namedtuple_wrong_arity_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "errors"
# case = "namedtuple_wrong_arity_raises"
# subject = "collections.namedtuple"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.namedtuple: namedtuple_wrong_arity_raises (errors)."""
import collections

_raised = False
try:
    collections.namedtuple('Point', 'x y')(1)
except TypeError:
    _raised = True
assert _raised, "namedtuple_wrong_arity_raises: expected TypeError"
print("namedtuple_wrong_arity_raises OK")
"###);
    assert_output(&out, r###"namedtuple_wrong_arity_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/collections/ordereddict_move_to_end_missing_raises.py`.
#[test]
fn test_gen_errors_std_libs_collections_ordereddict_move_to_end_missing_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections"
# dimension = "errors"
# case = "ordereddict_move_to_end_missing_raises"
# subject = "collections.OrderedDict"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""collections.OrderedDict: ordereddict_move_to_end_missing_raises (errors)."""
import collections

_raised = False
try:
    collections.OrderedDict([('a', 1)]).move_to_end('missing')
except KeyError:
    _raised = True
assert _raised, "ordereddict_move_to_end_missing_raises: expected KeyError"
print("ordereddict_move_to_end_missing_raises OK")
"###);
    assert_output(&out, r###"ordereddict_move_to_end_missing_raises OK
"###);
}
