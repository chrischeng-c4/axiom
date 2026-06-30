# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "difflib"
# dimension = "type"
# case = "ndiff__a_as_Sequence_wrong"
# subject = "difflib.ndiff(a: Sequence)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/difflib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: difflib.ndiff(a: Sequence); call it with the wrong type.

typeshed contract: a is Sequence. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from difflib import ndiff
try:
    ndiff(_W(), None)  # a: Sequence <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
