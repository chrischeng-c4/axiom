# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "audioop"
# dimension = "type"
# case = "ratecv__fragment_as_Buffer_wrong"
# subject = "audioop.ratecv(fragment: Buffer)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/audioop.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: audioop.ratecv(fragment: Buffer); call it with the wrong type.

typeshed contract: fragment is Buffer. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from audioop import ratecv
try:
    ratecv(_W(), 0, 0, 0, 0, None)  # fragment: Buffer <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
