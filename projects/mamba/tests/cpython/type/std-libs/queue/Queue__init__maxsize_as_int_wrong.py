# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "queue"
# dimension = "type"
# case = "Queue__init__maxsize_as_int_wrong"
# subject = "queue.Queue.__init__(maxsize: int)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/queue.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: queue.Queue.__init__(maxsize: int); call it with the wrong type.

typeshed contract: maxsize is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from queue import Queue
try:
    Queue("not_an_int")  # maxsize: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
