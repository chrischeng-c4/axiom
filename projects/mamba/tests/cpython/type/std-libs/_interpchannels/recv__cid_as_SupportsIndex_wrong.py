# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_interpchannels"
# dimension = "type"
# case = "recv__cid_as_SupportsIndex_wrong"
# subject = "_interpchannels.recv(cid: SupportsIndex)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_interpchannels.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _interpchannels.recv(cid: SupportsIndex); call it with the wrong type.

typeshed contract: cid is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _interpchannels import recv
try:
    recv(_W())  # cid: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
