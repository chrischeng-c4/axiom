# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_interpqueues"
# dimension = "type"
# case = "put__qid_as_SupportsIndex_wrong"
# subject = "_interpqueues.put(qid: SupportsIndex)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_interpqueues.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _interpqueues.put(qid: SupportsIndex); call it with the wrong type.

typeshed contract: qid is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _interpqueues import put
try:
    put(_W(), None)  # qid: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
