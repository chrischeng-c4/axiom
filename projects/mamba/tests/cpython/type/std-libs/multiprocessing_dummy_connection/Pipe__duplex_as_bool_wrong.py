# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_dummy_connection"
# dimension = "type"
# case = "Pipe__duplex_as_bool_wrong"
# subject = "multiprocessing.dummy.connection.Pipe(duplex: bool)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed duplex"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/dummy/connection.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed duplex
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.dummy.connection.Pipe(duplex: bool); call it with the wrong type.

typeshed contract: duplex is bool. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from multiprocessing.dummy.connection import Pipe
try:
    Pipe("not_a_bool")  # duplex: bool <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
