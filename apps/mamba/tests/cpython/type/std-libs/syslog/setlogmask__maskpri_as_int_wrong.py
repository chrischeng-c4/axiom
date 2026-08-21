# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "syslog"
# dimension = "type"
# case = "setlogmask__maskpri_as_int_wrong"
# subject = "syslog.setlogmask(maskpri: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/syslog.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: syslog.setlogmask(maskpri: int); call it with the wrong type.

typeshed contract: maskpri is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from syslog import setlogmask
try:
    setlogmask("not_an_int")  # maskpri: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
