# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "filecmp"
# dimension = "type"
# case = "cmp__f1_as_StrOrBytesPath_wrong"
# subject = "filecmp.cmp(f1: StrOrBytesPath)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/filecmp.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: filecmp.cmp(f1: StrOrBytesPath); call it with the wrong type.

typeshed contract: f1 is StrOrBytesPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from filecmp import cmp
try:
    cmp(_W(), None)  # f1: StrOrBytesPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
