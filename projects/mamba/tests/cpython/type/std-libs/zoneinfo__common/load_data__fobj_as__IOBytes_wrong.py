# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zoneinfo__common"
# dimension = "type"
# case = "load_data__fobj_as__IOBytes_wrong"
# subject = "zoneinfo._common.load_data(fobj: _IOBytes)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zoneinfo/_common.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zoneinfo._common.load_data(fobj: _IOBytes); call it with the wrong type.

typeshed contract: fobj is _IOBytes. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from zoneinfo._common import load_data
try:
    load_data(_W())  # fobj: _IOBytes <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
