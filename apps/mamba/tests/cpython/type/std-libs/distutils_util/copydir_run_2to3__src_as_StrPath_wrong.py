# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "distutils_util"
# dimension = "type"
# case = "copydir_run_2to3__src_as_StrPath_wrong"
# subject = "distutils.util.copydir_run_2to3(src: StrPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/distutils/util.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: distutils.util.copydir_run_2to3(src: StrPath); call it with the wrong type.

typeshed contract: src is StrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from distutils.util import copydir_run_2to3
try:
    copydir_run_2to3(_W(), None)  # src: StrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
