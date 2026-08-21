# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "builtin-libs"
# lib = "cpython321_builtins"
# dimension = "real_world"
# case = "test_ast_threading_builtin_iter_format_value_ops"
# subject = "cpython321.test_ast_threading_builtin_iter_format_value_ops"
# kind = "semantic"
# xfail = "CPython 3.12 seed pass; mamba promotion pending"
# mem_carveout = ""
# source = "tests/cpython/config/seeds/pass/test_ast_threading_builtin_iter_format_value_ops.py"
# status = "filled"
# ///
"""cpython321.test_ast_threading_builtin_iter_format_value_ops: execute CPython 3.12 seed test_ast_threading_builtin_iter_format_value_ops"""
# mamba-xfail: CPython 3.12 seed pass; mamba promotion pending
# Operational AssertionPass seed for the value contract of four
# bootstrap surfaces used by every parse-tree / concurrency-
# primitive / builtin-iterator / format-string path: `ast` (the
# documented `parse` / `dump` / `walk` / `literal_eval` /
# `Module` / `Expression` / `Name` / `Constant` attribute
# surface + the documented `literal_eval` scalar-/None-/bool-
# input contract), `threading` (the documented `Lock` / `RLock`
# / `Thread` / `Event` / `Condition` / `Semaphore` /
# `current_thread` / `get_ident` / `active_count` attribute
# surface + the documented `get_ident` int-return / `active_count`
# main-thread-only contract / `current_thread().name == "MainThread"`
# contract), builtin iterators (the documented `map` / `filter` /
# `zip` / `reversed` / `sorted` / `enumerate` / `sum` / `all` /
# `any` / `min` / `max` value contracts including `key=` /
# `reverse=` / `start=` / `default=` keyword arguments), and
# `str.format` (the documented positional / indexed / keyword /
# spec / float / hex / X / pct / zero-pad / format_map / comma /
# bin / oct format-spec contracts).
#
# The matching subset between mamba and CPython is the ast
# module-level attribute hasattr surface + the documented scalar
# / bool / None / str literal_eval inputs, the threading module-
# level attribute hasattr surface + the documented get_ident /
# active_count / current_thread().name MainThread contract, the
# full builtin-iterator value contract, and the full str.format
# spec contract.
#
# Surface in this fixture:
#   • ast — parse / dump / walk / literal_eval / Module /
#     Expression / Name / Constant hasattr + literal_eval scalar
#     int / float / str / True / None;
#   • threading — Lock / RLock / Thread / Event / Condition /
#     Semaphore / current_thread / get_ident / active_count
#     hasattr + get_ident returns int + active_count == 1 +
#     current_thread().name == "MainThread";
#   • builtin iterators — map / filter / zip / reversed / sorted
#     (+ key + reverse) / enumerate (+ start) / sum (+ start) /
#     all / any / min (+ key) / max (+ key) / min (+ default) /
#     max (+ default);
#   • str.format — positional / indexed / kw / right-align spec
#     / float / hex / HEX / pct / zero-pad / format_map / comma /
#     bin / oct.
#
# Behavioral edges that DIVERGE on mamba (ast.parse(...).body
# returns empty list, ast.literal_eval on arithmetic /
# list / dict / tuple inputs returns None, threading.Lock /
# RLock / Event instance method surface broken — acquire / set
# AttributeError) are covered in the matching spec fixture
# `lang_ast_threading_silent`.
import ast
import threading


_ledger: list[int] = []

# 1) ast — module attribute hasattr surface
assert hasattr(ast, "parse") == True; _ledger.append(1)
assert hasattr(ast, "dump") == True; _ledger.append(1)
assert hasattr(ast, "walk") == True; _ledger.append(1)
assert hasattr(ast, "literal_eval") == True; _ledger.append(1)
assert hasattr(ast, "Module") == True; _ledger.append(1)
assert hasattr(ast, "Expression") == True; _ledger.append(1)
assert hasattr(ast, "Name") == True; _ledger.append(1)
assert hasattr(ast, "Constant") == True; _ledger.append(1)

# 2) ast.literal_eval — scalar / bool / None / str contract
assert ast.literal_eval("42") == 42; _ledger.append(1)
assert ast.literal_eval("3.14") == 3.14; _ledger.append(1)
assert ast.literal_eval("'hello'") == "hello"; _ledger.append(1)
assert ast.literal_eval("True") == True; _ledger.append(1)
assert ast.literal_eval("False") == False; _ledger.append(1)
assert ast.literal_eval("None") == None; _ledger.append(1)

# 3) threading — module attribute hasattr surface
assert hasattr(threading, "Lock") == True; _ledger.append(1)
assert hasattr(threading, "RLock") == True; _ledger.append(1)
assert hasattr(threading, "Thread") == True; _ledger.append(1)
assert hasattr(threading, "Event") == True; _ledger.append(1)
assert hasattr(threading, "Condition") == True; _ledger.append(1)
assert hasattr(threading, "Semaphore") == True; _ledger.append(1)
assert hasattr(threading, "current_thread") == True; _ledger.append(1)
assert hasattr(threading, "get_ident") == True; _ledger.append(1)
assert hasattr(threading, "active_count") == True; _ledger.append(1)

# 4) threading — value contracts on main-thread accessors
assert isinstance(threading.get_ident(), int); _ledger.append(1)
assert threading.active_count() == 1; _ledger.append(1)
assert threading.current_thread().name == "MainThread"; _ledger.append(1)

# 5) builtin iterators — map / filter / zip / reversed
assert sum(map(lambda x: x * 2, [1, 2, 3])) == 12; _ledger.append(1)
assert sum(filter(lambda x: x % 2 == 0, [1, 2, 3, 4, 5])) == 6; _ledger.append(1)
assert list(zip([1, 2, 3], ["a", "b", "c"])) == [(1, "a"), (2, "b"), (3, "c")]; _ledger.append(1)
assert list(reversed([1, 2, 3])) == [3, 2, 1]; _ledger.append(1)

# 6) builtin iterators — sorted + key / reverse
assert sorted([3, 1, 2]) == [1, 2, 3]; _ledger.append(1)
assert sorted([3, 1, 2], reverse=True) == [3, 2, 1]; _ledger.append(1)
assert sorted(["bb", "a", "ccc"], key=len) == ["a", "bb", "ccc"]; _ledger.append(1)

# 7) builtin iterators — enumerate + start, sum + start
assert list(enumerate(["a", "b"], start=10)) == [(10, "a"), (11, "b")]; _ledger.append(1)
assert sum([1, 2, 3], 100) == 106; _ledger.append(1)

# 8) builtin iterators — all / any
assert all([True, True, True]) == True; _ledger.append(1)
assert all([True, False, True]) == False; _ledger.append(1)
assert any([False, False, True]) == True; _ledger.append(1)
assert any([False, False, False]) == False; _ledger.append(1)

# 9) builtin iterators — min / max + key + default
assert min(["bb", "a", "ccc"], key=len) == "a"; _ledger.append(1)
assert max(["bb", "a", "ccc"], key=len) == "ccc"; _ledger.append(1)
assert min([], default=99) == 99; _ledger.append(1)
assert max([], default=-1) == -1; _ledger.append(1)

# 10) str.format — positional / indexed / keyword
assert "{} {}".format("hi", "there") == "hi there"; _ledger.append(1)
assert "{1} {0}".format("a", "b") == "b a"; _ledger.append(1)
assert "{x} {y}".format(x=1, y=2) == "1 2"; _ledger.append(1)

# 11) str.format — spec / float / hex / HEX / pct
assert "{:>5}".format("ab") == "   ab"; _ledger.append(1)
assert "{:.3f}".format(3.14159) == "3.142"; _ledger.append(1)
assert "{:x}".format(255) == "ff"; _ledger.append(1)
assert "{:X}".format(255) == "FF"; _ledger.append(1)
assert "{:.1%}".format(0.5) == "50.0%"; _ledger.append(1)

# 12) str.format — zero-pad / format_map / comma / bin / oct
assert "{:05d}".format(42) == "00042"; _ledger.append(1)
assert "{x}".format_map({"x": "hello"}) == "hello"; _ledger.append(1)
assert "{:,}".format(1234567) == "1,234,567"; _ledger.append(1)
assert "{:b}".format(10) == "1010"; _ledger.append(1)
assert "{:o}".format(8) == "10"; _ledger.append(1)

# NB: ast.parse(...).body returns the empty list, ast.literal_eval
# on arithmetic / list / dict / tuple input returns None, threading
# Lock / RLock / Event instance method surface broken (acquire /
# set AttributeError) — all DIVERGE on mamba — moved to the
# divergence-spec fixture.

print(f"MAMBA_ASSERTION_PASS: test_ast_threading_builtin_iter_format_value_ops {sum(_ledger)} asserts")
