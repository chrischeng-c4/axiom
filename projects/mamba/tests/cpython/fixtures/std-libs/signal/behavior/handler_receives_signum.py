# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "behavior"
# case = "handler_receives_signum"
# subject = "signal.signal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""signal.signal: a custom SIGUSR1 handler installed via signal.signal runs exactly once per os.kill(getpid(), SIGUSR1) and receives the signal number as its first argument"""
import os
import signal

_received = []


def _record_handler(signum, frame):
    _received.append(signum)


signal.signal(signal.SIGUSR1, _record_handler)
os.kill(os.getpid(), signal.SIGUSR1)
assert len(_received) == 1, f"handler called once: {_received!r}"
assert _received[0] == signal.SIGUSR1, f"signum = {_received[0]!r}"

signal.signal(signal.SIGUSR1, signal.SIG_DFL)
print("handler_receives_signum OK")
