# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_decimal"
# dimension = "type"
# case = "setcontext__context_as_Context_wrong"
# subject = "_decimal.setcontext(context: Context)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_decimal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _decimal.setcontext(context: Context); call it with the wrong type.

typeshed contract: context is Context. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _decimal import setcontext
try:
    setcontext(_W())  # context: Context <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
