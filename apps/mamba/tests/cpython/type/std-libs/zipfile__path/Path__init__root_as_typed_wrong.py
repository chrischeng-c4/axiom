# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zipfile__path"
# dimension = "type"
# case = "Path__init__root_as_typed_wrong"
# subject = "zipfile._path.Path.__init__(root: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zipfile/_path.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zipfile._path.Path.__init__(root: typed); call it with the wrong type.

typeshed contract: root is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from zipfile._path import Path
try:
    Path(_W())  # root: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
