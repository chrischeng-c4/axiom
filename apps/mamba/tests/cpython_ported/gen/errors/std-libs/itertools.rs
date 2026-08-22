use super::super::super::super::harness::*;

/// Ported from `tests/cpython/errors/std-libs/itertools/chain_propagates_source_error.py`.
#[test]
fn test_gen_errors_std_libs_itertools_chain_propagates_source_error() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "errors"
# case = "chain_propagates_source_error"
# subject = "itertools.chain"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.chain: chain_propagates_source_error (errors)."""
import itertools

def _boom():
    yield 1
    raise ValueError('boom')

_raised = False
try:
    list(itertools.chain(_boom(), [9, 9]))
except ValueError:
    _raised = True
assert _raised, "chain_propagates_source_error: expected ValueError"
print("chain_propagates_source_error OK")
"###);
    assert_output(&out, r###"chain_propagates_source_error OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/itertools/combinations_negative_r_raises.py`.
#[test]
fn test_gen_errors_std_libs_itertools_combinations_negative_r_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "errors"
# case = "combinations_negative_r_raises"
# subject = "itertools.combinations"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.combinations: combinations_negative_r_raises (errors)."""
import itertools

_raised = False
try:
    list(itertools.combinations([1, 2, 3], -1))
except ValueError:
    _raised = True
assert _raised, "combinations_negative_r_raises: expected ValueError"
print("combinations_negative_r_raises OK")
"###);
    assert_output(&out, r###"combinations_negative_r_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/itertools/cycle_propagates_source_error.py`.
#[test]
fn test_gen_errors_std_libs_itertools_cycle_propagates_source_error() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "errors"
# case = "cycle_propagates_source_error"
# subject = "itertools.cycle"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.cycle: cycle_propagates_source_error (errors)."""
import itertools

def _boom():
    yield 1
    raise ValueError('boom')

_raised = False
try:
    list(itertools.cycle(_boom()))
except ValueError:
    _raised = True
assert _raised, "cycle_propagates_source_error: expected ValueError"
print("cycle_propagates_source_error OK")
"###);
    assert_output(&out, r###"cycle_propagates_source_error OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/itertools/islice_negative_step_raises.py`.
#[test]
fn test_gen_errors_std_libs_itertools_islice_negative_step_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "errors"
# case = "islice_negative_step_raises"
# subject = "itertools.islice"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.islice: islice_negative_step_raises (errors)."""
import itertools

_raised = False
try:
    list(itertools.islice([1, 2, 3], 0, None, -1))
except ValueError:
    _raised = True
assert _raised, "islice_negative_step_raises: expected ValueError"
print("islice_negative_step_raises OK")
"###);
    assert_output(&out, r###"islice_negative_step_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/itertools/islice_negative_stop_raises.py`.
#[test]
fn test_gen_errors_std_libs_itertools_islice_negative_stop_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "errors"
# case = "islice_negative_stop_raises"
# subject = "itertools.islice"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.islice: islice_negative_stop_raises (errors)."""
import itertools

_raised = False
try:
    list(itertools.islice([1, 2, 3], -1))
except ValueError:
    _raised = True
assert _raised, "islice_negative_stop_raises: expected ValueError"
print("islice_negative_stop_raises OK")
"###);
    assert_output(&out, r###"islice_negative_stop_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/itertools/permutations_negative_r_raises.py`.
#[test]
fn test_gen_errors_std_libs_itertools_permutations_negative_r_raises() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "errors"
# case = "permutations_negative_r_raises"
# subject = "itertools.permutations"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.permutations: permutations_negative_r_raises (errors)."""
import itertools

_raised = False
try:
    list(itertools.permutations([1, 2, 3], -1))
except ValueError:
    _raised = True
assert _raised, "permutations_negative_r_raises: expected ValueError"
print("permutations_negative_r_raises OK")
"###);
    assert_output(&out, r###"permutations_negative_r_raises OK
"###);
}

/// Ported from `tests/cpython/errors/std-libs/itertools/zip_longest_propagates_source_error.py`.
#[test]
fn test_gen_errors_std_libs_itertools_zip_longest_propagates_source_error() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "errors"
# case = "zip_longest_propagates_source_error"
# subject = "itertools.zip_longest"
# kind = "mechanical"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.zip_longest: zip_longest_propagates_source_error (errors)."""
import itertools

class _R:
    def __init__(self, n):
        self.n = n
    def __iter__(self):
        return self
    def __next__(self):
        if self.n > 0:
            self.n -= 1
            return 1
        raise RuntimeError('boom')

_raised = False
try:
    list(itertools.zip_longest(_R(3), _R(9), fillvalue=0))
except RuntimeError:
    _raised = True
assert _raised, "zip_longest_propagates_source_error: expected RuntimeError"
print("zip_longest_propagates_source_error OK")
"###);
    assert_output(&out, r###"zip_longest_propagates_source_error OK
"###);
}
