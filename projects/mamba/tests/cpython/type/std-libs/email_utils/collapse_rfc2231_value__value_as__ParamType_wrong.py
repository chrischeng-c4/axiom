# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_utils"
# dimension = "type"
# case = "collapse_rfc2231_value__value_as__ParamType_wrong"
# subject = "email.utils.collapse_rfc2231_value(value: _ParamType)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/utils.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.utils.collapse_rfc2231_value(value: _ParamType); call it with the wrong type.

typeshed contract: value is _ParamType. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from email.utils import collapse_rfc2231_value
try:
    collapse_rfc2231_value(_W())  # value: _ParamType <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
