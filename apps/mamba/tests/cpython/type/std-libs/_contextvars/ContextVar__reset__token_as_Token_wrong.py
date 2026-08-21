# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_contextvars"
# dimension = "type"
# case = "ContextVar__reset__token_as_Token_wrong"
# subject = "_contextvars.ContextVar.reset(token: Token)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_contextvars.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _contextvars.ContextVar.reset(token: Token); call it with the wrong type.

typeshed contract: token is Token. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _contextvars import ContextVar
obj = ContextVar("reset_token_wall")
try:
    obj.reset(_W())  # token: Token <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
