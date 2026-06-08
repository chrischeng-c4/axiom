# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "glob"
# dimension = "type"
# case = "glob__pathname_as_AnyStr_wrong"
# subject = "glob.glob(pathname: AnyStr)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed pathname"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/glob.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed pathname
# mamba-strict-type: TypeError
"""Type wall: glob.glob(pathname: AnyStr); call it with the wrong type.

typeshed contract: pathname is AnyStr. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from glob import glob
try:
    glob(_W())  # pathname: AnyStr <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
