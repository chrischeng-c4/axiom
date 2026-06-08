# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_subprocess"
# dimension = "type"
# case = "create_subprocess_exec__program_as_StrOrBytesPath_wrong"
# subject = "asyncio.subprocess.create_subprocess_exec(program: StrOrBytesPath)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed program"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/subprocess.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed program
# mamba-strict-type: TypeError
"""Type wall: asyncio.subprocess.create_subprocess_exec(program: StrOrBytesPath); call it with the wrong type.

typeshed contract: program is StrOrBytesPath. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.subprocess import create_subprocess_exec
try:
    create_subprocess_exec(_W())  # program: StrOrBytesPath <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
