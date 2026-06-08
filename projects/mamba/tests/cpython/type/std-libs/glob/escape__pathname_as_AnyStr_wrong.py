# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "glob"
# dimension = "type"
# case = "escape__pathname_as_AnyStr_wrong"
# subject = "glob.escape(pathname: AnyStr)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/glob.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: glob.escape(pathname: AnyStr); call it with the wrong type.

typeshed contract: pathname is AnyStr. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from glob import escape
try:
    escape(_W())  # pathname: AnyStr <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
