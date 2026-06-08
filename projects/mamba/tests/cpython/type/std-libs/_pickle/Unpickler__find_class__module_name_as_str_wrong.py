# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_pickle"
# dimension = "type"
# case = "Unpickler__find_class__module_name_as_str_wrong"
# subject = "_pickle.Unpickler.find_class(module_name: str)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_pickle.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _pickle.Unpickler.find_class(module_name: str); call it with the wrong type.

typeshed contract: module_name is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _pickle import Unpickler
obj = object.__new__(Unpickler)
try:
    obj.find_class(12345, "")  # module_name: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
