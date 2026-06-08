# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email__policybase"
# dimension = "type"
# case = "Compat32__fold__name_as_str_wrong"
# subject = "email._policybase.Compat32.fold(name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/_policybase.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email._policybase.Compat32.fold(name: str); call it with the wrong type.

typeshed contract: name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from email._policybase import Compat32
obj = object.__new__(Compat32)
try:
    obj.fold(12345, "")  # name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
