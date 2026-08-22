use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/dataclasses/mutable_list_default_raises.py`.
#[test]
fn test_gen_errors_std_libs_dataclasses_mutable_list_default_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "errors"
# case = "mutable_list_default_raises"
# subject = "dataclasses.dataclass"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dataclasses.py"
# status = "filled"
# ///
"""dataclasses.dataclass: mutable_list_default_raises (errors)."""
import dataclasses
from typing import List

_raised = False
try:
    dataclasses.dataclass(type('Bad', (), {'__annotations__': {'items': list}, 'items': []}))
except ValueError:
    _raised = True
assert _raised, "mutable_list_default_raises: expected ValueError"
print("mutable_list_default_raises OK")
"###);
    assert_output(&out, r###"mutable_list_default_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/dataclasses/non_callable_default_factory_raises.py`.
#[test]
fn test_gen_errors_std_libs_dataclasses_non_callable_default_factory_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "errors"
# case = "non_callable_default_factory_raises"
# subject = "dataclasses.field"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dataclasses.py"
# status = "filled"
# ///
"""dataclasses.field: non_callable_default_factory_raises (errors)."""
import dataclasses

_raised = False
try:
    dataclasses.dataclass(type('BadFactory', (), {'__annotations__': {'items': list}, 'items': dataclasses.field(default_factory=42)}))()
except TypeError:
    _raised = True
assert _raised, "non_callable_default_factory_raises: expected TypeError"
print("non_callable_default_factory_raises OK")
"###);
    assert_output(&out, r###"non_callable_default_factory_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/dataclasses/required_after_default_raises.py`.
#[test]
fn test_gen_errors_std_libs_dataclasses_required_after_default_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "dataclasses"
# dimension = "errors"
# case = "required_after_default_raises"
# subject = "dataclasses.dataclass"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_dataclasses.py"
# status = "filled"
# ///
"""dataclasses.dataclass: required_after_default_raises (errors)."""
import dataclasses

_raised = False
try:
    dataclasses.dataclass(type('WrongOrder', (), {'__annotations__': {'a': int, 'b': int}, 'a': 1}))
except TypeError:
    _raised = True
assert _raised, "required_after_default_raises: expected TypeError"
print("required_after_default_raises OK")
"###);
    assert_output(&out, r###"required_after_default_raises OK
"###);
}
