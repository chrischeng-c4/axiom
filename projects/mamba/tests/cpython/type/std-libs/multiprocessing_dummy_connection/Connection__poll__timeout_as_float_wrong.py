# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "multiprocessing_dummy_connection"
# dimension = "type"
# case = "Connection__poll__timeout_as_float_wrong"
# subject = "multiprocessing.dummy.connection.Connection.poll(timeout: float)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/multiprocessing/dummy/connection.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: multiprocessing.dummy.connection.Connection.poll(timeout: float); call it with the wrong type.

typeshed contract: timeout is float. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from multiprocessing.dummy.connection import Connection
obj = object.__new__(Connection)
try:
    obj.poll("not_a_float")  # timeout: float <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
