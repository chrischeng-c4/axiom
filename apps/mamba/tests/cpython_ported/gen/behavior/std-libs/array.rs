use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/array/append_pop_lifo_at_end.py`.
#[test]
fn test_gen_behavior_std_libs_array_append_pop_lifo_at_end() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "behavior"
# case = "append_pop_lifo_at_end"
# subject = "array.array"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_array.py"
# status = "filled"
# ///
"""array.array: append adds at the end and pop()/pop(0) remove from end/front like a list"""
import array

a = array.array("i", [1, 2, 3])
a.append(4)
assert a[-1] == 4, f"appended at end = {a[-1]!r}"
last = a.pop()
assert last == 4, f"popped last = {last!r}"
first = a.pop(0)
assert first == 1, f"pop(0) = {first!r}"
assert a.tolist() == [2, 3], f"after two pops = {a.tolist()!r}"

print("append_pop_lifo_at_end OK")
"###);
    assert_output(&out, r###"append_pop_lifo_at_end OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/array/buffer_info_reports_count.py`.
#[test]
fn test_gen_behavior_std_libs_array_buffer_info_reports_count() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "behavior"
# case = "buffer_info_reports_count"
# subject = "array.array"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_array.py"
# status = "filled"
# ///
"""array.array: buffer_info() returns a (address, element_count) tuple whose second item equals len(array)"""
import array

a = array.array("i", [1, 2, 3])
bi = a.buffer_info()
assert isinstance(bi, tuple), f"buffer_info type = {type(bi)!r}"
assert bi[1] == len(a), f"buffer_info count = {bi[1]!r}"

print("buffer_info_reports_count OK")
"###);
    assert_output(&out, r###"buffer_info_reports_count OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/array/byte_array_typecode_itemsize.py`.
#[test]
fn test_gen_behavior_std_libs_array_byte_array_typecode_itemsize() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "behavior"
# case = "byte_array_typecode_itemsize"
# subject = "array.array"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""array.array: a 'b' (signed char) array reports typecode 'b', itemsize 1, and stores values across the -128..127 range"""
import array

a = array.array("b", [-1, 0, 127])
assert a.typecode == "b", f"typecode = {a.typecode!r}"
assert a.itemsize == 1, f"itemsize = {a.itemsize!r}"
assert a.tolist() == [-1, 0, 127], f"values = {a.tolist()!r}"

print("byte_array_typecode_itemsize OK")
"###);
    assert_output(&out, r###"byte_array_typecode_itemsize OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/array/bytes_initializer_consumed_as_raw_words.py`.
#[test]
fn test_gen_behavior_std_libs_array_bytes_initializer_consumed_as_raw_words() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "behavior"
# case = "bytes_initializer_consumed_as_raw_words"
# subject = "array.array"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_array.py"
# status = "filled"
# ///
"""array.array: a bytes initializer is consumed as raw machine words, not values; b'1234' (4 bytes) into typecode 'H' (itemsize 2) yields 2 elements"""
import array

# b'1234' is 4 bytes; for typecode 'H' (itemsize 2) that is 2 elements.
a = array.array("H", b"1234")
assert len(a) * a.itemsize == 4, f"raw bytes consumed = {len(a) * a.itemsize!r}"
assert len(a) == 2, f"H elements from 4 bytes = {len(a)!r}"

print("bytes_initializer_consumed_as_raw_words OK")
"###);
    assert_output(&out, r###"bytes_initializer_consumed_as_raw_words OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/array/byteswap_reverses_element_bytes.py`.
#[test]
fn test_gen_behavior_std_libs_array_byteswap_reverses_element_bytes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "behavior"
# case = "byteswap_reverses_element_bytes"
# subject = "array.array"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_array.py"
# status = "filled"
# ///
"""array.array: byteswap reverses the byte order of each element; 0x0001 in an 'H' array becomes 0x0100 (256)"""
import array

bs = array.array("H", [1])  # 0x0001 -> 0x0100 = 256
bs.byteswap()
assert bs[0] == 256, f"byteswap = {bs[0]!r}"

print("byteswap_reverses_element_bytes OK")
"###);
    assert_output(&out, r###"byteswap_reverses_element_bytes OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/array/concat_same_typecode.py`.
#[test]
fn test_gen_behavior_std_libs_array_concat_same_typecode() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "behavior"
# case = "concat_same_typecode"
# subject = "array.array"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_array.py"
# status = "filled"
# ///
"""array.array: the + operator concatenates two same-typecode arrays into a new array"""
import array

x = array.array("i", [1, 2, 3])
y = array.array("i", [4, 5])
joined = x + y
assert isinstance(joined, array.array), f"concat type = {type(joined)!r}"
assert joined.tolist() == [1, 2, 3, 4, 5], f"concat = {joined.tolist()!r}"

print("concat_same_typecode OK")
"###);
    assert_output(&out, r###"concat_same_typecode OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/array/count_and_index_like_list.py`.
#[test]
fn test_gen_behavior_std_libs_array_count_and_index_like_list() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "behavior"
# case = "count_and_index_like_list"
# subject = "array.array"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_array.py"
# status = "filled"
# ///
"""array.array: count returns the number of matching elements and index returns the position of the first match, like list"""
import array

a = array.array("i", [1, 2, 2, 3, 2])
assert a.count(2) == 3, f"count(2) = {a.count(2)!r}"
assert a.index(3) == 3, f"index(3) = {a.index(3)!r}"

print("count_and_index_like_list OK")
"###);
    assert_output(&out, r###"count_and_index_like_list OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/array/double_array_typecode_itemsize.py`.
#[test]
fn test_gen_behavior_std_libs_array_double_array_typecode_itemsize() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "behavior"
# case = "double_array_typecode_itemsize"
# subject = "array.array"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_array.py"
# status = "filled"
# ///
"""array.array: a 'd' (double) array reports typecode 'd' and itemsize 8 and stores floats with full precision"""
import array

a = array.array("d", [1.1, 2.2, 3.3])
assert a.typecode == "d", f"typecode = {a.typecode!r}"
assert a.itemsize == 8, f"itemsize = {a.itemsize!r}"
assert abs(a[0] - 1.1) < 1e-10, f"float stored = {a[0]!r}"

print("double_array_typecode_itemsize OK")
"###);
    assert_output(&out, r###"double_array_typecode_itemsize OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/array/double_tobytes_frombytes_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_array_double_tobytes_frombytes_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "behavior"
# case = "double_tobytes_frombytes_roundtrip"
# subject = "array.array"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_array.py"
# status = "filled"
# ///
"""array.array: a 'd' array round-trips through tobytes()/frombytes(); 3 doubles serialize to 24 bytes and reload to the same values"""
import array

a = array.array("d", [1.1, 2.2, 3.3])
raw = a.tobytes()
assert len(raw) == 24, f"d bytes = {len(raw)!r}"  # 3 * 8 bytes
b = array.array("d")
b.frombytes(raw)
assert abs(b[1] - 2.2) < 1e-10, f"float frombytes = {b[1]!r}"

print("double_tobytes_frombytes_roundtrip OK")
"###);
    assert_output(&out, r###"double_tobytes_frombytes_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/array/empty_array_ops_stay_empty.py`.
#[test]
fn test_gen_behavior_std_libs_array_empty_array_ops_stay_empty() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "behavior"
# case = "empty_array_ops_stay_empty"
# subject = "array.array"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_array.py"
# status = "filled"
# ///
"""array.array: self slice-assign, concat, repeat and in-place concat on a zero-length array all leave it empty rather than raising; empty tolist()/tobytes() are empty"""
import array

a = array.array("B")
# Assigning an array to its own full slice is a no-op on an empty array.
a[:] = a
assert len(a) == 0, f"after self slice-assign = {len(a)!r}"
# Concatenation, repetition, in-place concat of empties stay empty.
assert len(a + a) == 0, f"empty + empty = {len(a + a)!r}"
assert len(a * 3) == 0, f"empty * 3 = {len(a * 3)!r}"
assert len(a * 0) == 0, f"empty * 0 = {len(a * 0)!r}"
a += a
assert len(a) == 0, f"empty += empty = {len(a)!r}"
# tolist and tobytes of an empty array are empty too.
assert a.tolist() == [], f"empty tolist = {a.tolist()!r}"
assert a.tobytes() == b"", f"empty tobytes = {a.tobytes()!r}"

print("empty_array_ops_stay_empty OK")
"###);
    assert_output(&out, r###"empty_array_ops_stay_empty OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/array/extend_accepts_list_and_array.py`.
#[test]
fn test_gen_behavior_std_libs_array_extend_accepts_list_and_array() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "behavior"
# case = "extend_accepts_list_and_array"
# subject = "array.array"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_array.py"
# status = "filled"
# ///
"""array.array: extend appends from any iterable, accepting both a plain list and another array of the same typecode"""
import array

a = array.array("i", [1, 2])
a.extend([3, 4, 5])
assert a.tolist() == [1, 2, 3, 4, 5], f"extend list = {a.tolist()!r}"
b = array.array("i", [6, 7])
a.extend(b)
assert a.tolist() == [1, 2, 3, 4, 5, 6, 7], f"extend array = {a.tolist()!r}"

print("extend_accepts_list_and_array OK")
"###);
    assert_output(&out, r###"extend_accepts_list_and_array OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/array/fromlist_appends_values.py`.
#[test]
fn test_gen_behavior_std_libs_array_fromlist_appends_values() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "behavior"
# case = "fromlist_appends_values"
# subject = "array.array"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_array.py"
# status = "filled"
# ///
"""array.array: fromlist appends each value to the end of the array and membership tests scan by value"""
import array

a = array.array("i", [1])
a.fromlist([2, 3])
assert a.tolist() == [1, 2, 3], f"fromlist = {a.tolist()!r}"
assert 2 in a, "value present"
assert 9 not in a, "value absent"

print("fromlist_appends_values OK")
"###);
    assert_output(&out, r###"fromlist_appends_values OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/array/insert_places_at_index.py`.
#[test]
fn test_gen_behavior_std_libs_array_insert_places_at_index() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "behavior"
# case = "insert_places_at_index"
# subject = "array.array"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_array.py"
# status = "filled"
# ///
"""array.array: insert places an element at the given index, including a negative index meaning 'before last'"""
import array

a = array.array("i", [1, 2, 3])
a.insert(1, 99)
assert a.tolist() == [1, 99, 2, 3], f"after insert = {a.tolist()!r}"
a.insert(-1, 88)  # insert before last
assert a[-2] == 88, f"insert before last = {a[-2]!r}"

print("insert_places_at_index OK")
"###);
    assert_output(&out, r###"insert_places_at_index OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/array/int_array_construct_and_index.py`.
#[test]
fn test_gen_behavior_std_libs_array_int_array_construct_and_index() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "behavior"
# case = "int_array_construct_and_index"
# subject = "array.array"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_array.py"
# status = "filled"
# ///
"""array.array: array('i', [1,2,3]) reports typecode 'i', itemsize 4, len 3, and supports positive and negative indexing"""
import array

a = array.array("i", [1, 2, 3])
assert isinstance(a, array.array), f"array type = {type(a)!r}"
assert a.typecode == "i", f"typecode = {a.typecode!r}"
assert a.itemsize == 4, f"itemsize = {a.itemsize!r}"
assert len(a) == 3, f"len = {len(a)!r}"
assert a[0] == 1, f"a[0] = {a[0]!r}"
assert a[-1] == 3, f"a[-1] = {a[-1]!r}"

print("int_array_construct_and_index OK")
"###);
    assert_output(&out, r###"int_array_construct_and_index OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/array/lexicographic_comparison.py`.
#[test]
fn test_gen_behavior_std_libs_array_lexicographic_comparison() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "behavior"
# case = "lexicographic_comparison"
# subject = "array.array"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_array.py"
# status = "filled"
# ///
"""array.array: arrays compare lexicographically element by element, with a longer prefix-equal array comparing greater"""
import array

assert array.array("i", [1, 2]) == array.array("i", [1, 2]), "equal"
assert array.array("i", [1, 2]) < array.array("i", [1, 3]), "less-than"
assert array.array("i", [1, 2, 3]) > array.array("i", [1, 2]), "longer prefix-equal is greater"

print("lexicographic_comparison OK")
"###);
    assert_output(&out, r###"lexicographic_comparison OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/array/nan_array_never_equal_or_ordered.py`.
#[test]
fn test_gen_behavior_std_libs_array_nan_array_never_equal_or_ordered() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "behavior"
# case = "nan_array_never_equal_or_ordered"
# subject = "array.array"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_array.py"
# status = "filled"
# ///
"""array.array: two 'd' arrays each holding a single NaN follow IEEE-754: != is True, every other relation (==, <, <=, >, >=, even == itself) is False"""
import array

a = array.array("d", [float("nan")])
b = array.array("d", [float("nan")])
# Inequality is the only relation that holds.
assert (a != b) is True, "nan array != nan array"
# Every other relation is False.
assert (a == b) is False, "nan array not equal"
assert (a > b) is False, "nan array not greater"
assert (a >= b) is False, "nan array not greater-or-equal"
assert (a < b) is False, "nan array not less"
assert (a <= b) is False, "nan array not less-or-equal"
# An array even compares unequal to itself when it contains NaN.
assert (a == a) is False, "nan array unequal to itself"
assert (a != a) is True, "nan array != itself"

print("nan_array_never_equal_or_ordered OK")
"###);
    assert_output(&out, r###"nan_array_never_equal_or_ordered OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/array/registers_as_mutable_sequence.py`.
#[test]
fn test_gen_behavior_std_libs_array_registers_as_mutable_sequence() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "behavior"
# case = "registers_as_mutable_sequence"
# subject = "array.array"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_array.py"
# status = "filled"
# ///
"""array.array: an array is an instance of collections.abc.MutableSequence and collections.abc.Reversible"""
import array

import collections.abc

a = array.array("i", [1, 2, 3])
assert isinstance(a, collections.abc.MutableSequence), "array is MutableSequence"
assert isinstance(a, collections.abc.Reversible), "array is Reversible"

print("registers_as_mutable_sequence OK")
"###);
    assert_output(&out, r###"registers_as_mutable_sequence OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/array/remove_deletes_first_occurrence.py`.
#[test]
fn test_gen_behavior_std_libs_array_remove_deletes_first_occurrence() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "behavior"
# case = "remove_deletes_first_occurrence"
# subject = "array.array"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_array.py"
# status = "filled"
# ///
"""array.array: remove deletes only the first matching occurrence, leaving later duplicates intact"""
import array

a = array.array("i", [1, 2, 3, 2, 4])
a.remove(2)
assert a.tolist() == [1, 3, 2, 4], f"remove first = {a.tolist()!r}"

print("remove_deletes_first_occurrence OK")
"###);
    assert_output(&out, r###"remove_deletes_first_occurrence OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/array/repeat_tiles_elements.py`.
#[test]
fn test_gen_behavior_std_libs_array_repeat_tiles_elements() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "behavior"
# case = "repeat_tiles_elements"
# subject = "array.array"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_array.py"
# status = "filled"
# ///
"""array.array: the * operator tiles the elements n times into a new array"""
import array

tiled = array.array("i", [1, 2]) * 3
assert tiled.tolist() == [1, 2, 1, 2, 1, 2], f"repeat = {tiled.tolist()!r}"

print("repeat_tiles_elements OK")
"###);
    assert_output(&out, r###"repeat_tiles_elements OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/array/reverse_in_place.py`.
#[test]
fn test_gen_behavior_std_libs_array_reverse_in_place() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "behavior"
# case = "reverse_in_place"
# subject = "array.array"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_array.py"
# status = "filled"
# ///
"""array.array: reverse reverses the array elements in place"""
import array

a = array.array("i", [3, 1, 4, 1, 5])
a.reverse()
assert a.tolist() == [5, 1, 4, 1, 3], f"reversed = {a.tolist()!r}"

print("reverse_in_place OK")
"###);
    assert_output(&out, r###"reverse_in_place OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/array/slice_assignment_changes_length.py`.
#[test]
fn test_gen_behavior_std_libs_array_slice_assignment_changes_length() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "behavior"
# case = "slice_assignment_changes_length"
# subject = "array.array"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_array.py"
# status = "filled"
# ///
"""array.array: assigning an array to a slice can change the length of the target array"""
import array

s = array.array("i", [0, 1, 2, 3, 4])
s[1:3] = array.array("i", [10, 20, 30])
assert s.tolist() == [0, 10, 20, 30, 3, 4], f"slice-assign = {s.tolist()!r}"

print("slice_assignment_changes_length OK")
"###);
    assert_output(&out, r###"slice_assignment_changes_length OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/array/slice_deletion_removes_range.py`.
#[test]
fn test_gen_behavior_std_libs_array_slice_deletion_removes_range() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "behavior"
# case = "slice_deletion_removes_range"
# subject = "array.array"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_array.py"
# status = "filled"
# ///
"""array.array: del array[a:b] removes the contiguous range of elements"""
import array

d = array.array("i", [0, 1, 2, 3, 4])
del d[1:3]
assert d.tolist() == [0, 3, 4], f"del-slice = {d.tolist()!r}"

print("slice_deletion_removes_range OK")
"###);
    assert_output(&out, r###"slice_deletion_removes_range OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/array/slice_returns_new_array_same_typecode.py`.
#[test]
fn test_gen_behavior_std_libs_array_slice_returns_new_array_same_typecode() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "behavior"
# case = "slice_returns_new_array_same_typecode"
# subject = "array.array"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_array.py"
# status = "filled"
# ///
"""array.array: slicing returns a new array of the same typecode holding the sliced values"""
import array

a = array.array("i", [0, 1, 2, 3, 4])
s = a[1:4]
assert isinstance(s, array.array), f"slice type = {type(s)!r}"
assert s.typecode == "i", f"slice typecode = {s.typecode!r}"
assert s.tolist() == [1, 2, 3], f"slice values = {s.tolist()!r}"

print("slice_returns_new_array_same_typecode OK")
"###);
    assert_output(&out, r###"slice_returns_new_array_same_typecode OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/array/tobytes_frombytes_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_array_tobytes_frombytes_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "behavior"
# case = "tobytes_frombytes_roundtrip"
# subject = "array.array"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_array.py"
# status = "filled"
# ///
"""array.array: tobytes()/frombytes() round-trip preserves int values; 3 int32 elements serialize to 12 bytes"""
import array

a = array.array("i", [10, 20, 30])
raw = a.tobytes()
assert isinstance(raw, bytes), f"tobytes type = {type(raw)!r}"
assert len(raw) == 12, f"bytes len = {len(raw)!r}"  # 3 * 4 bytes
b = array.array("i")
b.frombytes(raw)
assert b.tolist() == [10, 20, 30], f"frombytes = {b.tolist()!r}"

print("tobytes_frombytes_roundtrip OK")
"###);
    assert_output(&out, r###"tobytes_frombytes_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/array/typecodes_lists_core_codes.py`.
#[test]
fn test_gen_behavior_std_libs_array_typecodes_lists_core_codes() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "behavior"
# case = "typecodes_lists_core_codes"
# subject = "array.typecodes"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_array.py"
# status = "filled"
# ///
"""array.typecodes: array.typecodes is a str containing every core integer and float code in 'bBhHiIlLqQfd', and each advertised code constructs an empty array of that code"""
import array

assert isinstance(array.typecodes, str), f"typecodes type = {type(array.typecodes)!r}"
for code in "bBhHiIlLqQfd":
    assert code in array.typecodes, f"{code!r} in typecodes"
# Every advertised typecode constructs an empty array of that code.
for code in array.typecodes:
    ac = array.array(code)
    assert ac.typecode == code, f"constructed typecode = {ac.typecode!r}"

print("typecodes_lists_core_codes OK")
"###);
    assert_output(&out, r###"typecodes_lists_core_codes OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/array/unicode_typecode_roundtrips_text.py`.
#[test]
fn test_gen_behavior_std_libs_array_unicode_typecode_roundtrips_text() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "array"
# dimension = "behavior"
# case = "unicode_typecode_roundtrips_text"
# subject = "array.array"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_array.py"
# status = "filled"
# ///
"""array.array: the 'u' typecode round-trips text via fromunicode/tounicode and accepts a str initializer while integer typecodes reject one (DeprecationWarning for 'u' silenced)"""
import array

import warnings

warnings.simplefilter("ignore", DeprecationWarning)

# Build from a str, then append more characters with fromunicode.
a = array.array("u", "\xa0\xc2ሴ")
a.fromunicode("")  # appending the empty string is a no-op
a.fromunicode("\x11abc\xffሴ")
assert a.tounicode() == "\xa0\xc2ሴ\x11abc\xffሴ", "fromunicode/tounicode round-trip"
# Each element is one wide character; itemsize matches the platform wchar.
assert a.itemsize == array.array("u").itemsize, "u itemsize stable"
assert a.typecode == "u", f"typecode = {a.typecode!r}"

print("unicode_typecode_roundtrips_text OK")
"###);
    assert_output(&out, r###"unicode_typecode_roundtrips_text OK
"###);
}
