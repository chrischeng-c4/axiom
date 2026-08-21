# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "warnings"
# dimension = "type"
# case = "deprecated__init__message_as_LiteralString_wrong"
# subject = "warnings.deprecated.__init__(message: LiteralString)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/warnings.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: warnings.deprecated.__init__(message: LiteralString); call it with the wrong type.

typeshed contract: message is LiteralString. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from warnings import deprecated
try:
    deprecated(_W())  # message: LiteralString <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
