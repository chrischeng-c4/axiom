# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "threading"
# dimension = "type"
# case = "Thread__setDaemon__daemonic_as_bool_wrong"
# subject = "threading.Thread.setDaemon(daemonic: bool)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed daemonic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/threading.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed daemonic
# mamba-strict-type: TypeError
"""Type wall: threading.Thread.setDaemon(daemonic: bool); call it with the wrong type.

typeshed contract: daemonic is bool. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from threading import Thread
obj = object.__new__(Thread)
try:
    obj.setDaemon("not_a_bool")  # daemonic: bool <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
