# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "modulefinder"
# dimension = "type"
# case = "ModuleFinder__determine_parent__caller_as_typed_wrong"
# subject = "modulefinder.ModuleFinder.determine_parent(caller: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/modulefinder.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: modulefinder.ModuleFinder.determine_parent(caller: typed); call it with the wrong type.

typeshed contract: caller is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from modulefinder import ModuleFinder
obj = object.__new__(ModuleFinder)
try:
    obj.determine_parent(_W())  # caller: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
