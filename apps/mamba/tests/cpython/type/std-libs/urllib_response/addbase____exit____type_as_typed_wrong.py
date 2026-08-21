# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_response"
# dimension = "type"
# case = "addbase____exit____type_as_typed_wrong"
# subject = "urllib.response.addbase.__exit__(type: typed)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed type"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/response.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed type
# mamba-strict-type: TypeError
"""Type wall: urllib.response.addbase.__exit__(type: typed); call it with the wrong type.

typeshed contract: type is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from urllib.response import addbase
obj = object.__new__(addbase)
try:
    obj.__exit__(_W(), None, None)  # type: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
