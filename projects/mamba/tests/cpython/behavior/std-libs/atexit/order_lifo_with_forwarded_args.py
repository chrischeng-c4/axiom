# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "atexit"
# dimension = "behavior"
# case = "order_lifo_with_forwarded_args"
# subject = "atexit._run_exitfuncs"
# kind = "semantic"
# xfail = "_run_exitfuncs() never invokes registered handlers (stub loop body is empty, #652; src/runtime/stdlib/atexit_mod.rs) so no callback fires"
# mem_carveout = ""
# source = "Lib/test/test_atexit.py"
# status = "filled"
# ///
"""atexit._run_exitfuncs: callbacks fire in LIFO (reverse-registration) order with the positional and keyword args captured at register() time"""
import atexit

calls = []


def func1(*args, **kwargs):
    calls.append(("func1", args, kwargs))


def func2(*args, **kwargs):
    calls.append(("func2", args, kwargs))


atexit._clear()
atexit.register(func1, 1, 2)
atexit.register(func2)
atexit.register(func2, 3, key="value")
atexit._run_exitfuncs()

expected = [
    ("func2", (3,), {"key": "value"}),
    ("func2", (), {}),
    ("func1", (1, 2), {}),
]
assert calls == expected, f"reverse-order args: {calls}"
atexit._clear()
print("order_lifo_with_forwarded_args OK")
