# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_frozen_importlib_external"
# dimension = "type"
# case = "source_from_cache__path_as_StrPath_wrong"
# subject = "_frozen_importlib_external.source_from_cache(path: StrPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_frozen_importlib_external.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _frozen_importlib_external.source_from_cache(path: StrPath); call it with the wrong type.

typeshed contract: path is StrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _frozen_importlib_external import source_from_cache
try:
    source_from_cache(_W())  # path: StrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
