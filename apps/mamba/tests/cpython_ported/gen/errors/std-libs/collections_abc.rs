use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/collections_abc/incomplete_subclass_raises.py`.
#[test]
fn test_gen_errors_std_libs_collections_abc_incomplete_subclass_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections_abc"
# dimension = "errors"
# case = "incomplete_subclass_raises"
# subject = "collections.abc.MutableMapping"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_collections_abc.py"
# status = "filled"
# ///
"""collections.abc.MutableMapping: incomplete_subclass_raises (errors)."""
import collections.abc

_raised = False
try:
    type('IncompleteMap', (collections.abc.MutableMapping,), {})()
except TypeError:
    _raised = True
assert _raised, "incomplete_subclass_raises: expected TypeError"
print("incomplete_subclass_raises OK")
"###);
    assert_output(&out, r###"incomplete_subclass_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/collections_abc/instantiate_iterator_raises.py`.
#[test]
fn test_gen_errors_std_libs_collections_abc_instantiate_iterator_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections_abc"
# dimension = "errors"
# case = "instantiate_iterator_raises"
# subject = "collections.abc.Iterator"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_collections_abc.py"
# status = "filled"
# ///
"""collections.abc.Iterator: instantiate_iterator_raises (errors)."""
import collections.abc

_raised = False
try:
    collections.abc.Iterator()
except TypeError:
    _raised = True
assert _raised, "instantiate_iterator_raises: expected TypeError"
print("instantiate_iterator_raises OK")
"###);
    assert_output(&out, r###"instantiate_iterator_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/collections_abc/instantiate_mapping_raises.py`.
#[test]
fn test_gen_errors_std_libs_collections_abc_instantiate_mapping_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections_abc"
# dimension = "errors"
# case = "instantiate_mapping_raises"
# subject = "collections.abc.Mapping"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_collections_abc.py"
# status = "filled"
# ///
"""collections.abc.Mapping: instantiate_mapping_raises (errors)."""
import collections.abc

_raised = False
try:
    collections.abc.Mapping()
except TypeError:
    _raised = True
assert _raised, "instantiate_mapping_raises: expected TypeError"
print("instantiate_mapping_raises OK")
"###);
    assert_output(&out, r###"instantiate_mapping_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/collections_abc/instantiate_sequence_raises.py`.
#[test]
fn test_gen_errors_std_libs_collections_abc_instantiate_sequence_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "collections_abc"
# dimension = "errors"
# case = "instantiate_sequence_raises"
# subject = "collections.abc.Sequence"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_collections_abc.py"
# status = "filled"
# ///
"""collections.abc.Sequence: instantiate_sequence_raises (errors)."""
import collections.abc

_raised = False
try:
    collections.abc.Sequence()
except TypeError:
    _raised = True
assert _raised, "instantiate_sequence_raises: expected TypeError"
print("instantiate_sequence_raises OK")
"###);
    assert_output(&out, r###"instantiate_sequence_raises OK
"###);
}
