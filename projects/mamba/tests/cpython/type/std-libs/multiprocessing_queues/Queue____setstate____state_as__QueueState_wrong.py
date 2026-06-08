# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_queues"
# dimension = "type"
# case = "Queue____setstate____state_as__QueueState_wrong"
# subject = "multiprocessing.queues.Queue.__setstate__(state: _QueueState)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/queues.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.queues.Queue.__setstate__(state: _QueueState); call it with the wrong type.

typeshed contract: state is _QueueState. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from multiprocessing.queues import Queue
obj = object.__new__(Queue)
try:
    obj.__setstate__(_W())  # state: _QueueState <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
