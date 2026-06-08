# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "asyncio_events"
# dimension = "type"
# case = "set_child_watcher__watcher_as_AbstractChildWatcher_wrong"
# subject = "asyncio.events.set_child_watcher(watcher: AbstractChildWatcher)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/asyncio/events.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: asyncio.events.set_child_watcher(watcher: AbstractChildWatcher); call it with the wrong type.

typeshed contract: watcher is AbstractChildWatcher. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from asyncio.events import set_child_watcher
try:
    set_child_watcher(_W())  # watcher: AbstractChildWatcher <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
