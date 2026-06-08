# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "behavior"
# case = "sigterm_sigint_constant_values"
# subject = "signal.Signals"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""signal.Signals: POSIX-stable signal numbers: SIGTERM == 15, SIGINT == 2, SIGKILL == 9, SIGHUP == 1, SIGALRM == 14, SIG_DFL == 0, SIG_IGN == 1"""
import signal

for name, value in [
    ("SIGINT", 2),
    ("SIGTERM", 15),
    ("SIGKILL", 9),
    ("SIGHUP", 1),
    ("SIGALRM", 14),
    ("SIG_DFL", 0),
    ("SIG_IGN", 1),
]:
    got = int(getattr(signal, name))
    assert got == value, f"{name} = {got!r}, expected {value}"

print("sigterm_sigint_constant_values OK")
