# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_contextvars"
# dimension = "behavior"
# case = "contextvar_set_accepts_object_value"
# subject = "_contextvars.ContextVar.set(value)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_contextvars.pyi"
# status = "filled"
# ///
"""_contextvars.ContextVar.set(value): TypeVar values accept arbitrary objects."""

from _contextvars import ContextVar, Token


class Value:
    pass


value = Value()
var = ContextVar("value_object")
token = var.set(value)
assert isinstance(token, Token)
assert var.get() is value
print("contextvar_set_accepts_object_value OK")
