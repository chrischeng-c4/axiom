# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "binhex"
# dimension = "type"
# case = "BinHex__init__name_finfo_dlen_rlen_as__FileInfoTuple_wrong"
# subject = "binhex.BinHex.__init__(name_finfo_dlen_rlen: _FileInfoTuple)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/binhex.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: binhex.BinHex.__init__(name_finfo_dlen_rlen: _FileInfoTuple); call it with the wrong type.

typeshed contract: name_finfo_dlen_rlen is _FileInfoTuple. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from binhex import BinHex
try:
    BinHex(_W(), None)  # name_finfo_dlen_rlen: _FileInfoTuple <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
