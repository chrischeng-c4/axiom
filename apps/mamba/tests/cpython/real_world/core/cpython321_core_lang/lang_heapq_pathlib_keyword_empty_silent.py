# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "core"
# lib = "cpython321_core_lang"
# dimension = "real_world"
# case = "lang_heapq_pathlib_keyword_empty_silent"
# subject = "cpython321.lang_heapq_pathlib_keyword_empty_silent"
# kind = "semantic"
# xfail = "CPython 3.12 seed spec; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/spec/lang_heapq_pathlib_keyword_empty_silent.py"
# status = "filled"
# ///
"""cpython321.lang_heapq_pathlib_keyword_empty_silent: execute CPython 3.12 seed lang_heapq_pathlib_keyword_empty_silent"""
# mamba-xfail: CPython 3.12 seed spec; mamba promotion pending
# Operational AssertionPass seed for SILENT divergences in `heapq`
# (string heap order, merge(reverse=) kwarg, heappop on an empty heap)
# and `pathlib` (every Path attribute / arithmetic operation). The
# matching subset (scalar int/float min-heap + keyword-free merge /
# nsmallest / nlargest / heapreplace / heappushpop) is covered by
# `test_heapq_min_heap_merge_ops`; this fixture pins the CPython-only
# contracts that mamba currently elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • heapq.heapify on a string list — should produce a min-heap in
#     lex order (mamba: heapify is a no-op on str — root remains the
#     first input element);
#   • heapq.merge(..., reverse=True) — should yield descending merge
#     (mamba: silently ignores reverse, still ascending);
#   • heapq.heappop([]) — should raise IndexError (mamba: returns None);
#   • pathlib.Path.name / .stem / .suffix / .parts / .parent — should
#     return non-empty str values (mamba: returns None or 'None');
#   • pathlib.Path / part — should produce a Path with the parts joined
#     (mamba: returns 'None');
#   • pathlib.Path.with_suffix / .with_name / .is_absolute — should be
#     instance methods (mamba: raises AttributeError).
import heapq
from pathlib import Path
from typing import Any

_ledger: list[int] = []

# 1) heapq.heapify on a string list — root is the lex-smallest
_sh: list[str] = ["delta", "alpha", "charlie", "bravo"]
heapq.heapify(_sh)
assert _sh[0] == "alpha"; _ledger.append(1)
_sh_out: list[Any] = []
while _sh:
    _sh_out.append(heapq.heappop(_sh))
assert _sh_out == ["alpha", "bravo", "charlie", "delta"]; _ledger.append(1)

# 2) heapq.merge with reverse=True — descending k-way merge
_merged: list[Any] = list(heapq.merge([5, 3, 1], [6, 4, 2], reverse=True))
assert _merged == [6, 5, 4, 3, 2, 1]; _ledger.append(1)
_merged2: list[Any] = list(heapq.merge([9, 5, 1], [8, 4, 0], reverse=True))
assert _merged2 == [9, 8, 5, 4, 1, 0]; _ledger.append(1)

# 3) heapq.heappop on an empty heap — IndexError, not silent None
try:
    _popped: Any = heapq.heappop([])
    _empty_pop_result = "no-raise"
except IndexError:
    _empty_pop_result = "indexerror"
except Exception:
    _empty_pop_result = "other"
assert _empty_pop_result == "indexerror"; _ledger.append(1)

# 4) pathlib.Path attribute accessors — name / stem / suffix
_p: Any = Path("/foo/bar/baz.txt")
assert _p.name == "baz.txt"; _ledger.append(1)
assert _p.stem == "baz"; _ledger.append(1)
assert _p.suffix == ".txt"; _ledger.append(1)
assert isinstance(_p.name, str); _ledger.append(1)

# 5) pathlib.Path.parts — tuple of the path components
_parts: Any = _p.parts
assert _parts == ("/", "foo", "bar", "baz.txt"); _ledger.append(1)
assert isinstance(_parts, tuple); _ledger.append(1)

# 6) pathlib.Path.parent — the containing directory
_parent: Any = _p.parent
assert str(_parent) == "/foo/bar"; _ledger.append(1)

# 7) pathlib.Path / part — joins one or more parts
_joined: Any = Path("/foo") / "bar" / "baz"
assert str(_joined) == "/foo/bar/baz"; _ledger.append(1)
_joined2: Any = Path("/foo") / "bar"
assert str(_joined2) == "/foo/bar"; _ledger.append(1)

# 8) pathlib.Path.with_suffix — replace the final suffix
_p2: Any = Path("/x/y.txt")
_w: Any = _p2.with_suffix(".log")
assert str(_w) == "/x/y.log"; _ledger.append(1)
_w2: Any = _p2.with_suffix("")
assert str(_w2) == "/x/y"; _ledger.append(1)

# 9) pathlib.Path.with_name — replace the final component
_w_name: Any = _p2.with_name("z.log")
assert str(_w_name) == "/x/z.log"; _ledger.append(1)

# 10) pathlib.Path.is_absolute — distinguishes absolute vs relative
assert Path("/a").is_absolute() == True; _ledger.append(1)
assert Path("a").is_absolute() == False; _ledger.append(1)
assert Path("/foo/bar").is_absolute() == True; _ledger.append(1)
assert Path("foo/bar").is_absolute() == False; _ledger.append(1)

# 11) pathlib.Path equality — same string path compares equal
assert Path("/a/b") == Path("/a/b"); _ledger.append(1)
assert Path("/a/b") != Path("/a/c"); _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_heapq_pathlib_keyword_empty_silent {sum(_ledger)} asserts")
