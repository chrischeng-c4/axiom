# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_tools"
# dimension = "type"
# case = "CycleFoundException__init__cycles_as_list_wrong"
# subject = "asyncio.tools.CycleFoundException.__init__(cycles: list)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed cycles"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/tools.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed cycles
# mamba-strict-type: TypeError
"""Type wall: asyncio.tools.CycleFoundException.__init__(cycles: list); call it with the wrong type.

typeshed contract: cycles is list. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from asyncio.tools import CycleFoundException
try:
    CycleFoundException(12345, None)  # cycles: list <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
