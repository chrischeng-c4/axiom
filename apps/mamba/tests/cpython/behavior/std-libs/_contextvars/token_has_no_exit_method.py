# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_contextvars"
# dimension = "behavior"
# case = "token_has_no_exit_method"
# subject = "_contextvars.Token.__exit__"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_contextvars.pyi"
# status = "filled"
# ///
"""_contextvars.Token has no __exit__ method on CPython 3.12."""

from _contextvars import ContextVar

var = ContextVar("token_exit_absent")
token = var.set("value")
assert not hasattr(token, "__exit__")
try:
    token.__exit__(None, None, None)
except AttributeError:
    print("token_has_no_exit_method OK")
else:
    raise AssertionError("Token.__exit__ must be absent")
