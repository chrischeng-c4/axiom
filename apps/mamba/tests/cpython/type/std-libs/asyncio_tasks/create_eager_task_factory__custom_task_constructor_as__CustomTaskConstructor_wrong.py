# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_tasks"
# dimension = "type"
# case = "create_eager_task_factory__custom_task_constructor_as__CustomTaskConstructor_wrong"
# subject = "asyncio.tasks.create_eager_task_factory(custom_task_constructor: _CustomTaskConstructor)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/tasks.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.tasks.create_eager_task_factory(custom_task_constructor: _CustomTaskConstructor); call it with the wrong type.

typeshed contract: custom_task_constructor is _CustomTaskConstructor. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.tasks import create_eager_task_factory
try:
    create_eager_task_factory(_W())  # custom_task_constructor: _CustomTaskConstructor <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
