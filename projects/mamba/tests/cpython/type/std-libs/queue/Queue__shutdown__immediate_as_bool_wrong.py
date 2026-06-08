# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "queue"
# dimension = "type"
# case = "Queue__shutdown__immediate_as_bool_wrong"
# subject = "queue.Queue.shutdown(immediate: bool)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed immediate"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/queue.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed immediate
# mamba-strict-type: TypeError
"""Type wall: queue.Queue.shutdown(immediate: bool); call it with the wrong type.

typeshed contract: immediate is bool. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from queue import Queue
obj = object.__new__(Queue)
try:
    obj.shutdown("not_a_bool")  # immediate: bool <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
