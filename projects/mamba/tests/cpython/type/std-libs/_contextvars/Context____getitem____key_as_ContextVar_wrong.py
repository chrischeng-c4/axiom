# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_contextvars"
# dimension = "type"
# case = "Context____getitem____key_as_ContextVar_wrong"
# subject = "_contextvars.Context.__getitem__(key: ContextVar)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_contextvars.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _contextvars.Context.__getitem__(key: ContextVar); call it with the wrong type.

typeshed contract: key is ContextVar. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _contextvars import Context
obj = Context()
try:
    obj.__getitem__(_W())  # key: ContextVar <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
