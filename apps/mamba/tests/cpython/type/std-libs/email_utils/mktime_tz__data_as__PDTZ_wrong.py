# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "email_utils"
# dimension = "type"
# case = "mktime_tz__data_as__PDTZ_wrong"
# subject = "email.utils.mktime_tz(data: _PDTZ)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/email/utils.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: email.utils.mktime_tz(data: _PDTZ); call it with the wrong type.

typeshed contract: data is _PDTZ. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

class _W:
    pass


from email.utils import mktime_tz
try:
    mktime_tz(_W())  # data: _PDTZ <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
