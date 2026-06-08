# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "json"
# dimension = "security"
# case = "recursion_bomb_raises_not_crash"
# subject = "json.JSONDecoder"
# kind = "semantic"
# xfail = "deep recursion aborts the process (fatal runtime error: stack overflow) instead of raising RecursionError; mamba enforces no recursion limit, and json.JSONDecoder is an integer-handle stub with no scan_once / json.scanner submodule (src/runtime/stdlib/json_mod.rs:281,409)"
# mem_carveout = ""
# source = "Lib/test/json/test_recursion.py"
# status = "filled"
# ///
"""json.JSONDecoder: a deeply nested JSON payload (['*N + ]'*N) drives the pure-Python scanner past sys.getrecursionlimit() and must raise a catchable RecursionError rather than crashing the interpreter"""
import json
import sys
from json import scanner

limit = sys.getrecursionlimit()

# Force the pure-Python recursive scanner -- the code path that respects
# sys.getrecursionlimit(). (The default C accelerator is iterative for flat
# nesting and would not exercise the recursion guard.)
decoder = json.JSONDecoder()
decoder.scan_once = scanner.py_make_scanner(decoder)

# Depth >= the recursion limit guarantees stack exhaustion: every "[" consumes
# a scanner frame. Pinned to the live limit => deterministic everywhere.
attack_depth = limit
bomb = "[" * attack_depth + "]" * attack_depth
assert len(bomb) == attack_depth * 2

# Hostile input must raise RecursionError -- a normal, catchable exception --
# and crucially must NOT crash the interpreter (segfault / abort).
raised = None
try:
    decoder.decode(bomb)
except RecursionError as e:
    raised = e
assert raised is not None, "deep-nested JSON must raise RecursionError"
assert isinstance(raised, RuntimeError)  # RecursionError subclasses RuntimeError

# The interpreter survived the attack and keeps running normally.
# A moderate depth (comfortably under the limit) parses fine and round-trips.
safe_depth = 20
assert safe_depth < limit
safe = "[" * safe_depth + "]" * safe_depth
parsed = decoder.decode(safe)
assert json.loads(safe) == parsed  # C accelerator agrees on moderate input
assert json.dumps(parsed, separators=(",", ":")) == safe

# Object nesting is the same attack surface; confirm it also raises cleanly.
obj_bomb = '{"a":' * limit + "1" + "}" * limit
obj_raised = False
try:
    decoder.decode(obj_bomb)
except RecursionError:
    obj_raised = True
assert obj_raised, "deep-nested JSON object must raise RecursionError"

print("recursion_bomb_raises_not_crash OK")
