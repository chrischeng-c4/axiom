# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sunau"
# dimension = "type"
# case = "Au_write__setparams__params_as__sunau_params_wrong"
# subject = "sunau.Au_write.setparams(params: _sunau_params)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/sunau.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: sunau.Au_write.setparams(params: _sunau_params); call it with the wrong type.

typeshed contract: params is _sunau_params. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from sunau import Au_write
obj = object.__new__(Au_write)
try:
    obj.setparams(_W())  # params: _sunau_params <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
