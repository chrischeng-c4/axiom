# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "behavior"
# case = "signal_returns_previous_handler"
# subject = "signal.signal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""signal.signal: signal.signal returns the handler it replaced; installing a second handler over SIGUSR1 returns the first one, then restore SIG_DFL"""
import signal


def _handler_a(n, f):
    pass


def _handler_b(n, f):
    pass


# The first install returns whatever was registered before (SIG_DFL/SIG_IGN
# or some callable inherited from the runner).
_prev1 = signal.signal(signal.SIGUSR1, _handler_a)
assert _prev1 in (signal.SIG_DFL, signal.SIG_IGN) or callable(_prev1), \
    f"first signal: {_prev1!r}"

# The second install must return exactly the handler we just put in place.
_prev2 = signal.signal(signal.SIGUSR1, _handler_b)
assert _prev2 is _handler_a, "second signal returns previous handler"

signal.signal(signal.SIGUSR1, signal.SIG_DFL)
print("signal_returns_previous_handler OK")
