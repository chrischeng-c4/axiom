# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "type"
# case = "pidfd_send_signal__pidfd_as_int_wrong"
# subject = "signal.pidfd_send_signal(pidfd: int)"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = "vendor/typeshed/stdlib/signal.pyi"
# status = "filled"
# ///
# mamba-strict-type: TypeError
"""Type wall: signal.pidfd_send_signal(pidfd: int); call it with the wrong type.

typeshed contract: pidfd is int. mamba is force-typed, so a wrong-typed
argument MUST raise TypeError (CPython may accept or raise — mamba's to enforce)."""

from signal import pidfd_send_signal
try:
    pidfd_send_signal("not_an_int", 0)  # pidfd: int <- wrong-typed
    print("no_typeerror:")  # CPython accepted the wrong-typed arg; mamba must raise
except TypeError as e:
    print("typeerror:", type(e).__name__)
except Exception as e:
    print("setup_or_other:", type(e).__name__)
