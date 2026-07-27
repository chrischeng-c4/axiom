use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/itertools/accumulate_initial_seed.py`.
#[test]
fn test_gen_behavior_std_libs_itertools_accumulate_initial_seed() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "accumulate_initial_seed"
# subject = "itertools.accumulate"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.accumulate: accumulate(it, initial=v) prepends the seed before folding the rest"""
import itertools

acc = list(itertools.accumulate([1, 2, 3, 4], initial=0))
assert acc == [0, 1, 3, 6, 10], f"accumulate initial = {acc!r}"

print("accumulate_initial_seed OK")
"###);
    assert_output(&out, r###"accumulate_initial_seed OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/itertools/accumulate_non_numeric_op.py`.
#[test]
fn test_gen_behavior_std_libs_itertools_accumulate_non_numeric_op() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "accumulate_non_numeric_op"
# subject = "itertools.accumulate"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.accumulate: accumulate works with a non-numeric binary op: operator.is_ over [None,None,None] gives [None, True, False]"""
import itertools

import operator
acc_is = list(itertools.accumulate([None, None, None], operator.is_))
assert acc_is == [None, True, False], f"accumulate is_ = {acc_is!r}"

print("accumulate_non_numeric_op OK")
"###);
    assert_output(&out, r###"accumulate_non_numeric_op OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/itertools/accumulate_running_fold.py`.
#[test]
fn test_gen_behavior_std_libs_itertools_accumulate_running_fold() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "accumulate_running_fold"
# subject = "itertools.accumulate"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.accumulate: accumulate yields running results: default add, a custom binary op (mul), and a running-max lambda"""
import itertools

import operator
assert list(itertools.accumulate([1, 2, 3, 4])) == [1, 3, 6, 10], "accumulate sum"
assert list(itertools.accumulate([1, 2, 3, 4], operator.mul)) == [1, 2, 6, 24], "accumulate mul"
assert list(itertools.accumulate([3, 1, 4, 1, 5], lambda a, b: a if a > b else b)) == [3, 3, 4, 4, 5], "running max"
assert list(itertools.accumulate([10])) == [10], "single element"

print("accumulate_running_fold OK")
"###);
    assert_output(&out, r###"accumulate_running_fold OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/itertools/chain_concatenates_iterables.py`.
#[test]
fn test_gen_behavior_std_libs_itertools_chain_concatenates_iterables() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "chain_concatenates_iterables"
# subject = "itertools.chain"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.chain: chain flattens its positional iterables in order, including mixed list/tuple/str and empty inputs"""
import itertools

assert list(itertools.chain([1, 2], [3, 4], [5])) == [1, 2, 3, 4, 5], "chain three"
assert list(itertools.chain("abc", "def")) == ["a", "b", "c", "d", "e", "f"], "chain strings"
assert list(itertools.chain([1, 2], (3, 4), [5])) == [1, 2, 3, 4, 5], "chain mixed types"
assert list(itertools.chain()) == [], "chain no args"
assert list(itertools.chain([], [])) == [], "chain empty"
assert list(itertools.chain([1, 2, 3], [])) == [1, 2, 3], "chain trailing empty"
assert list(itertools.chain("")) == [], "chain empty string"

print("chain_concatenates_iterables OK")
"###);
    assert_output(&out, r###"chain_concatenates_iterables OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/itertools/chain_from_iterable_flattens.py`.
#[test]
fn test_gen_behavior_std_libs_itertools_chain_from_iterable_flattens() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "chain_from_iterable_flattens"
# subject = "itertools.chain.from_iterable"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.chain.from_iterable: chain.from_iterable interleaves nested iterables (incl. per-char strings) and ends cleanly"""
import itertools

assert list(itertools.chain.from_iterable([[1, 2], [3, 4]])) == [1, 2, 3, 4], "from_iterable lists"
assert "".join(itertools.chain.from_iterable(["ABC", "DEF"])) == "ABCDEF", "from_iterable join"

print("chain_from_iterable_flattens OK")
"###);
    assert_output(&out, r###"chain_from_iterable_flattens OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/itertools/chain_from_iterable_infinite_compose.py`.
#[test]
fn test_gen_behavior_std_libs_itertools_chain_from_iterable_infinite_compose() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "chain_from_iterable_infinite_compose"
# subject = "itertools.chain.from_iterable"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.chain.from_iterable: chain.from_iterable over an infinite repeat composes lazily; islice bounds it without diverging"""
import itertools

endless = itertools.chain.from_iterable(itertools.repeat(range(5)))
assert list(itertools.islice(endless, 7)) == [0, 1, 2, 3, 4, 0, 1], "endless prefix"

empties = itertools.chain.from_iterable(() for _ in range(10000))
raised = False
try:
    next(empties)
except StopIteration:
    raised = True
assert raised, "long run of empty sub-iterables terminates cleanly"

print("chain_from_iterable_infinite_compose OK")
"###);
    assert_output(&out, r###"chain_from_iterable_infinite_compose OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/itertools/combinations_r_subsets.py`.
#[test]
fn test_gen_behavior_std_libs_itertools_combinations_r_subsets() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "combinations_r_subsets"
# subject = "itertools.combinations"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.combinations: combinations(it, r) yields sorted r-subsets without repetition; r==0 is [()], r==n is [tuple(it)]"""
import itertools

combs = list(itertools.combinations([1, 2, 3], 2))
assert combs == [(1, 2), (1, 3), (2, 3)], f"combinations = {combs!r}"
assert list(itertools.combinations([1, 2, 3], 0)) == [()], "r==0 is [()]"
assert list(itertools.combinations([1, 2, 3], 3)) == [(1, 2, 3)], "r==n is the whole tuple"
assert list(itertools.combinations("abc", 2)) == [("a", "b"), ("a", "c"), ("b", "c")], "string combinations"

print("combinations_r_subsets OK")
"###);
    assert_output(&out, r###"combinations_r_subsets OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/itertools/combinations_with_replacement_multisets.py`.
#[test]
fn test_gen_behavior_std_libs_itertools_combinations_with_replacement_multisets() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "combinations_with_replacement_multisets"
# subject = "itertools.combinations_with_replacement"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.combinations_with_replacement: combinations_with_replacement allows repeats; r may exceed n; counts follow C(n+r-1, r); r==0 is [()]"""
import itertools

cwr = list(itertools.combinations_with_replacement([1, 2], 2))
assert cwr == [(1, 1), (1, 2), (2, 2)], f"cwr = {cwr!r}"
assert list(itertools.combinations_with_replacement([1], 3)) == [(1, 1, 1)], "r may exceed n"
assert list(itertools.combinations_with_replacement([1, 2], 0)) == [()], "r==0 is [()]"
assert list(itertools.combinations_with_replacement([], 2)) == [], "empty input r>=1"
assert len(list(itertools.combinations_with_replacement([1, 2, 3, 4], 3))) == 20, "C(n+r-1, r) count"

print("combinations_with_replacement_multisets OK")
"###);
    assert_output(&out, r###"combinations_with_replacement_multisets OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/itertools/compress_selects_by_truthy_mask.py`.
#[test]
fn test_gen_behavior_std_libs_itertools_compress_selects_by_truthy_mask() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "compress_selects_by_truthy_mask"
# subject = "itertools.compress"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.compress: compress yields data items whose selector is truthy, stopping at the shorter of data/selectors"""
import itertools

assert list(itertools.compress("ABCD", [1, 0, 1, 0])) == ["A", "C"], "compress basic"
assert list(itertools.compress("AB", [1, 1, 1, 1, 1])) == ["A", "B"], "compress stops at data"
assert list(itertools.compress([1, 2, 3], [True, False, True])) == [1, 3], "compress bool mask"
assert list(itertools.compress([1, 2, 3], ["x", "", "y"])) == [1, 3], "compress truthy strings"
assert list(itertools.compress([1, 2, 3], [0, 0, 0])) == [], "compress all false"

print("compress_selects_by_truthy_mask OK")
"###);
    assert_output(&out, r###"compress_selects_by_truthy_mask OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/itertools/count_lazy_infinite.py`.
#[test]
fn test_gen_behavior_std_libs_itertools_count_lazy_infinite() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "count_lazy_infinite"
# subject = "itertools.count"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.count: count(start[, step]) is a lazy infinite counter; first N pulls give start, start+step, ..."""
import itertools

c = itertools.count(0)
first5 = [next(c) for _ in range(5)]
assert first5 == [0, 1, 2, 3, 4], f"count 5 = {first5!r}"

stepped = itertools.count(10, 2)
assert next(stepped) == 10, "count start"
assert next(stepped) == 12, "count step"

print("count_lazy_infinite OK")
"###);
    assert_output(&out, r###"count_lazy_infinite OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/itertools/cycle_repeats_sequence.py`.
#[test]
fn test_gen_behavior_std_libs_itertools_cycle_repeats_sequence() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "cycle_repeats_sequence"
# subject = "itertools.cycle"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.cycle: cycle(seq) repeats the source endlessly; the first 2*len pulls reproduce the sequence twice"""
import itertools

cy = itertools.cycle([1, 2, 3])
got = [next(cy) for _ in range(6)]
assert got == [1, 2, 3, 1, 2, 3], f"cycle = {got!r}"

print("cycle_repeats_sequence OK")
"###);
    assert_output(&out, r###"cycle_repeats_sequence OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/itertools/filterfalse_keeps_falsy.py`.
#[test]
fn test_gen_behavior_std_libs_itertools_filterfalse_keeps_falsy() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "filterfalse_keeps_falsy"
# subject = "itertools.filterfalse"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.filterfalse: filterfalse keeps items where the predicate is falsy (complement of filter)"""
import itertools

assert list(itertools.filterfalse(lambda x: x % 2, range(6))) == [0, 2, 4], "filterfalse evens"
assert list(itertools.filterfalse(lambda x: x % 2 == 0, [1, 2, 3, 4, 5])) == [1, 3, 5], "filterfalse odds"
assert list(itertools.filterfalse(lambda x: x, [0, 1, 0, 2, 0])) == [0, 0, 0], "filterfalse falsy"
assert list(itertools.filterfalse(lambda x: x, [])) == [], "filterfalse empty"

print("filterfalse_keeps_falsy OK")
"###);
    assert_output(&out, r###"filterfalse_keeps_falsy OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/itertools/groupby_consecutive_runs.py`.
#[test]
fn test_gen_behavior_std_libs_itertools_groupby_consecutive_runs() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "groupby_consecutive_runs"
# subject = "itertools.groupby"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.groupby: groupby groups consecutive equal elements (no global sort), with and without a key function"""
import itertools

data = [1, 1, 2, 3, 3, 1]
groups = [(k, list(g)) for k, g in itertools.groupby(data)]
assert groups == [(1, [1, 1]), (2, [2]), (3, [3, 3]), (1, [1])], f"groupby = {groups!r}"

words = ["ant", "bear", "cat", "dog", "eagle"]
by_len = [(k, list(g)) for k, g in itertools.groupby(words, key=len)]
assert by_len[0] == (3, ["ant"]), f"by_len[0] = {by_len[0]!r}"

print("groupby_consecutive_runs OK")
"###);
    assert_output(&out, r###"groupby_consecutive_runs OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/itertools/groupby_empty_input.py`.
#[test]
fn test_gen_behavior_std_libs_itertools_groupby_empty_input() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "groupby_empty_input"
# subject = "itertools.groupby"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.groupby: groupby over empty input yields no groups, with and without a key function"""
import itertools

assert list(itertools.groupby([])) == [], "empty"
assert list(itertools.groupby([], key=id)) == [], "empty keyed"

print("groupby_empty_input OK")
"###);
    assert_output(&out, r###"groupby_empty_input OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/itertools/groupby_nested_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_itertools_groupby_nested_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "groupby_nested_roundtrip"
# subject = "itertools.groupby"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.groupby: nested groupby (by first field, then second) re-assembles exactly the original rows"""
import itertools

rows = [(0, 10), (0, 10), (0, 11), (1, 11), (1, 12)]
rebuilt = []
for k, g in itertools.groupby(rows, key=lambda r: r[0]):
    for ik, ig in itertools.groupby(g, key=lambda r: r[1]):
        for elem in ig:
            assert k == elem[0] and ik == elem[1], f"nested key {elem!r}"
            rebuilt.append(elem)
assert rebuilt == rows, f"rebuilt = {rebuilt!r}"

print("groupby_nested_roundtrip OK")
"###);
    assert_output(&out, r###"groupby_nested_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/itertools/groupby_shared_iterator_invalidation.py`.
#[test]
fn test_gen_behavior_std_libs_itertools_groupby_shared_iterator_invalidation() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "groupby_shared_iterator_invalidation"
# subject = "itertools.groupby"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.groupby: groupby groups share one underlying iterator; advancing the outer iterator empties earlier un-consumed groups"""
import itertools

data = list(zip("AABBBAAAA", range(9)))
it = itertools.groupby(data, key=lambda r: r[0])
_, g1 = next(it)
_, g2 = next(it)
_, g3 = next(it)
assert list(g1) == [], "stale g1"
assert list(g2) == [], "stale g2"
assert next(g3) == ("A", 5), "live g3 first"
list(it)  # exhaust outer -> g3 also goes stale
assert list(g3) == [], "g3 stale after outer exhausted"

print("groupby_shared_iterator_invalidation OK")
"###);
    assert_output(&out, r###"groupby_shared_iterator_invalidation OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/itertools/groupby_sorted_keys_and_counts.py`.
#[test]
fn test_gen_behavior_std_libs_itertools_groupby_sorted_keys_and_counts() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "groupby_sorted_keys_and_counts"
# subject = "itertools.groupby"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.groupby: sort-then-group idiom over a string gives distinct keys and per-group element counts"""
import itertools

s = "abracadabra"
keys = [k for k, g in itertools.groupby(sorted(s))]
assert keys == ["a", "b", "c", "d", "r"], f"keys = {keys!r}"
counts = [(len(list(g)), k) for k, g in itertools.groupby(sorted(s))]
assert counts == [(5, "a"), (2, "b"), (1, "c"), (1, "d"), (2, "r")], f"counts = {counts!r}"

print("groupby_sorted_keys_and_counts OK")
"###);
    assert_output(&out, r###"groupby_sorted_keys_and_counts OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/itertools/islice_bounds_infinite_source.py`.
#[test]
fn test_gen_behavior_std_libs_itertools_islice_bounds_infinite_source() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "islice_bounds_infinite_source"
# subject = "itertools.islice"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.islice: islice bounds an infinite count(): islice(count(), 5) yields exactly the first five values"""
import itertools

limited = list(itertools.islice(itertools.count(), 5))
assert limited == [0, 1, 2, 3, 4], f"islice count = {limited!r}"

stepped = list(itertools.islice(itertools.count(100, 5), 4))
assert stepped == [100, 105, 110, 115], f"islice count step = {stepped!r}"

print("islice_bounds_infinite_source OK")
"###);
    assert_output(&out, r###"islice_bounds_infinite_source OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/itertools/islice_stop_start_step_forms.py`.
#[test]
fn test_gen_behavior_std_libs_itertools_islice_stop_start_step_forms() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "islice_stop_start_step_forms"
# subject = "itertools.islice"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.islice: islice supports stop-only, start/stop, and start/stop/step; over-long stop is clamped to the input"""
import itertools

assert list(itertools.islice(range(10), 3)) == [0, 1, 2], "stop only"
assert list(itertools.islice(range(20), 2, 8)) == [2, 3, 4, 5, 6, 7], "start/stop"
assert list(itertools.islice(range(10), 2, 7, 2)) == [2, 4, 6], "start/stop/step"
assert list(itertools.islice(range(5), 100)) == [0, 1, 2, 3, 4], "stop past end clamps"
assert list(itertools.islice("abcdef", 2, 5)) == ["c", "d", "e"], "string slice"

print("islice_stop_start_step_forms OK")
"###);
    assert_output(&out, r###"islice_stop_start_step_forms OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/itertools/lazy_islice_partial_consumption.py`.
#[test]
fn test_gen_behavior_std_libs_itertools_lazy_islice_partial_consumption() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "lazy_islice_partial_consumption"
# subject = "itertools.islice"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.islice: islice over a live iterator consumes exactly the requested window and leaves the remainder on the same source"""
import itertools

it = iter(range(10))
assert list(itertools.islice(it, 3)) == [0, 1, 2], "islice prefix"
assert list(it) == [3, 4, 5, 6, 7, 8, 9], "remainder after islice"

it = iter(range(10))
assert list(itertools.islice(it, 3, 3)) == [], "empty slice"
assert list(it) == [3, 4, 5, 6, 7, 8, 9], "untouched remainder"

c = itertools.count()
assert list(itertools.islice(c, 1, 3, 50)) == [1], "single hit"
assert next(c) == 3, "count position after islice"

print("lazy_islice_partial_consumption OK")
"###);
    assert_output(&out, r###"lazy_islice_partial_consumption OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/itertools/pairwise_consecutive_pairs.py`.
#[test]
fn test_gen_behavior_std_libs_itertools_pairwise_consecutive_pairs() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "pairwise_consecutive_pairs"
# subject = "itertools.pairwise"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.pairwise: pairwise yields consecutive (prev, curr) 2-tuples; fewer than two inputs yields nothing"""
import itertools

assert list(itertools.pairwise([1, 2, 3, 4])) == [(1, 2), (2, 3), (3, 4)], "pairwise list"
assert list(itertools.pairwise("ABCDE")) == [("A", "B"), ("B", "C"), ("C", "D"), ("D", "E")], "pairwise string"
assert list(itertools.pairwise([1])) == [], "pairwise single"
assert list(itertools.pairwise([])) == [], "pairwise empty"

print("pairwise_consecutive_pairs OK")
"###);
    assert_output(&out, r###"pairwise_consecutive_pairs OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/itertools/permutations_r_orderings.py`.
#[test]
fn test_gen_behavior_std_libs_itertools_permutations_r_orderings() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "permutations_r_orderings"
# subject = "itertools.permutations"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.permutations: permutations(it, r) yields ordered r-length arrangements; default r==n; r==1 is singletons; len is n!/(n-r)!"""
import itertools

perms = list(itertools.permutations([1, 2, 3], 2))
assert len(perms) == 6, f"permutations len = {len(perms)!r}"
assert (1, 2) in perms, "perm (1,2)"
assert list(itertools.permutations([1, 2])) == [(1, 2), (2, 1)], "default r==n"
assert list(itertools.permutations([1, 2, 3], 1)) == [(1,), (2,), (3,)], "r==1 singletons"

print("permutations_r_orderings OK")
"###);
    assert_output(&out, r###"permutations_r_orderings OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/itertools/predicate_truthiness_not_strict_bool.py`.
#[test]
fn test_gen_behavior_std_libs_itertools_predicate_truthiness_not_strict_bool() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "predicate_truthiness_not_strict_bool"
# subject = "itertools.takewhile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.takewhile: takewhile/dropwhile/filterfalse use Python truthiness (int/str/object), not strict bool, for the predicate result"""
import itertools

def isodd(x):
    return x % 2

def echo(x):
    return x

# int-returning predicate uses Python truthiness, not strict bool
assert list(itertools.filterfalse(isodd, [1, 2, 3, 4, 5])) == [2, 4], "filterfalse int pred"
assert list(itertools.takewhile(isodd, [1, 3, 5, 4, 7])) == [1, 3, 5], "takewhile int pred"
assert list(itertools.dropwhile(isodd, [1, 3, 5, 4, 7])) == [4, 7], "dropwhile int pred"

# str-returning predicate: truthy iff non-empty
assert list(itertools.takewhile(echo, ["a", "b", "", "c"])) == ["a", "b"], "takewhile str pred"
assert list(itertools.filterfalse(echo, ["a", "", "b", ""])) == ["", ""], "filterfalse str pred"

# bare-value predicate: truthy iff non-zero
assert list(itertools.takewhile(echo, [1, 2, 3, 0, 4])) == [1, 2, 3], "takewhile value pred"
assert list(itertools.dropwhile(echo, [0, 0, 1, 0, 2])) == [0, 0, 1, 0, 2], "dropwhile value pred"

print("predicate_truthiness_not_strict_bool OK")
"###);
    assert_output(&out, r###"predicate_truthiness_not_strict_bool OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/itertools/product_cartesian_and_empty_rules.py`.
#[test]
fn test_gen_behavior_std_libs_itertools_product_cartesian_and_empty_rules() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "product_cartesian_and_empty_rules"
# subject = "itertools.product"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.product: product is the cartesian product; product() is [()], any empty pool collapses to [], size is the product of lengths"""
import itertools

prod = list(itertools.product([1, 2], [3, 4]))
assert prod == [(1, 3), (1, 4), (2, 3), (2, 4)], f"product = {prod!r}"
assert list(itertools.product()) == [()], "product() = [()]"
assert list(itertools.product([])) == [], "product([]) empty"
assert list(itertools.product(range(2), range(0), range(3))) == [], "empty pool collapses"
assert len(list(itertools.product(*[range(7)] * 2))) == 49, "product size"

print("product_cartesian_and_empty_rules OK")
"###);
    assert_output(&out, r###"product_cartesian_and_empty_rules OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/itertools/product_repeat_multiplies_pools.py`.
#[test]
fn test_gen_behavior_std_libs_itertools_product_repeat_multiplies_pools() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "product_repeat_multiplies_pools"
# subject = "itertools.product"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.product: product(*pools, repeat=k) equals repeating the pools k times, e.g. product('AB', repeat=2) == product('AB','AB')"""
import itertools

pp = list(itertools.product("AB", repeat=2))
assert pp == [("A", "A"), ("A", "B"), ("B", "A"), ("B", "B")], f"product repeat = {pp!r}"
assert list(itertools.product("AB", repeat=2)) == list(itertools.product("AB", "AB")), "repeat == args"

print("product_repeat_multiplies_pools OK")
"###);
    assert_output(&out, r###"product_repeat_multiplies_pools OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/itertools/repeat_finite_and_infinite.py`.
#[test]
fn test_gen_behavior_std_libs_itertools_repeat_finite_and_infinite() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "repeat_finite_and_infinite"
# subject = "itertools.repeat"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.repeat: repeat(obj, n) yields obj n times; repeat(obj) is infinite (one next() returns obj)"""
import itertools

assert list(itertools.repeat(5, 3)) == [5, 5, 5], "repeat finite"
assert list(itertools.repeat("x", 0)) == [], "repeat zero"
assert next(itertools.repeat(99)) == 99, "repeat infinite"

print("repeat_finite_and_infinite OK")
"###);
    assert_output(&out, r###"repeat_finite_and_infinite OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/itertools/repeat_negative_count_clamped_to_zero.py`.
#[test]
fn test_gen_behavior_std_libs_itertools_repeat_negative_count_clamped_to_zero() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "repeat_negative_count_clamped_to_zero"
# subject = "itertools.repeat"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.repeat: repeat clamps a negative times to zero: empty output and repr shows times=0 (positional and keyword)"""
import itertools

assert repr(itertools.repeat("a", -1)) == "repeat('a', 0)", repr(itertools.repeat("a", -1))
assert repr(itertools.repeat("a", times=-2)) == "repeat('a', 0)", "repeat kw repr"
assert list(itertools.repeat("a", -1)) == [], "repeat negative is empty"

print("repeat_negative_count_clamped_to_zero OK")
"###);
    assert_output(&out, r###"repeat_negative_count_clamped_to_zero OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/itertools/starmap_unpacks_tuples.py`.
#[test]
fn test_gen_behavior_std_libs_itertools_starmap_unpacks_tuples() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "starmap_unpacks_tuples"
# subject = "itertools.starmap"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.starmap: starmap applies the function to each argument tuple (unpacked), for user lambdas and builtins (pow, divmod)"""
import itertools

result = list(itertools.starmap(lambda a, b: a + b, [(1, 2), (3, 4), (5, 6)]))
assert result == [3, 7, 11], f"starmap = {result!r}"
assert list(itertools.starmap(pow, [(2, 3), (3, 2), (10, 2)])) == [8, 9, 100], "starmap pow"
assert list(itertools.starmap(divmod, [(10, 3), (15, 4)])) == [(3, 1), (3, 3)], "starmap divmod"
assert list(itertools.starmap(lambda a, b, c: a * b + c, [(1, 2, 3), (4, 5, 6)])) == [5, 26], "starmap 3-arg"

print("starmap_unpacks_tuples OK")
"###);
    assert_output(&out, r###"starmap_unpacks_tuples OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/itertools/takewhile_dropwhile_split.py`.
#[test]
fn test_gen_behavior_std_libs_itertools_takewhile_dropwhile_split() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "takewhile_dropwhile_split"
# subject = "itertools.takewhile"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.takewhile: takewhile yields the leading run where pred is true; dropwhile yields the rest once pred first fails"""
import itertools

assert list(itertools.takewhile(lambda x: x < 4, [1, 2, 3, 4, 5])) == [1, 2, 3], "takewhile"
assert list(itertools.dropwhile(lambda x: x < 4, [1, 2, 3, 4, 5])) == [4, 5], "dropwhile"
assert list(itertools.takewhile(lambda x: x < 0, [1, 2, 3])) == [], "takewhile none"
assert list(itertools.dropwhile(lambda x: x < 0, [1, 2, 3])) == [1, 2, 3], "dropwhile none"
assert list(itertools.dropwhile(lambda x: x < 100, [1, 2, 3])) == [], "dropwhile all"

print("takewhile_dropwhile_split OK")
"###);
    assert_output(&out, r###"takewhile_dropwhile_split OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/itertools/tee_independent_copies.py`.
#[test]
fn test_gen_behavior_std_libs_itertools_tee_independent_copies() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "tee_independent_copies"
# subject = "itertools.tee"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.tee: tee returns independent iterators over the same source; exhausting one leaves the other intact"""
import itertools

a, b = itertools.tee([1, 2, 3])
assert list(a) == [1, 2, 3], f"tee a = {list(a)!r}"

# Exhausting one branch leaves the other intact.
a2, b2 = itertools.tee([1, 2, 3])
list(a2)  # exhaust a2
assert list(b2) == [1, 2, 3], f"tee b independent = {list(b2)!r}"

print("tee_independent_copies OK")
"###);
    assert_output(&out, r###"tee_independent_copies OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/itertools/test_basic_ops__test_tee_dealloc_segfault.py`.
#[test]
fn test_gen_behavior_std_libs_itertools_test_basic_ops__test_tee_dealloc_segfault() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "test_basic_ops__test_tee_dealloc_segfault"
# subject = "cpython.test_itertools.TestBasicOps.test_tee_dealloc_segfault"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_itertools.py::TestBasicOps::test_tee_dealloc_segfault
"""Auto-ported test: TestBasicOps::test_tee_dealloc_segfault (CPython 3.12 oracle)."""


import doctest
import unittest
import itertools
from test import support
from test.support import threading_helper, script_helper
from itertools import *
import weakref
from decimal import Decimal
from fractions import Fraction
import operator
import random
import copy
import pickle
from functools import reduce
import sys
import struct
import threading
import gc
import warnings


def pickle_deprecated(testfunc):
    """ Run the test three times.
    First, verify that a Deprecation Warning is raised.
    Second, run normally but with DeprecationWarnings temporarily disabled.
    Third, run with warnings promoted to errors.
    """

    def inner(self):
        with self.assertWarns(DeprecationWarning):
            testfunc(self)
        with warnings.catch_warnings():
            warnings.simplefilter('ignore', category=DeprecationWarning)
            testfunc(self)
        with warnings.catch_warnings():
            warnings.simplefilter('error', category=DeprecationWarning)
            with self.assertRaises((DeprecationWarning, AssertionError, SystemError)):
                testfunc(self)
    return inner

maxsize = support.MAX_Py_ssize_t

minsize = -maxsize - 1

def lzip(*args):
    return list(zip(*args))

def onearg(x):
    """Test function of one argument"""
    return 2 * x

def errfunc(*args):
    """Test function that raises an error"""
    raise ValueError

def gen3():
    """Non-restartable source sequence"""
    for i in (0, 1, 2):
        yield i

def isEven(x):
    """Test predicate"""
    return x % 2 == 0

def isOdd(x):
    """Test predicate"""
    return x % 2 == 1

def tupleize(*args):
    return args

def irange(n):
    for i in range(n):
        yield i

class StopNow:
    """Class emulating an empty iterable."""

    def __iter__(self):
        return self

    def __next__(self):
        raise StopIteration

def take(n, seq):
    """Convenience function for partially consuming a long of infinite iterable"""
    return list(islice(seq, n))

def prod(iterable):
    return reduce(operator.mul, iterable, 1)

def fact(n):
    """Factorial"""
    return prod(range(1, n + 1))

def testR(r):
    return r[0]

def testR2(r):
    return r[2]

def underten(x):
    return x < 10

picklecopiers = [lambda s, proto=proto: pickle.loads(pickle.dumps(s, proto)) for proto in range(pickle.HIGHEST_PROTOCOL + 1)]

def R(seqn):
    """Regular generator"""
    for i in seqn:
        yield i

class G:
    """Sequence using __getitem__"""

    def __init__(self, seqn):
        self.seqn = seqn

    def __getitem__(self, i):
        return self.seqn[i]

class I:
    """Sequence using iterator protocol"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class Ig:
    """Sequence using iterator protocol defined with a generator"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        for val in self.seqn:
            yield val

class X:
    """Missing __getitem__ and __iter__"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class N:
    """Iterator missing __next__()"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

class E:
    """Test propagation of exceptions"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        3 // 0

class E2:
    """Test propagation of exceptions after two iterations"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        if self.i == 2:
            raise ZeroDivisionError
        v = self.seqn[self.i]
        self.i += 1
        return v

class S:
    """Test immediate stop"""

    def __init__(self, seqn):
        pass

    def __iter__(self):
        return self

    def __next__(self):
        raise StopIteration

def L(seqn):
    """Test multiple tiers of iterators"""
    return chain(map(lambda x: x, R(Ig(G(seqn)))))

def load_tests(loader, tests, pattern):
    tests.addTest(doctest.DocTestSuite(itertools))
    return tests


# --- test body ---
script = 'import typing, copyreg, itertools; copyreg.buggy_tee = itertools.tee(())'
script_helper.assert_python_ok('-c', script)
print("TestBasicOps::test_tee_dealloc_segfault: ok")
"###);
    assert_output(&out, r###"TestBasicOps::test_tee_dealloc_segfault: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/itertools/test_examples__test_filter.py`.
#[test]
fn test_gen_behavior_std_libs_itertools_test_examples__test_filter() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "test_examples__test_filter"
# subject = "cpython.test_itertools.TestExamples.test_filter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_itertools.py::TestExamples::test_filter
"""Auto-ported test: TestExamples::test_filter (CPython 3.12 oracle)."""


import doctest
import unittest
import itertools
from test import support
from test.support import threading_helper, script_helper
from itertools import *
import weakref
from decimal import Decimal
from fractions import Fraction
import operator
import random
import copy
import pickle
from functools import reduce
import sys
import struct
import threading
import gc
import warnings


def pickle_deprecated(testfunc):
    """ Run the test three times.
    First, verify that a Deprecation Warning is raised.
    Second, run normally but with DeprecationWarnings temporarily disabled.
    Third, run with warnings promoted to errors.
    """

    def inner(self):
        with self.assertWarns(DeprecationWarning):
            testfunc(self)
        with warnings.catch_warnings():
            warnings.simplefilter('ignore', category=DeprecationWarning)
            testfunc(self)
        with warnings.catch_warnings():
            warnings.simplefilter('error', category=DeprecationWarning)
            with self.assertRaises((DeprecationWarning, AssertionError, SystemError)):
                testfunc(self)
    return inner

maxsize = support.MAX_Py_ssize_t

minsize = -maxsize - 1

def lzip(*args):
    return list(zip(*args))

def onearg(x):
    """Test function of one argument"""
    return 2 * x

def errfunc(*args):
    """Test function that raises an error"""
    raise ValueError

def gen3():
    """Non-restartable source sequence"""
    for i in (0, 1, 2):
        yield i

def isEven(x):
    """Test predicate"""
    return x % 2 == 0

def isOdd(x):
    """Test predicate"""
    return x % 2 == 1

def tupleize(*args):
    return args

def irange(n):
    for i in range(n):
        yield i

class StopNow:
    """Class emulating an empty iterable."""

    def __iter__(self):
        return self

    def __next__(self):
        raise StopIteration

def take(n, seq):
    """Convenience function for partially consuming a long of infinite iterable"""
    return list(islice(seq, n))

def prod(iterable):
    return reduce(operator.mul, iterable, 1)

def fact(n):
    """Factorial"""
    return prod(range(1, n + 1))

def testR(r):
    return r[0]

def testR2(r):
    return r[2]

def underten(x):
    return x < 10

picklecopiers = [lambda s, proto=proto: pickle.loads(pickle.dumps(s, proto)) for proto in range(pickle.HIGHEST_PROTOCOL + 1)]

def R(seqn):
    """Regular generator"""
    for i in seqn:
        yield i

class G:
    """Sequence using __getitem__"""

    def __init__(self, seqn):
        self.seqn = seqn

    def __getitem__(self, i):
        return self.seqn[i]

class I:
    """Sequence using iterator protocol"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class Ig:
    """Sequence using iterator protocol defined with a generator"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        for val in self.seqn:
            yield val

class X:
    """Missing __getitem__ and __iter__"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class N:
    """Iterator missing __next__()"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

class E:
    """Test propagation of exceptions"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        3 // 0

class E2:
    """Test propagation of exceptions after two iterations"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        if self.i == 2:
            raise ZeroDivisionError
        v = self.seqn[self.i]
        self.i += 1
        return v

class S:
    """Test immediate stop"""

    def __init__(self, seqn):
        pass

    def __iter__(self):
        return self

    def __next__(self):
        raise StopIteration

def L(seqn):
    """Test multiple tiers of iterators"""
    return chain(map(lambda x: x, R(Ig(G(seqn)))))

def load_tests(loader, tests, pattern):
    tests.addTest(doctest.DocTestSuite(itertools))
    return tests


# --- test body ---

assert list(filter(lambda x: x % 2, range(10))) == [1, 3, 5, 7, 9]
print("TestExamples::test_filter: ok")
"###);
    assert_output(&out, r###"TestExamples::test_filter: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/itertools/test_examples__test_map.py`.
#[test]
fn test_gen_behavior_std_libs_itertools_test_examples__test_map() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "test_examples__test_map"
# subject = "cpython.test_itertools.TestExamples.test_map"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_itertools.py::TestExamples::test_map
"""Auto-ported test: TestExamples::test_map (CPython 3.12 oracle)."""


import doctest
import unittest
import itertools
from test import support
from test.support import threading_helper, script_helper
from itertools import *
import weakref
from decimal import Decimal
from fractions import Fraction
import operator
import random
import copy
import pickle
from functools import reduce
import sys
import struct
import threading
import gc
import warnings


def pickle_deprecated(testfunc):
    """ Run the test three times.
    First, verify that a Deprecation Warning is raised.
    Second, run normally but with DeprecationWarnings temporarily disabled.
    Third, run with warnings promoted to errors.
    """

    def inner(self):
        with self.assertWarns(DeprecationWarning):
            testfunc(self)
        with warnings.catch_warnings():
            warnings.simplefilter('ignore', category=DeprecationWarning)
            testfunc(self)
        with warnings.catch_warnings():
            warnings.simplefilter('error', category=DeprecationWarning)
            with self.assertRaises((DeprecationWarning, AssertionError, SystemError)):
                testfunc(self)
    return inner

maxsize = support.MAX_Py_ssize_t

minsize = -maxsize - 1

def lzip(*args):
    return list(zip(*args))

def onearg(x):
    """Test function of one argument"""
    return 2 * x

def errfunc(*args):
    """Test function that raises an error"""
    raise ValueError

def gen3():
    """Non-restartable source sequence"""
    for i in (0, 1, 2):
        yield i

def isEven(x):
    """Test predicate"""
    return x % 2 == 0

def isOdd(x):
    """Test predicate"""
    return x % 2 == 1

def tupleize(*args):
    return args

def irange(n):
    for i in range(n):
        yield i

class StopNow:
    """Class emulating an empty iterable."""

    def __iter__(self):
        return self

    def __next__(self):
        raise StopIteration

def take(n, seq):
    """Convenience function for partially consuming a long of infinite iterable"""
    return list(islice(seq, n))

def prod(iterable):
    return reduce(operator.mul, iterable, 1)

def fact(n):
    """Factorial"""
    return prod(range(1, n + 1))

def testR(r):
    return r[0]

def testR2(r):
    return r[2]

def underten(x):
    return x < 10

picklecopiers = [lambda s, proto=proto: pickle.loads(pickle.dumps(s, proto)) for proto in range(pickle.HIGHEST_PROTOCOL + 1)]

def R(seqn):
    """Regular generator"""
    for i in seqn:
        yield i

class G:
    """Sequence using __getitem__"""

    def __init__(self, seqn):
        self.seqn = seqn

    def __getitem__(self, i):
        return self.seqn[i]

class I:
    """Sequence using iterator protocol"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class Ig:
    """Sequence using iterator protocol defined with a generator"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        for val in self.seqn:
            yield val

class X:
    """Missing __getitem__ and __iter__"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class N:
    """Iterator missing __next__()"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

class E:
    """Test propagation of exceptions"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        3 // 0

class E2:
    """Test propagation of exceptions after two iterations"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        if self.i == 2:
            raise ZeroDivisionError
        v = self.seqn[self.i]
        self.i += 1
        return v

class S:
    """Test immediate stop"""

    def __init__(self, seqn):
        pass

    def __iter__(self):
        return self

    def __next__(self):
        raise StopIteration

def L(seqn):
    """Test multiple tiers of iterators"""
    return chain(map(lambda x: x, R(Ig(G(seqn)))))

def load_tests(loader, tests, pattern):
    tests.addTest(doctest.DocTestSuite(itertools))
    return tests


# --- test body ---

assert list(map(pow, (2, 3, 10), (5, 2, 3))) == [32, 9, 1000]
print("TestExamples::test_map: ok")
"###);
    assert_output(&out, r###"TestExamples::test_map: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/itertools/test_examples__test_zip.py`.
#[test]
fn test_gen_behavior_std_libs_itertools_test_examples__test_zip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "test_examples__test_zip"
# subject = "cpython.test_itertools.TestExamples.test_zip"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_itertools.py::TestExamples::test_zip
"""Auto-ported test: TestExamples::test_zip (CPython 3.12 oracle)."""


import doctest
import unittest
import itertools
from test import support
from test.support import threading_helper, script_helper
from itertools import *
import weakref
from decimal import Decimal
from fractions import Fraction
import operator
import random
import copy
import pickle
from functools import reduce
import sys
import struct
import threading
import gc
import warnings


def pickle_deprecated(testfunc):
    """ Run the test three times.
    First, verify that a Deprecation Warning is raised.
    Second, run normally but with DeprecationWarnings temporarily disabled.
    Third, run with warnings promoted to errors.
    """

    def inner(self):
        with self.assertWarns(DeprecationWarning):
            testfunc(self)
        with warnings.catch_warnings():
            warnings.simplefilter('ignore', category=DeprecationWarning)
            testfunc(self)
        with warnings.catch_warnings():
            warnings.simplefilter('error', category=DeprecationWarning)
            with self.assertRaises((DeprecationWarning, AssertionError, SystemError)):
                testfunc(self)
    return inner

maxsize = support.MAX_Py_ssize_t

minsize = -maxsize - 1

def lzip(*args):
    return list(zip(*args))

def onearg(x):
    """Test function of one argument"""
    return 2 * x

def errfunc(*args):
    """Test function that raises an error"""
    raise ValueError

def gen3():
    """Non-restartable source sequence"""
    for i in (0, 1, 2):
        yield i

def isEven(x):
    """Test predicate"""
    return x % 2 == 0

def isOdd(x):
    """Test predicate"""
    return x % 2 == 1

def tupleize(*args):
    return args

def irange(n):
    for i in range(n):
        yield i

class StopNow:
    """Class emulating an empty iterable."""

    def __iter__(self):
        return self

    def __next__(self):
        raise StopIteration

def take(n, seq):
    """Convenience function for partially consuming a long of infinite iterable"""
    return list(islice(seq, n))

def prod(iterable):
    return reduce(operator.mul, iterable, 1)

def fact(n):
    """Factorial"""
    return prod(range(1, n + 1))

def testR(r):
    return r[0]

def testR2(r):
    return r[2]

def underten(x):
    return x < 10

picklecopiers = [lambda s, proto=proto: pickle.loads(pickle.dumps(s, proto)) for proto in range(pickle.HIGHEST_PROTOCOL + 1)]

def R(seqn):
    """Regular generator"""
    for i in seqn:
        yield i

class G:
    """Sequence using __getitem__"""

    def __init__(self, seqn):
        self.seqn = seqn

    def __getitem__(self, i):
        return self.seqn[i]

class I:
    """Sequence using iterator protocol"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class Ig:
    """Sequence using iterator protocol defined with a generator"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        for val in self.seqn:
            yield val

class X:
    """Missing __getitem__ and __iter__"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class N:
    """Iterator missing __next__()"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

class E:
    """Test propagation of exceptions"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        3 // 0

class E2:
    """Test propagation of exceptions after two iterations"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        if self.i == 2:
            raise ZeroDivisionError
        v = self.seqn[self.i]
        self.i += 1
        return v

class S:
    """Test immediate stop"""

    def __init__(self, seqn):
        pass

    def __iter__(self):
        return self

    def __next__(self):
        raise StopIteration

def L(seqn):
    """Test multiple tiers of iterators"""
    return chain(map(lambda x: x, R(Ig(G(seqn)))))

def load_tests(loader, tests, pattern):
    tests.addTest(doctest.DocTestSuite(itertools))
    return tests


# --- test body ---

assert list(zip('ABCD', 'xy')) == [('A', 'x'), ('B', 'y')]
print("TestExamples::test_zip: ok")
"###);
    assert_output(&out, r###"TestExamples::test_zip: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/itertools/test_gc__test_filter.py`.
#[test]
fn test_gen_behavior_std_libs_itertools_test_gc__test_filter() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "test_gc__test_filter"
# subject = "cpython.test_itertools.TestGC.test_filter"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_itertools.py::TestGC::test_filter
"""Auto-ported test: TestGC::test_filter (CPython 3.12 oracle)."""


import doctest
import unittest
import itertools
from test import support
from test.support import threading_helper, script_helper
from itertools import *
import weakref
from decimal import Decimal
from fractions import Fraction
import operator
import random
import copy
import pickle
from functools import reduce
import sys
import struct
import threading
import gc
import warnings


def pickle_deprecated(testfunc):
    """ Run the test three times.
    First, verify that a Deprecation Warning is raised.
    Second, run normally but with DeprecationWarnings temporarily disabled.
    Third, run with warnings promoted to errors.
    """

    def inner(self):
        with self.assertWarns(DeprecationWarning):
            testfunc(self)
        with warnings.catch_warnings():
            warnings.simplefilter('ignore', category=DeprecationWarning)
            testfunc(self)
        with warnings.catch_warnings():
            warnings.simplefilter('error', category=DeprecationWarning)
            with self.assertRaises((DeprecationWarning, AssertionError, SystemError)):
                testfunc(self)
    return inner

maxsize = support.MAX_Py_ssize_t

minsize = -maxsize - 1

def lzip(*args):
    return list(zip(*args))

def onearg(x):
    """Test function of one argument"""
    return 2 * x

def errfunc(*args):
    """Test function that raises an error"""
    raise ValueError

def gen3():
    """Non-restartable source sequence"""
    for i in (0, 1, 2):
        yield i

def isEven(x):
    """Test predicate"""
    return x % 2 == 0

def isOdd(x):
    """Test predicate"""
    return x % 2 == 1

def tupleize(*args):
    return args

def irange(n):
    for i in range(n):
        yield i

class StopNow:
    """Class emulating an empty iterable."""

    def __iter__(self):
        return self

    def __next__(self):
        raise StopIteration

def take(n, seq):
    """Convenience function for partially consuming a long of infinite iterable"""
    return list(islice(seq, n))

def prod(iterable):
    return reduce(operator.mul, iterable, 1)

def fact(n):
    """Factorial"""
    return prod(range(1, n + 1))

def testR(r):
    return r[0]

def testR2(r):
    return r[2]

def underten(x):
    return x < 10

picklecopiers = [lambda s, proto=proto: pickle.loads(pickle.dumps(s, proto)) for proto in range(pickle.HIGHEST_PROTOCOL + 1)]

def R(seqn):
    """Regular generator"""
    for i in seqn:
        yield i

class G:
    """Sequence using __getitem__"""

    def __init__(self, seqn):
        self.seqn = seqn

    def __getitem__(self, i):
        return self.seqn[i]

class I:
    """Sequence using iterator protocol"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class Ig:
    """Sequence using iterator protocol defined with a generator"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        for val in self.seqn:
            yield val

class X:
    """Missing __getitem__ and __iter__"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class N:
    """Iterator missing __next__()"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

class E:
    """Test propagation of exceptions"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        3 // 0

class E2:
    """Test propagation of exceptions after two iterations"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        if self.i == 2:
            raise ZeroDivisionError
        v = self.seqn[self.i]
        self.i += 1
        return v

class S:
    """Test immediate stop"""

    def __init__(self, seqn):
        pass

    def __iter__(self):
        return self

    def __next__(self):
        raise StopIteration

def L(seqn):
    """Test multiple tiers of iterators"""
    return chain(map(lambda x: x, R(Ig(G(seqn)))))

def load_tests(loader, tests, pattern):
    tests.addTest(doctest.DocTestSuite(itertools))
    return tests


# --- test body ---
def makecycle(iterator, container):
    container.append(iterator)
    next(iterator)
    del container, iterator
a = []
makecycle(filter(lambda x: True, [a] * 2), a)
print("TestGC::test_filter: ok")
"###);
    assert_output(&out, r###"TestGC::test_filter: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/itertools/test_gc__test_map.py`.
#[test]
fn test_gen_behavior_std_libs_itertools_test_gc__test_map() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "test_gc__test_map"
# subject = "cpython.test_itertools.TestGC.test_map"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_itertools.py::TestGC::test_map
"""Auto-ported test: TestGC::test_map (CPython 3.12 oracle)."""


import doctest
import unittest
import itertools
from test import support
from test.support import threading_helper, script_helper
from itertools import *
import weakref
from decimal import Decimal
from fractions import Fraction
import operator
import random
import copy
import pickle
from functools import reduce
import sys
import struct
import threading
import gc
import warnings


def pickle_deprecated(testfunc):
    """ Run the test three times.
    First, verify that a Deprecation Warning is raised.
    Second, run normally but with DeprecationWarnings temporarily disabled.
    Third, run with warnings promoted to errors.
    """

    def inner(self):
        with self.assertWarns(DeprecationWarning):
            testfunc(self)
        with warnings.catch_warnings():
            warnings.simplefilter('ignore', category=DeprecationWarning)
            testfunc(self)
        with warnings.catch_warnings():
            warnings.simplefilter('error', category=DeprecationWarning)
            with self.assertRaises((DeprecationWarning, AssertionError, SystemError)):
                testfunc(self)
    return inner

maxsize = support.MAX_Py_ssize_t

minsize = -maxsize - 1

def lzip(*args):
    return list(zip(*args))

def onearg(x):
    """Test function of one argument"""
    return 2 * x

def errfunc(*args):
    """Test function that raises an error"""
    raise ValueError

def gen3():
    """Non-restartable source sequence"""
    for i in (0, 1, 2):
        yield i

def isEven(x):
    """Test predicate"""
    return x % 2 == 0

def isOdd(x):
    """Test predicate"""
    return x % 2 == 1

def tupleize(*args):
    return args

def irange(n):
    for i in range(n):
        yield i

class StopNow:
    """Class emulating an empty iterable."""

    def __iter__(self):
        return self

    def __next__(self):
        raise StopIteration

def take(n, seq):
    """Convenience function for partially consuming a long of infinite iterable"""
    return list(islice(seq, n))

def prod(iterable):
    return reduce(operator.mul, iterable, 1)

def fact(n):
    """Factorial"""
    return prod(range(1, n + 1))

def testR(r):
    return r[0]

def testR2(r):
    return r[2]

def underten(x):
    return x < 10

picklecopiers = [lambda s, proto=proto: pickle.loads(pickle.dumps(s, proto)) for proto in range(pickle.HIGHEST_PROTOCOL + 1)]

def R(seqn):
    """Regular generator"""
    for i in seqn:
        yield i

class G:
    """Sequence using __getitem__"""

    def __init__(self, seqn):
        self.seqn = seqn

    def __getitem__(self, i):
        return self.seqn[i]

class I:
    """Sequence using iterator protocol"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class Ig:
    """Sequence using iterator protocol defined with a generator"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        for val in self.seqn:
            yield val

class X:
    """Missing __getitem__ and __iter__"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class N:
    """Iterator missing __next__()"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

class E:
    """Test propagation of exceptions"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        3 // 0

class E2:
    """Test propagation of exceptions after two iterations"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        if self.i == 2:
            raise ZeroDivisionError
        v = self.seqn[self.i]
        self.i += 1
        return v

class S:
    """Test immediate stop"""

    def __init__(self, seqn):
        pass

    def __iter__(self):
        return self

    def __next__(self):
        raise StopIteration

def L(seqn):
    """Test multiple tiers of iterators"""
    return chain(map(lambda x: x, R(Ig(G(seqn)))))

def load_tests(loader, tests, pattern):
    tests.addTest(doctest.DocTestSuite(itertools))
    return tests


# --- test body ---
def makecycle(iterator, container):
    container.append(iterator)
    next(iterator)
    del container, iterator
a = []
makecycle(map(lambda x: x, [a] * 2), a)
print("TestGC::test_map: ok")
"###);
    assert_output(&out, r###"TestGC::test_map: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/itertools/test_gc__test_zip.py`.
#[test]
fn test_gen_behavior_std_libs_itertools_test_gc__test_zip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "test_gc__test_zip"
# subject = "cpython.test_itertools.TestGC.test_zip"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
# Auto-ported from CPython 3.12 test_itertools.py::TestGC::test_zip
"""Auto-ported test: TestGC::test_zip (CPython 3.12 oracle)."""


import doctest
import unittest
import itertools
from test import support
from test.support import threading_helper, script_helper
from itertools import *
import weakref
from decimal import Decimal
from fractions import Fraction
import operator
import random
import copy
import pickle
from functools import reduce
import sys
import struct
import threading
import gc
import warnings


def pickle_deprecated(testfunc):
    """ Run the test three times.
    First, verify that a Deprecation Warning is raised.
    Second, run normally but with DeprecationWarnings temporarily disabled.
    Third, run with warnings promoted to errors.
    """

    def inner(self):
        with self.assertWarns(DeprecationWarning):
            testfunc(self)
        with warnings.catch_warnings():
            warnings.simplefilter('ignore', category=DeprecationWarning)
            testfunc(self)
        with warnings.catch_warnings():
            warnings.simplefilter('error', category=DeprecationWarning)
            with self.assertRaises((DeprecationWarning, AssertionError, SystemError)):
                testfunc(self)
    return inner

maxsize = support.MAX_Py_ssize_t

minsize = -maxsize - 1

def lzip(*args):
    return list(zip(*args))

def onearg(x):
    """Test function of one argument"""
    return 2 * x

def errfunc(*args):
    """Test function that raises an error"""
    raise ValueError

def gen3():
    """Non-restartable source sequence"""
    for i in (0, 1, 2):
        yield i

def isEven(x):
    """Test predicate"""
    return x % 2 == 0

def isOdd(x):
    """Test predicate"""
    return x % 2 == 1

def tupleize(*args):
    return args

def irange(n):
    for i in range(n):
        yield i

class StopNow:
    """Class emulating an empty iterable."""

    def __iter__(self):
        return self

    def __next__(self):
        raise StopIteration

def take(n, seq):
    """Convenience function for partially consuming a long of infinite iterable"""
    return list(islice(seq, n))

def prod(iterable):
    return reduce(operator.mul, iterable, 1)

def fact(n):
    """Factorial"""
    return prod(range(1, n + 1))

def testR(r):
    return r[0]

def testR2(r):
    return r[2]

def underten(x):
    return x < 10

picklecopiers = [lambda s, proto=proto: pickle.loads(pickle.dumps(s, proto)) for proto in range(pickle.HIGHEST_PROTOCOL + 1)]

def R(seqn):
    """Regular generator"""
    for i in seqn:
        yield i

class G:
    """Sequence using __getitem__"""

    def __init__(self, seqn):
        self.seqn = seqn

    def __getitem__(self, i):
        return self.seqn[i]

class I:
    """Sequence using iterator protocol"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class Ig:
    """Sequence using iterator protocol defined with a generator"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        for val in self.seqn:
            yield val

class X:
    """Missing __getitem__ and __iter__"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __next__(self):
        if self.i >= len(self.seqn):
            raise StopIteration
        v = self.seqn[self.i]
        self.i += 1
        return v

class N:
    """Iterator missing __next__()"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

class E:
    """Test propagation of exceptions"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        3 // 0

class E2:
    """Test propagation of exceptions after two iterations"""

    def __init__(self, seqn):
        self.seqn = seqn
        self.i = 0

    def __iter__(self):
        return self

    def __next__(self):
        if self.i == 2:
            raise ZeroDivisionError
        v = self.seqn[self.i]
        self.i += 1
        return v

class S:
    """Test immediate stop"""

    def __init__(self, seqn):
        pass

    def __iter__(self):
        return self

    def __next__(self):
        raise StopIteration

def L(seqn):
    """Test multiple tiers of iterators"""
    return chain(map(lambda x: x, R(Ig(G(seqn)))))

def load_tests(loader, tests, pattern):
    tests.addTest(doctest.DocTestSuite(itertools))
    return tests


# --- test body ---
def makecycle(iterator, container):
    container.append(iterator)
    next(iterator)
    del container, iterator
a = []
makecycle(zip([a] * 2, [a] * 3), a)
print("TestGC::test_zip: ok")
"###);
    assert_output(&out, r###"TestGC::test_zip: ok
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/itertools/zip_longest_pads_with_fillvalue.py`.
#[test]
fn test_gen_behavior_std_libs_itertools_zip_longest_pads_with_fillvalue() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "zip_longest_pads_with_fillvalue"
# subject = "itertools.zip_longest"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.zip_longest: zip_longest pads every short column to the longest length, default fill None, custom via fillvalue"""
import itertools

z = list(itertools.zip_longest([1, 2, 3], [4, 5], fillvalue=0))
assert z == [(1, 4), (2, 5), (3, 0)], f"zip_longest = {z!r}"

default = list(itertools.zip_longest([1, 2], [3, 4, 5]))
assert default == [(1, 3), (2, 4), (None, 5)], f"default fill = {default!r}"

multi = list(itertools.zip_longest(range(3), range(1), range(2)))
assert multi == [(0, 0, 0), (1, None, 1), (2, None, None)], f"zip_longest multi = {multi!r}"

print("zip_longest_pads_with_fillvalue OK")
"###);
    assert_output(&out, r###"zip_longest_pads_with_fillvalue OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/itertools/zip_longest_swallows_source_stopiteration.py`.
#[test]
fn test_gen_behavior_std_libs_itertools_zip_longest_swallows_source_stopiteration() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "zip_longest_swallows_source_stopiteration"
# subject = "itertools.zip_longest"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.zip_longest: zip_longest treats a source's StopIteration as exhaustion and pads with fillvalue (bug 7244)"""
import itertools

class Repeater:
    """Yields `o` exactly `t` times, then raises `e`."""

    def __init__(self, o, t, e):
        self.o = o
        self.t = t
        self.e = e

    def __iter__(self):
        return self

    def __next__(self):
        if self.t > 0:
            self.t -= 1
            return self.o
        raise self.e

r1 = Repeater(1, 3, StopIteration)
r2 = Repeater(2, 4, StopIteration)
got = list(itertools.zip_longest(r1, r2, fillvalue=0))
assert got == [(1, 2), (1, 2), (1, 2), (0, 2)], f"zip_longest fill = {got!r}"

print("zip_longest_swallows_source_stopiteration OK")
"###);
    assert_output(&out, r###"zip_longest_swallows_source_stopiteration OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/itertools/zip_stops_at_shortest.py`.
#[test]
fn test_gen_behavior_std_libs_itertools_zip_stops_at_shortest() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "itertools"
# dimension = "behavior"
# case = "zip_stops_at_shortest"
# subject = "itertools.count"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_itertools.py"
# status = "filled"
# ///
"""itertools.count: builtin zip pairs with count() and stops at the shortest input"""
import itertools

assert list(zip("abc", itertools.count())) == [("a", 0), ("b", 1), ("c", 2)], "zip+count"
assert list(zip("abcdef", range(3))) == [("a", 0), ("b", 1), ("c", 2)], "zip shortest"

print("zip_stops_at_shortest OK")
"###);
    assert_output(&out, r###"zip_stops_at_shortest OK
"###);
}
