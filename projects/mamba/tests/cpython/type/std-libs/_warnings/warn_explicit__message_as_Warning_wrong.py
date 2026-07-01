# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "_warnings"
# dimension = "type"
# case = "warn_explicit__message_as_Warning_wrong"
# subject = "_warnings.warn_explicit(message: Warning)"
# kind = "semantic"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/_warnings.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: _warnings.warn_explicit(message: Warning); call it with the wrong type.

typeshed contract: message is Warning. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from _warnings import warn_explicit
try:
    warn_explicit(_W(), None, "", 0)  # message: Warning <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
