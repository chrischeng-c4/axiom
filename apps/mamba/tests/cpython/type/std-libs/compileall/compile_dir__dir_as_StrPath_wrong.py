# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "compileall"
# dimension = "type"
# case = "compile_dir__dir_as_StrPath_wrong"
# subject = "compileall.compile_dir(dir: StrPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/compileall.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: compileall.compile_dir(dir: StrPath); call it with the wrong type.

typeshed contract: dir is StrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from compileall import compile_dir
try:
    compile_dir(_W())  # dir: StrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
