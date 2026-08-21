# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib_abc"
# dimension = "type"
# case = "InspectLoader__source_to_code__data_as_typed_wrong"
# subject = "importlib.abc.InspectLoader.source_to_code(data: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/importlib/abc.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: importlib.abc.InspectLoader.source_to_code(data: typed); call it with the wrong type.

typeshed contract: data is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from importlib.abc import InspectLoader
try:
    InspectLoader.source_to_code(_W())  # data: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
