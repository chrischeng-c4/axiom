# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_zstd"
# dimension = "type"
# case = "set_parameter_types__c_parameter_type_as_type_wrong"
# subject = "_zstd.set_parameter_types(c_parameter_type: type)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed c_parameter_type"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_zstd.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed c_parameter_type
# mamba-strict-type: TypeError
"""Type wall: _zstd.set_parameter_types(c_parameter_type: type); call it with the wrong type.

typeshed contract: c_parameter_type is type. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _zstd import set_parameter_types
try:
    set_parameter_types(_W(), None)  # c_parameter_type: type <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
