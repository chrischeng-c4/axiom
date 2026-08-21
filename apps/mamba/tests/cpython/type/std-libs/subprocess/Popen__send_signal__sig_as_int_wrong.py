# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "subprocess"
# dimension = "type"
# case = "Popen__send_signal__sig_as_int_wrong"
# subject = "subprocess.Popen.send_signal(sig: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/subprocess.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: subprocess.Popen.send_signal(sig: int); call it with the wrong type.

typeshed contract: sig is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from subprocess import Popen
obj = object.__new__(Popen)
try:
    obj.send_signal("not_an_int")  # sig: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
