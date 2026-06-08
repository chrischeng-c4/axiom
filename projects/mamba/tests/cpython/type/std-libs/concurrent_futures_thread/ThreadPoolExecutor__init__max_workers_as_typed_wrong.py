# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "concurrent_futures_thread"
# dimension = "type"
# case = "ThreadPoolExecutor__init__max_workers_as_typed_wrong"
# subject = "concurrent.futures.thread.ThreadPoolExecutor.__init__(max_workers: typed)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed max_workers"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/concurrent/futures/thread.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed max_workers
# mamba-strict-type: TypeError
"""Type wall: concurrent.futures.thread.ThreadPoolExecutor.__init__(max_workers: typed); call it with the wrong type.

typeshed contract: max_workers is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from concurrent.futures.thread import ThreadPoolExecutor
try:
    ThreadPoolExecutor(_W())  # max_workers: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
