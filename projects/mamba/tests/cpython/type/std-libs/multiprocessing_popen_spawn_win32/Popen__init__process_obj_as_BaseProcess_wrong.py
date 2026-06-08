# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_popen_spawn_win32"
# dimension = "type"
# case = "Popen__init__process_obj_as_BaseProcess_wrong"
# subject = "multiprocessing.popen_spawn_win32.Popen.__init__(process_obj: BaseProcess)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/popen_spawn_win32.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.popen_spawn_win32.Popen.__init__(process_obj: BaseProcess); call it with the wrong type.

typeshed contract: process_obj is BaseProcess. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from multiprocessing.popen_spawn_win32 import Popen
try:
    Popen(_W())  # process_obj: BaseProcess <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
