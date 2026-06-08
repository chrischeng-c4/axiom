# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "type"
# case = "WarningMessage__init__message_as_typed_wrong"
# subject = "warnings.WarningMessage.__init__(message: typed)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/warnings.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: warnings.WarningMessage.__init__(message: typed); call it with the wrong type.

typeshed contract: message is typed. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from warnings import WarningMessage
try:
    WarningMessage(_W(), None, "", 0)  # message: typed <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
