# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "behavior"
# case = "handler_fires_once_per_kill"
# subject = "signal.signal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""signal.signal: a counting SIGUSR1 handler fires once per os.kill so three kills increment the counter to exactly 3"""
import os
import signal

_count = [0]


def _multi_handler(signum, frame):
    _count[0] += 1


signal.signal(signal.SIGUSR1, _multi_handler)
os.kill(os.getpid(), signal.SIGUSR1)
os.kill(os.getpid(), signal.SIGUSR1)
os.kill(os.getpid(), signal.SIGUSR1)
assert _count[0] == 3, f"handler called 3 times: {_count[0]!r}"

signal.signal(signal.SIGUSR1, signal.SIG_DFL)
print("handler_fires_once_per_kill OK")
