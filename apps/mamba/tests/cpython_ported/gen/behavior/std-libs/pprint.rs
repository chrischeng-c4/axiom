use super::super::super::super::harness::*;

/// Ported from `tests/cpython/behavior/std-libs/pprint/collections_repr.py`.
#[test]
fn test_gen_behavior_std_libs_pprint_collections_repr() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pprint"
# dimension = "behavior"
# case = "collections_repr"
# subject = "pprint.pformat"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pprint.py"
# status = "filled"
# ///
"""pprint.pformat: pprint keeps each container's repr prefix (Counter/OrderedDict/deque/ChainMap/defaultdict/mappingproxy, deque maxlen=) while wrapping contents, and User* wrappers format like their builtin"""
import collections
import itertools
import types

import pprint

words = "the quick brown fox jumped over a lazy dog".split()

# Empty containers keep their type prefix even at width=1.
assert pprint.pformat(collections.Counter(), width=1) == "Counter()"
assert pprint.pformat(collections.OrderedDict(), width=1) == "OrderedDict()"
assert pprint.pformat(collections.deque(), width=1) == "deque([])"
assert pprint.pformat(collections.ChainMap(), width=1) == "ChainMap({})"
assert pprint.pformat(collections.defaultdict(int), width=1) == \
    "defaultdict(<class 'int'>, {})"

# Counter wraps and keeps insertion (count-descending) order, not sorted.
c = collections.Counter("senselessness")
assert pprint.pformat(c, width=40) == (
    "Counter({'s': 6,\n         'e': 4,\n         'n': 2,\n         'l': 1})"
)

# deque with a maxlen surfaces the maxlen= keyword in the repr.
dq = collections.deque(zip(words, itertools.count()), maxlen=7)
assert pprint.pformat(dq) == (
    "deque([('brown', 2),\n       ('fox', 3),\n       ('jumped', 4),\n"
    "       ('over', 5),\n       ('a', 6),\n       ('lazy', 7),\n"
    "       ('dog', 8)],\n      maxlen=7)"
)

# OrderedDict preserves insertion order inside an OrderedDict({...}) wrap.
od = collections.OrderedDict(zip(words[:3], itertools.count()))
assert pprint.pformat(od) == \
    "OrderedDict({'the': 0, 'quick': 1, 'brown': 2})"

# mappingproxy delegates to the underlying mapping (here keeping its order).
mp = types.MappingProxyType({"b": 2, "a": 1})
assert pprint.pformat(mp) == "mappingproxy({'b': 2, 'a': 1})"

# User* wrappers format identically to the builtin they emulate.
assert pprint.pformat(collections.UserDict(), width=1) == "{}"
assert pprint.pformat(collections.UserList(), width=1) == "[]"
assert pprint.pformat(collections.UserString(""), width=1) == "''"
assert pprint.pformat(collections.UserDict({"a": 1})) == "{'a': 1}"
print("collections_repr OK")
"###);
    assert_output(&out, r###"collections_repr OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pprint/depth_truncation.py`.
#[test]
fn test_gen_behavior_std_libs_pprint_depth_truncation() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pprint"
# dimension = "behavior"
# case = "depth_truncation"
# subject = "pprint.pformat"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pprint.py"
# status = "filled"
# ///
"""pprint.pformat: unbounded depth reproduces repr; depth=1 collapses past the first level to (...)/{...}/[...]; depth=2 keeps two levels then truncates, across nested tuple/dict/list"""
import pprint

nested_tuple = (1, (2, (3, (4, (5, 6)))))
nested_dict = {1: {2: {3: {4: {5: {6: 6}}}}}}
nested_list = [1, [2, [3, [4, [5, [6, []]]]]]]

# Unbounded depth: pformat == repr for nested containers.
assert pprint.pformat(nested_tuple) == repr(nested_tuple)
assert pprint.pformat(nested_dict) == repr(nested_dict)
assert pprint.pformat(nested_list) == repr(nested_list)

# depth=1 collapses everything past the first level into the ellipsis form.
assert pprint.pformat(nested_tuple, depth=1) == "(1, (...))"
assert pprint.pformat(nested_dict, depth=1) == "{1: {...}}"
assert pprint.pformat(nested_list, depth=1) == "[1, [...]]"

# depth=2 keeps two levels, then truncates.
assert pprint.pformat(nested_tuple, depth=2) == "(1, (2, (...)))"
assert pprint.pformat(nested_list, depth=2) == "[1, [2, [...]]]"
print("depth_truncation OK")
"###);
    assert_output(&out, r###"depth_truncation OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pprint/dict_key_ordering.py`.
#[test]
fn test_gen_behavior_std_libs_pprint_dict_key_ordering() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pprint"
# dimension = "behavior"
# case = "dict_key_ordering"
# subject = "pprint.pformat"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pprint.py"
# status = "filled"
# ///
"""pprint.pformat: pprint sorts dict keys by default (and inside nested lists), sort_dicts=False preserves insertion order, and heterogeneous sortable keys order deterministically by key value"""
import pprint

# Default: keys are sorted alphabetically regardless of insertion order,
# including dicts nested inside a list.
d = {"b": 1, "a": 1, "c": 1}
assert pprint.pformat(d) == "{'a': 1, 'b': 1, 'c': 1}"
assert pprint.pformat([d, d]) == \
    "[{'a': 1, 'b': 1, 'c': 1}, {'a': 1, 'b': 1, 'c': 1}]"

# sort_dicts=False preserves insertion order.
ins = dict.fromkeys("cba")
assert pprint.pformat(ins, sort_dicts=False) == \
    "{'c': None, 'b': None, 'a': None}"
assert pprint.pformat([ins, ins], sort_dicts=False) == \
    "[{'c': None, 'b': None, 'a': None}, " \
    "{'c': None, 'b': None, 'a': None}]"

# Heterogeneous sortable keys order deterministically by pprint's safe order
# (int before str before tuple).
mixed = {"xy\tab\n": (3,), 5: [[]], (): {}}
assert pprint.pformat(mixed) == "{5: [[]], 'xy\\tab\\n': (3,), (): {}}"
print("dict_key_ordering OK")
"###);
    assert_output(&out, r###"dict_key_ordering OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pprint/integer_formatting.py`.
#[test]
fn test_gen_behavior_std_libs_pprint_integer_formatting() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pprint"
# dimension = "behavior"
# case = "integer_formatting"
# subject = "pprint.pformat"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pprint.py"
# status = "filled"
# ///
"""pprint.pformat: plain ints render as decimal repr; underscore_numbers=True groups thousands (1_234_567); an int subclass is rendered via its own __repr__"""
import pprint

# Plain integers render as their decimal repr.
assert pprint.pformat(1234567) == "1234567"

# underscore_numbers=True groups digits in threes.
assert pprint.pformat(1234567, underscore_numbers=True) == "1_234_567"
assert pprint.pformat(1000, underscore_numbers=True) == "1_000"
assert pprint.pformat(999, underscore_numbers=True) == "999"


# An int subclass with a custom __repr__ is honored by pformat.
class Temperature(int):
    def __new__(cls, celsius):
        return super().__new__(cls, celsius)

    def __repr__(self):
        return f"{self + 273.15}K"


assert pprint.pformat(Temperature(1000)) == "1273.15K"
print("integer_formatting OK")
"###);
    assert_output(&out, r###"integer_formatting OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pprint/isrecursive_isreadable_predicates.py`.
#[test]
fn test_gen_behavior_std_libs_pprint_isrecursive_isreadable_predicates() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pprint"
# dimension = "behavior"
# case = "isrecursive_isreadable_predicates"
# subject = "pprint.isrecursive"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pprint.py"
# status = "filled"
# ///
"""pprint.isrecursive: isrecursive reports reference cycles and isreadable reports eval-back-ability: True for acyclic literals, False for cyclic structures and non-literal reprs (functions/modules/object()); breaking a cycle restores readability"""
import pprint

pp = pprint.PrettyPrinter()

# Acyclic, eval-able values: not recursive, and readable.
for safe in (2, 2.0, 2j, "abc", [3], (2, 2), {3: 3},
             b"def", bytearray(b"ghi"), True, False, None, ...):
    assert not pprint.isrecursive(safe)
    assert pprint.isreadable(safe)
    assert not pp.isrecursive(safe)
    assert pp.isreadable(safe)

# A self-referential list is recursive and therefore not readable.
cyclic: list = [1, 2]
cyclic.append(cyclic)
assert pprint.isrecursive(cyclic)
assert not pprint.isreadable(cyclic)

# A self-referential dict, and a tuple wrapping it, are recursive too.
d: dict = {}
d[0] = d[1] = d
for icky in (d, (d, d)):
    assert pprint.isrecursive(icky)
    assert not pprint.isreadable(icky)

# Breaking the cycle restores readability.
d.clear()
assert not pprint.isrecursive(d)
assert pprint.isreadable(d)

# Objects with non-literal reprs are readable=False but not recursive.
for unreadable in (object(), int, pprint, pprint.isrecursive):
    assert not pprint.isrecursive(unreadable)
    assert not pprint.isreadable(unreadable)
print("isrecursive_isreadable_predicates OK")
"###);
    assert_output(&out, r###"isrecursive_isreadable_predicates OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pprint/line_wrapping.py`.
#[test]
fn test_gen_behavior_std_libs_pprint_line_wrapping() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pprint"
# dimension = "behavior"
# case = "line_wrapping"
# subject = "pprint.pformat"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pprint.py"
# status = "filled"
# ///
"""pprint.pformat: pformat breaks a composite only when its single-line render exceeds width; indent sets the continuation offset, compact bounds every line to width, and long str/bytes split across adjacent literals (round-tripping via eval)"""
import pprint

# indent controls continuation offset; width decides when to break.
o = [list(range(10)), dict(first=1, second=2, third=3)]
assert pprint.pformat(o, indent=4, width=42) == (
    "[   [0, 1, 2, 3, 4, 5, 6, 7, 8, 9],\n"
    "    {'first': 1, 'second': 2, 'third': 3}]"
)
# A tighter width forces the inner dict to break, one key per line.
assert pprint.pformat(o, indent=4, width=41) == (
    "[   [0, 1, 2, 3, 4, 5, 6, 7, 8, 9],\n"
    "    {   'first': 1,\n"
    "        'second': 2,\n"
    "        'third': 3}]"
)

# compact=True keeps the line width bound: every line stays within width
# (above the minimum needed to render the deepest single, unsplittable
# nesting frame).
deep = [10] * 10
for outer in range(19):
    deep = [deep]
for w in range(42, 69):
    lines = pprint.pformat(deep, width=w, compact=True).splitlines()
    assert max(len(line) for line in lines) <= w

# Long str values are split across adjacent string literals when narrow.
fox = "the quick brown fox jumped over a lazy dog"
assert pprint.pformat(fox, width=19) == (
    "('the quick brown '\n"
    " 'fox jumped over '\n"
    " 'a lazy dog')"
)

# Long bytes split the same way; round-trips back to the original.
letters = b"abcdefghijklmnopqrstuvwxyz"
assert pprint.pformat(letters, width=19) == (
    "(b'abcdefghijkl'\n b'mnopqrstuvwxyz')"
)
for width in range(1, 40):
    assert eval(pprint.pformat(fox, width=width)) == fox
    assert eval(pprint.pformat(letters, width=width)) == letters
print("line_wrapping OK")
"###);
    assert_output(&out, r###"line_wrapping OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pprint/pformat_eval_roundtrip.py`.
#[test]
fn test_gen_behavior_std_libs_pprint_pformat_eval_roundtrip() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pprint"
# dimension = "behavior"
# case = "pformat_eval_roundtrip"
# subject = "pprint.pformat"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pprint.py"
# status = "filled"
# ///
"""pprint.pformat: pformat output is round-trippable for readable values: eval(pformat('hello'))=='hello' and eval(pformat((1,2,3)))==(1,2,3)"""
import pprint

# For readable values the rendered text is a valid literal: eval reconstructs
# the original object exactly.
assert eval(pprint.pformat("hello")) == "hello"
assert eval(pprint.pformat((1, 2, 3))) == (1, 2, 3)
print("pformat_eval_roundtrip OK")
"###);
    assert_output(&out, r###"pformat_eval_roundtrip OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pprint/pprint_returns_none.py`.
#[test]
fn test_gen_behavior_std_libs_pprint_pprint_returns_none() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pprint"
# dimension = "behavior"
# case = "pprint_returns_none"
# subject = "pprint.pprint"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pprint.py"
# status = "filled"
# ///
"""pprint.pprint: pprint() prints as a side effect and returns None (the return contract, independent of stream)"""
import contextlib
import io
import pprint

# pprint is a side-effecting printer: it writes to the stream and returns None.
buf = io.StringIO()
with contextlib.redirect_stdout(buf):
    rv = pprint.pprint(7)
assert rv is None
assert buf.getvalue() == "7\n", repr(buf.getvalue())
print("pprint_returns_none OK")
"###);
    assert_output(&out, r###"pprint_returns_none OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pprint/recursive_marker_rendered.py`.
#[test]
fn test_gen_behavior_std_libs_pprint_recursive_marker_rendered() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pprint"
# dimension = "behavior"
# case = "recursive_marker_rendered"
# subject = "pprint.pformat"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pprint.py"
# status = "filled"
# ///
"""pprint.pformat: a self-referential list is rendered with a <Recursion ...> marker instead of looping forever"""
import pprint

# A cyclic structure must terminate: pprint detects the back-reference and
# renders a <Recursion on ...> marker rather than recursing forever.
lst: list = [1, 2]
lst.append(lst)
out = pprint.pformat(lst)
assert "Recursion" in out, out
print("recursive_marker_rendered OK")
"###);
    assert_output(&out, r###"recursive_marker_rendered OK
"###);
}

/// Ported from `tests/cpython/behavior/std-libs/pprint/scalar_pformat.py`.
#[test]
fn test_gen_behavior_std_libs_pprint_scalar_pformat() {
    let out = jit_capture(r###"# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "pprint"
# dimension = "behavior"
# case = "scalar_pformat"
# subject = "pprint.pformat"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "Lib/test/test_pprint.py"
# status = "filled"
# ///
"""pprint.pformat: pformat renders scalars verbatim: 42->'42', 'hello'->"'hello'", None->'None', True->'True'"""
import pprint

# Scalars render exactly as their repr; a short scalar never wraps.
assert pprint.pformat(42) == "42"
assert pprint.pformat("hello") == "'hello'"
assert pprint.pformat(None) == "None"
assert pprint.pformat(True) == "True"
print("scalar_pformat OK")
"###);
    assert_output(&out, r###"scalar_pformat OK
"###);
}
