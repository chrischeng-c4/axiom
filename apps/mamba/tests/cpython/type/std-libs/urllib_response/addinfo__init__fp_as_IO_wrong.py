# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "urllib_response"
# dimension = "type"
# case = "addinfo__init__fp_as_IO_wrong"
# subject = "urllib.response.addinfo.__init__(fp: IO)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/urllib/response.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: urllib.response.addinfo.__init__(fp: IO); call it with the wrong type.

typeshed contract: fp is IO. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from urllib.response import addinfo
try:
    addinfo(_W(), None)  # fp: IO <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
