# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "ssl"
# dimension = "type"
# case = "SSLObject__getpeercert__binary_form_as_bool_wrong"
# subject = "ssl.SSLObject.getpeercert(binary_form: bool)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed binary_form"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/ssl.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed binary_form
# mamba-strict-type: TypeError
"""Type wall: ssl.SSLObject.getpeercert(binary_form: bool); call it with the wrong type.

typeshed contract: binary_form is bool. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from ssl import SSLObject
obj = object.__new__(SSLObject)
try:
    obj.getpeercert("not_a_bool")  # binary_form: bool <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
