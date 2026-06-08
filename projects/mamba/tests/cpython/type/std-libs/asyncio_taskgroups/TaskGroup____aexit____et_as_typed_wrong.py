# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_taskgroups"
# dimension = "type"
# case = "TaskGroup____aexit____et_as_typed_wrong"
# subject = "asyncio.taskgroups.TaskGroup.__aexit__(et: typed)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed et"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/taskgroups.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed et
# mamba-strict-type: TypeError
"""Type wall: asyncio.taskgroups.TaskGroup.__aexit__(et: typed); call it with the wrong type.

typeshed contract: et is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.taskgroups import TaskGroup
obj = object.__new__(TaskGroup)
try:
    obj.__aexit__(_W(), None, None)  # et: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
