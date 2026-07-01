# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "glob"
# dimension = "type"
# case = "glob0__dirname_as_AnyStr_wrong"
# subject = "glob.glob0(dirname: AnyStr)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/glob.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: glob.glob0(dirname: AnyStr); call it with the wrong type.

typeshed contract: dirname is AnyStr. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from glob import glob0
try:
    glob0(_W(), None)  # dirname: AnyStr <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
