# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "syslog"
# dimension = "type"
# case = "syslog__priority_as_int_wrong"
# subject = "syslog.syslog(priority: int)"
# kind = "semantic"
# xfail = "force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed priority"
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/syslog.pyi"
# status = "filled"
# ///
# mamba-xfail: force-typed arg enforcement pending; mamba must raise TypeError on wrong-typed priority
# mamba-strict-type: TypeError
"""Type wall: syslog.syslog(priority: int); call it with the wrong type.

typeshed contract: priority is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from syslog import syslog
try:
    syslog("not_an_int", "")  # priority: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
