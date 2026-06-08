# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_managers"
# dimension = "type"
# case = "Server__init__registry_as_dict_wrong"
# subject = "multiprocessing.managers.Server.__init__(registry: dict)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed registry"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/managers.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed registry
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.managers.Server.__init__(registry: dict); call it with the wrong type.

typeshed contract: registry is dict. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from multiprocessing.managers import Server
try:
    Server(12345, None, b"", "")  # registry: dict <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
