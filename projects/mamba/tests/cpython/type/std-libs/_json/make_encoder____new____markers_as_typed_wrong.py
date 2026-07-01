# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_json"
# dimension = "type"
# case = "make_encoder____new____markers_as_typed_wrong"
# subject = "_json.make_encoder.__new__(markers: typed)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_json.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _json.make_encoder.__new__(markers: typed); call it with the wrong type.

typeshed contract: markers is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _json import make_encoder
obj = object.__new__(make_encoder)
try:
    obj.__new__(_W(), None, None, None, "", "", True, True, True)  # markers: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
