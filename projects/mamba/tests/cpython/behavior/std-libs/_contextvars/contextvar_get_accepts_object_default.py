# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_contextvars"
# dimension = "behavior"
# case = "contextvar_get_accepts_object_default"
# subject = "_contextvars.ContextVar.get(default)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_contextvars.pyi"
# status = "filled"
# ///
"""_contextvars.ContextVar.get(default): TypeVar defaults accept arbitrary objects."""

from _contextvars import ContextVar


class Default:
    pass


default = Default()
var = ContextVar("default_object")
assert var.get(default) is default
print("contextvar_get_accepts_object_default OK")
