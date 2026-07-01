# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_msi"
# dimension = "type"
# case = "FCICreate__cabname_as_str_wrong"
# subject = "_msi.FCICreate(cabname: str)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_msi.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _msi.FCICreate(cabname: str); call it with the wrong type.

typeshed contract: cabname is str. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from _msi import FCICreate
try:
    FCICreate(12345, None)  # cabname: str <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
