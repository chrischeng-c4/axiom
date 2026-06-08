# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_asyncio"
# dimension = "type"
# case = "current_task__loop_as_typed_wrong"
# subject = "_asyncio.current_task(loop: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_asyncio.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _asyncio.current_task(loop: typed); call it with the wrong type.

typeshed contract: loop is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _asyncio import current_task
try:
    current_task(_W())  # loop: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
