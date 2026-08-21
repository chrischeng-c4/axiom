# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "cgitb"
# dimension = "type"
# case = "text__einfo_as_OptExcInfo_wrong"
# subject = "cgitb.text(einfo: OptExcInfo)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/cgitb.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: cgitb.text(einfo: OptExcInfo); call it with the wrong type.

typeshed contract: einfo is OptExcInfo. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from cgitb import text
try:
    text(_W())  # einfo: OptExcInfo <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
