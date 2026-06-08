# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "mailcap"
# dimension = "type"
# case = "findmatch__caps_as_Mapping_wrong"
# subject = "mailcap.findmatch(caps: Mapping)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed caps"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/mailcap.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed caps
# mamba-strict-type: TypeError
"""Type wall: mailcap.findmatch(caps: Mapping); call it with the wrong type.

typeshed contract: caps is Mapping. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from mailcap import findmatch
try:
    findmatch(_W(), "")  # caps: Mapping <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
