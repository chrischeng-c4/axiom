# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_base_subprocess"
# dimension = "type"
# case = "WriteSubprocessPipeProto__init__proc_as_BaseSubprocessTransport_wrong"
# subject = "asyncio.base_subprocess.WriteSubprocessPipeProto.__init__(proc: BaseSubprocessTransport)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/base_subprocess.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.base_subprocess.WriteSubprocessPipeProto.__init__(proc: BaseSubprocessTransport); call it with the wrong type.

typeshed contract: proc is BaseSubprocessTransport. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.base_subprocess import WriteSubprocessPipeProto
try:
    WriteSubprocessPipeProto(_W(), 0)  # proc: BaseSubprocessTransport <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
