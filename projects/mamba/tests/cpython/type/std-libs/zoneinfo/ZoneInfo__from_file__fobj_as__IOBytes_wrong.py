# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "zoneinfo"
# dimension = "type"
# case = "ZoneInfo__from_file__fobj_as__IOBytes_wrong"
# subject = "zoneinfo.ZoneInfo.from_file(fobj: _IOBytes)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/zoneinfo.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: zoneinfo.ZoneInfo.from_file(fobj: _IOBytes); call it with the wrong type.

typeshed contract: fobj is _IOBytes. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from zoneinfo import ZoneInfo
try:
    ZoneInfo.from_file(_W())  # fobj: _IOBytes <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
