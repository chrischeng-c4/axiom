# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "behavior"
# case = "sig_ign_suppresses_delivery"
# subject = "signal.signal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""signal.signal: installing SIG_IGN for SIGUSR2 makes os.kill(getpid(), SIGUSR2) a no-op (handler never runs) and getsignal still reports SIG_IGN; then restore SIG_DFL"""
import os
import signal

signal.signal(signal.SIGUSR2, signal.SIG_IGN)
os.kill(os.getpid(), signal.SIGUSR2)  # ignored: must not raise or run anything
assert signal.getsignal(signal.SIGUSR2) == signal.SIG_IGN, "still SIG_IGN"

signal.signal(signal.SIGUSR2, signal.SIG_DFL)
print("sig_ign_suppresses_delivery OK")
