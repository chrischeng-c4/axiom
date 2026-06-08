# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_forkserver"
# dimension = "type"
# case = "ForkServer__set_forkserver_preload__modules_names_as_list_wrong"
# subject = "multiprocessing.forkserver.ForkServer.set_forkserver_preload(modules_names: list)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed modules_names"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/forkserver.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed modules_names
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.forkserver.ForkServer.set_forkserver_preload(modules_names: list); call it with the wrong type.

typeshed contract: modules_names is list. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from multiprocessing.forkserver import ForkServer
obj = object.__new__(ForkServer)
try:
    obj.set_forkserver_preload(12345)  # modules_names: list <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
