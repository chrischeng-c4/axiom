use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/operator/add_mixed_types_typeerror.py`.
#[test]
fn test_gen_errors_std_libs_operator_add_mixed_types_typeerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "errors"
# case = "add_mixed_types_typeerror"
# subject = "operator.add"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.add: add_mixed_types_typeerror (errors)."""
import operator

_raised = False
try:
    operator.add(1, "a")
except TypeError:
    _raised = True
assert _raised, "add_mixed_types_typeerror: expected TypeError"
print("add_mixed_types_typeerror OK")
"###);
    assert_output(&out, r###"add_mixed_types_typeerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/operator/attrgetter_empty_path_segment_attributeerror.py`.
#[test]
fn test_gen_errors_std_libs_operator_attrgetter_empty_path_segment_attributeerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "errors"
# case = "attrgetter_empty_path_segment_attributeerror"
# subject = "operator.attrgetter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.attrgetter: empty dotted-path segments ('child.' and '.child') are not valid attribute names so attrgetter raises AttributeError"""
import operator


class Node:
    pass


root = Node()
root.child = Node()

for bad in ("child.", ".child"):
    _raised = False
    try:
        operator.attrgetter(bad)(root)
    except AttributeError:
        _raised = True
    assert _raised, f"expected AttributeError for {bad!r}"
print("attrgetter_empty_path_segment_attributeerror OK")
"###);
    assert_output(&out, r###"attrgetter_empty_path_segment_attributeerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/operator/attrgetter_missing_attr_attributeerror.py`.
#[test]
fn test_gen_errors_std_libs_operator_attrgetter_missing_attr_attributeerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "errors"
# case = "attrgetter_missing_attr_attributeerror"
# subject = "operator.attrgetter"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.attrgetter: attrgetter_missing_attr_attributeerror (errors)."""
import operator

_raised = False
try:
    operator.attrgetter("foo")(object())
except AttributeError:
    _raised = True
assert _raised, "attrgetter_missing_attr_attributeerror: expected AttributeError"
print("attrgetter_missing_attr_attributeerror OK")
"###);
    assert_output(&out, r###"attrgetter_missing_attr_attributeerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/operator/iconcat_non_sequence_typeerror.py`.
#[test]
fn test_gen_errors_std_libs_operator_iconcat_non_sequence_typeerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "errors"
# case = "iconcat_non_sequence_typeerror"
# subject = "operator.iconcat"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.iconcat: iconcat_non_sequence_typeerror (errors)."""
import operator

_raised = False
try:
    operator.iconcat(1, 0.5)
except TypeError:
    _raised = True
assert _raised, "iconcat_non_sequence_typeerror: expected TypeError"
print("iconcat_non_sequence_typeerror OK")
"###);
    assert_output(&out, r###"iconcat_non_sequence_typeerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/operator/itemgetter_missing_key_keyerror.py`.
#[test]
fn test_gen_errors_std_libs_operator_itemgetter_missing_key_keyerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "errors"
# case = "itemgetter_missing_key_keyerror"
# subject = "operator.itemgetter"
# kind = "mechanical"
# xfail = "operator.itemgetter(i)(row) returns 0 and swallows the raise under mamba (repo-memory project_mamba_operator_itemgetter_returns_zero)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.itemgetter: itemgetter_missing_key_keyerror (errors)."""
import operator

_raised = False
try:
    operator.itemgetter("a")({})
except KeyError:
    _raised = True
assert _raised, "itemgetter_missing_key_keyerror: expected KeyError"
print("itemgetter_missing_key_keyerror OK")
"###);
    assert_output(&out, r###"itemgetter_missing_key_keyerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/operator/itemgetter_out_of_range_indexerror.py`.
#[test]
fn test_gen_errors_std_libs_operator_itemgetter_out_of_range_indexerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "errors"
# case = "itemgetter_out_of_range_indexerror"
# subject = "operator.itemgetter"
# kind = "mechanical"
# xfail = "operator.itemgetter(i)(row) returns 0 and swallows the raise under mamba (repo-memory project_mamba_operator_itemgetter_returns_zero)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.itemgetter: itemgetter_out_of_range_indexerror (errors)."""
import operator

_raised = False
try:
    operator.itemgetter(5)([1, 2, 3])
except IndexError:
    _raised = True
assert _raised, "itemgetter_out_of_range_indexerror: expected IndexError"
print("itemgetter_out_of_range_indexerror OK")
"###);
    assert_output(&out, r###"itemgetter_out_of_range_indexerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/operator/itemgetter_str_key_on_str_typeerror.py`.
#[test]
fn test_gen_errors_std_libs_operator_itemgetter_str_key_on_str_typeerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "errors"
# case = "itemgetter_str_key_on_str_typeerror"
# subject = "operator.itemgetter"
# kind = "mechanical"
# xfail = "operator.itemgetter(i)(row) returns 0 and swallows the raise under mamba (repo-memory project_mamba_operator_itemgetter_returns_zero)"
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.itemgetter: itemgetter_str_key_on_str_typeerror (errors)."""
import operator

_raised = False
try:
    operator.itemgetter("name")("ABCDE")
except TypeError:
    _raised = True
assert _raised, "itemgetter_str_key_on_str_typeerror: expected TypeError"
print("itemgetter_str_key_on_str_typeerror OK")
"###);
    assert_output(&out, r###"itemgetter_str_key_on_str_typeerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/operator/length_hint_negative_valueerror.py`.
#[test]
fn test_gen_errors_std_libs_operator_length_hint_negative_valueerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "errors"
# case = "length_hint_negative_valueerror"
# subject = "operator.length_hint"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.length_hint: a __length_hint__ returning a negative count makes operator.length_hint raise ValueError"""
import operator


class Hinted:
    def __length_hint__(self):
        return -2


_raised = False
try:
    operator.length_hint(Hinted())
except ValueError:
    _raised = True
assert _raised, "expected ValueError for negative __length_hint__"
print("length_hint_negative_valueerror OK")
"###);
    assert_output(&out, r###"length_hint_negative_valueerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/operator/matmul_on_ints_typeerror.py`.
#[test]
fn test_gen_errors_std_libs_operator_matmul_on_ints_typeerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "errors"
# case = "matmul_on_ints_typeerror"
# subject = "operator.matmul"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.matmul: matmul_on_ints_typeerror (errors)."""
import operator

_raised = False
try:
    operator.matmul(42, 42)
except TypeError:
    _raised = True
assert _raised, "matmul_on_ints_typeerror: expected TypeError"
print("matmul_on_ints_typeerror OK")
"###);
    assert_output(&out, r###"matmul_on_ints_typeerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/operator/methodcaller_missing_method_attributeerror.py`.
#[test]
fn test_gen_errors_std_libs_operator_methodcaller_missing_method_attributeerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "errors"
# case = "methodcaller_missing_method_attributeerror"
# subject = "operator.methodcaller"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.methodcaller: methodcaller_missing_method_attributeerror (errors)."""
import operator

_raised = False
try:
    operator.methodcaller("no_such_method")(object())
except AttributeError:
    _raised = True
assert _raised, "methodcaller_missing_method_attributeerror: expected AttributeError"
print("methodcaller_missing_method_attributeerror OK")
"###);
    assert_output(&out, r###"methodcaller_missing_method_attributeerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/operator/methodcaller_propagates_method_argerror.py`.
#[test]
fn test_gen_errors_std_libs_operator_methodcaller_propagates_method_argerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "errors"
# case = "methodcaller_propagates_method_argerror"
# subject = "operator.methodcaller"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.methodcaller: the bound method's own argument error propagates unchanged: methodcaller('add_first_two')(t) with too few args raises IndexError from the method body"""
import operator


class Target:
    def add_first_two(self, *args, **kwds):
        return args[0] + args[1]


_raised = False
try:
    operator.methodcaller("add_first_two")(Target())
except IndexError:
    _raised = True
assert _raised, "expected IndexError from the method body"
print("methodcaller_propagates_method_argerror OK")
"###);
    assert_output(&out, r###"methodcaller_propagates_method_argerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/operator/setitem_out_of_range_indexerror.py`.
#[test]
fn test_gen_errors_std_libs_operator_setitem_out_of_range_indexerror() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "errors"
# case = "setitem_out_of_range_indexerror"
# subject = "operator.setitem"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.setitem: setitem_out_of_range_indexerror (errors)."""
import operator

_raised = False
try:
    operator.setitem([0, 1, 2], 4, 99)
except IndexError:
    _raised = True
assert _raised, "setitem_out_of_range_indexerror: expected IndexError"
print("setitem_out_of_range_indexerror OK")
"###);
    assert_output(&out, r###"setitem_out_of_range_indexerror OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/operator/truediv_by_zero_zerodivision.py`.
#[test]
fn test_gen_errors_std_libs_operator_truediv_by_zero_zerodivision() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "operator"
# dimension = "errors"
# case = "truediv_by_zero_zerodivision"
# subject = "operator.truediv"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""operator.truediv: truediv_by_zero_zerodivision (errors)."""
import operator

_raised = False
try:
    operator.truediv(1, 0)
except ZeroDivisionError:
    _raised = True
assert _raised, "truediv_by_zero_zerodivision: expected ZeroDivisionError"
print("truediv_by_zero_zerodivision OK")
"###);
    assert_output(&out, r###"truediv_by_zero_zerodivision OK
"###);
}
