# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_forkserver"
# dimension = "type"
# case = "ForkServer__connect_to_new_process__fds_as_Sequence_wrong"
# subject = "multiprocessing.forkserver.ForkServer.connect_to_new_process(fds: Sequence)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/forkserver.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.forkserver.ForkServer.connect_to_new_process(fds: Sequence); call it with the wrong type.

typeshed contract: fds is Sequence. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from multiprocessing.forkserver import ForkServer
obj = object.__new__(ForkServer)
try:
    obj.connect_to_new_process(_W())  # fds: Sequence <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
