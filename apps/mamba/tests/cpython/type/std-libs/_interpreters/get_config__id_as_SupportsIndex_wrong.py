# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_interpreters"
# dimension = "type"
# case = "get_config__id_as_SupportsIndex_wrong"
# subject = "_interpreters.get_config(id: SupportsIndex)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_interpreters.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _interpreters.get_config(id: SupportsIndex); call it with the wrong type.

typeshed contract: id is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _interpreters import get_config
try:
    get_config(_W())  # id: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
