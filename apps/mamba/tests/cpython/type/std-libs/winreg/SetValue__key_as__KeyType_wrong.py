# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "winreg"
# dimension = "type"
# case = "SetValue__key_as__KeyType_wrong"
# subject = "winreg.SetValue(key: _KeyType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/winreg.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: winreg.SetValue(key: _KeyType); call it with the wrong type.

typeshed contract: key is _KeyType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from winreg import SetValue
try:
    SetValue(_W(), None, 0, "")  # key: _KeyType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
