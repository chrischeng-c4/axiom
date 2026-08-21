# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_subprocess"
# dimension = "type"
# case = "SubprocessStreamProtocol__init__limit_as_int_wrong"
# subject = "asyncio.subprocess.SubprocessStreamProtocol.__init__(limit: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/subprocess.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.subprocess.SubprocessStreamProtocol.__init__(limit: int); call it with the wrong type.

typeshed contract: limit is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from asyncio.subprocess import SubprocessStreamProtocol
try:
    SubprocessStreamProtocol("not_an_int", None)  # limit: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
