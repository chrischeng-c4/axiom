# Operational AssertionPass seed for SILENT divergences across the
# parse-tree / concurrency-primitive pair pinned by atomic 168:
# `ast` (the documented `parse(source).body` non-empty-statement-
# list contract + the documented `literal_eval` on container /
# tuple inputs) and `threading` (the documented `Lock` / `RLock`
# instance `acquire` / `locked` method surface + the documented
# `Event` instance `set` / `is_set` method surface).
#
# The matching subset (ast module attribute hasattr surface +
# literal_eval on scalar / bool / None / str inputs, threading
# module attribute hasattr surface + get_ident / active_count /
# current_thread().name MainThread contract, full builtin-iterator
# value contract, full str.format spec contract) is covered by
# `test_ast_threading_builtin_iter_format_value_ops`; this
# fixture pins the CPython-only contracts that mamba currently
# elides.
#
# Surface (CPython AssertionPass; mamba diverges silently):
#   • len(ast.parse("x = 1 + 2").body) == 1 — documented parse-
#     tree statement list (mamba: returns 0 — body list is
#     empty even for a syntactically valid assignment);
#   • ast.literal_eval("[1, 2, 3]") == [1, 2, 3] — documented
#     list literal-eval (mamba: returns None);
#   • ast.literal_eval('{"a": 1, "b": 2}') == {"a": 1, "b": 2}
#     — documented dict literal-eval (mamba: returns None);
#   • ast.literal_eval("(1, 2, 3)") == (1, 2, 3) — documented
#     tuple literal-eval (mamba: returns None);
#   • ast.literal_eval("[1, [2, 3]]") == [1, [2, 3]] —
#     documented nested-container literal-eval (mamba: returns
#     None);
#   • threading.Lock().acquire() == True — documented Lock
#     instance acquire contract (mamba: AttributeError, 'Lock'
#     object has no attribute 'acquire');
#   • threading.Lock().locked() after acquire is True (mamba:
#     AttributeError at acquire — can't reach locked contract);
#   • threading.RLock().acquire() == True — documented RLock
#     instance acquire contract (mamba: same AttributeError);
#   • threading.Event().is_set() after set is True — documented
#     Event instance set / is_set contract (mamba: AttributeError,
#     'Event' object has no attribute 'set').
import ast as _ast_mod
import threading as _threading_mod
from typing import Any

# Module bindings retyped as `Any` to bypass Pyright stub-driven
# narrowing — every spec contract below probes documented public
# class identifiers / module-level helpers / instance methods
# that mamba's bundled type stubs do not surface accurately.
ast: Any = _ast_mod
threading: Any = _threading_mod


_ledger: list[int] = []

# 1) ast.parse — body returns a non-empty statement list
assert len(ast.parse("x = 1 + 2").body) == 1; _ledger.append(1)
assert len(ast.parse("a = 1\nb = 2").body) == 2; _ledger.append(1)

# 2) ast.literal_eval — container value contracts
assert ast.literal_eval("[1, 2, 3]") == [1, 2, 3]; _ledger.append(1)
assert ast.literal_eval('{"a": 1, "b": 2}') == {"a": 1, "b": 2}; _ledger.append(1)
assert ast.literal_eval("(1, 2, 3)") == (1, 2, 3); _ledger.append(1)
assert ast.literal_eval("[1, [2, 3]]") == [1, [2, 3]]; _ledger.append(1)

# 3) threading.Lock — acquire + locked instance contract
_lk = threading.Lock()
assert _lk.acquire() == True; _ledger.append(1)
assert _lk.locked() == True; _ledger.append(1)

# 4) threading.RLock — acquire instance contract
_rl = threading.RLock()
assert _rl.acquire() == True; _ledger.append(1)

# 5) threading.Event — set + is_set instance contract
_ev = threading.Event()
_ev.set()
assert _ev.is_set() == True; _ledger.append(1)
_ev.clear()
assert _ev.is_set() == False; _ledger.append(1)

print(f"MAMBA_ASSERTION_PASS: lang_ast_threading_silent {sum(_ledger)} asserts")
