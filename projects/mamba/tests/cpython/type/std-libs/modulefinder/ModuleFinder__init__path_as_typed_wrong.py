# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "modulefinder"
# dimension = "type"
# case = "ModuleFinder__init__path_as_typed_wrong"
# subject = "modulefinder.ModuleFinder.__init__(path: typed)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed path"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/modulefinder.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed path
# mamba-strict-type: TypeError
"""Type wall: modulefinder.ModuleFinder.__init__(path: typed); call it with the wrong type.

typeshed contract: path is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from modulefinder import ModuleFinder
try:
    ModuleFinder(_W())  # path: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
