# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "webbrowser"
# dimension = "type"
# case = "get__using_as_typed_wrong"
# subject = "webbrowser.get(using: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/webbrowser.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: webbrowser.get(using: typed); call it with the wrong type.

typeshed contract: using is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from webbrowser import get
try:
    get(_W())  # using: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
