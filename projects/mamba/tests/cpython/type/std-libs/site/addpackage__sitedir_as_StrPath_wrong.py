# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "site"
# dimension = "type"
# case = "addpackage__sitedir_as_StrPath_wrong"
# subject = "site.addpackage(sitedir: StrPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/site.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: site.addpackage(sitedir: StrPath); call it with the wrong type.

typeshed contract: sitedir is StrPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from site import addpackage
try:
    addpackage(_W(), None, None)  # sitedir: StrPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
