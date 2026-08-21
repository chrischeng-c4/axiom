# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "atexit"
# dimension = "errors"
# case = "register_non_callable_raises_typeerror"
# subject = "atexit.register"
# kind = "mechanical"
# xfail = "atexit.register() does not validate callability — accepts a non-callable silently instead of raising TypeError (stub, #652; src/runtime/stdlib/atexit_mod.rs)"
# mem_carveout = ""
# source = "Lib/test/test_atexit.py"
# status = "filled"
# ///
"""atexit.register: register_non_callable_raises_typeerror (errors)."""
import atexit

_raised = False
try:
    atexit.register(42)
except TypeError:
    _raised = True
assert _raised, "register_non_callable_raises_typeerror: expected TypeError"
print("register_non_callable_raises_typeerror OK")
