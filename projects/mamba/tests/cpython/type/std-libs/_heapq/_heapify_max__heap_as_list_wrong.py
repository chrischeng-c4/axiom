# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_heapq"
# dimension = "type"
# case = "_heapify_max__heap_as_list_wrong"
# subject = "_heapq._heapify_max(heap: list)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "python3.12:_heapq"
# status = "filled"
# ///
# mamba-strict-type: TypeError

from _heapq import _heapify_max

try:
    _heapify_max(12345)  # heap: list <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
