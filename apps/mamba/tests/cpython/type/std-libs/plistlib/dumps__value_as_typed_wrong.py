# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "plistlib"
# dimension = "type"
# case = "dumps__value_as_typed_wrong"
# subject = "plistlib.dumps(value: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/plistlib.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: plistlib.dumps(value: typed); call it with the wrong type.

typeshed contract: value is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from plistlib import dumps
try:
    dumps(_W())  # value: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
