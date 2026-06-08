# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_tools"
# dimension = "type"
# case = "display_awaited_by_tasks_tree__pid_as_SupportsIndex_wrong"
# subject = "asyncio.tools.display_awaited_by_tasks_tree(pid: SupportsIndex)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/tools.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.tools.display_awaited_by_tasks_tree(pid: SupportsIndex); call it with the wrong type.

typeshed contract: pid is SupportsIndex. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.tools import display_awaited_by_tasks_tree
try:
    display_awaited_by_tasks_tree(_W())  # pid: SupportsIndex <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
