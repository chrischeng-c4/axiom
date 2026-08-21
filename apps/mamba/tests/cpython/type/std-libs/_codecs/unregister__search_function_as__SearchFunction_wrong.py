# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_codecs"
# dimension = "type"
# case = "unregister__search_function_as__SearchFunction_wrong"
# subject = "_codecs.unregister(search_function: _SearchFunction)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_codecs.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _codecs.unregister(search_function: _SearchFunction); call it with the wrong type.

typeshed contract: search_function is _SearchFunction. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _codecs import unregister
try:
    unregister(_W())  # search_function: _SearchFunction <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
