# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_subprocess"
# dimension = "type"
# case = "create_subprocess_shell__cmd_as_typed_wrong"
# subject = "asyncio.subprocess.create_subprocess_shell(cmd: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/subprocess.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.subprocess.create_subprocess_shell(cmd: typed); call it with the wrong type.

typeshed contract: cmd is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.subprocess import create_subprocess_shell
try:
    create_subprocess_shell(_W())  # cmd: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
