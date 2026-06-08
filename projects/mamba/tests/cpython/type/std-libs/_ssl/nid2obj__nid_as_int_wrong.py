# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_ssl"
# dimension = "type"
# case = "nid2obj__nid_as_int_wrong"
# subject = "_ssl.nid2obj(nid: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_ssl.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _ssl.nid2obj(nid: int); call it with the wrong type.

typeshed contract: nid is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _ssl import nid2obj
try:
    nid2obj("not_an_int")  # nid: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
