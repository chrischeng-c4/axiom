# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "importlib_metadata"
# dimension = "type"
# case = "PathDistribution__init__path_as__SimplePath_wrong"
# subject = "importlib.metadata.PathDistribution.__init__(path: _SimplePath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/importlib/metadata.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: importlib.metadata.PathDistribution.__init__(path: _SimplePath); call it with the wrong type.

typeshed contract: path is _SimplePath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from importlib.metadata import PathDistribution
try:
    PathDistribution(_W())  # path: _SimplePath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
