# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_base_subprocess"
# dimension = "type"
# case = "BaseSubprocessTransport__send_signal__signal_as_int_wrong"
# subject = "asyncio.base_subprocess.BaseSubprocessTransport.send_signal(signal: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/base_subprocess.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.base_subprocess.BaseSubprocessTransport.send_signal(signal: int); call it with the wrong type.

typeshed contract: signal is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from asyncio.base_subprocess import BaseSubprocessTransport
obj = object.__new__(BaseSubprocessTransport)
try:
    obj.send_signal("not_an_int")  # signal: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
