# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "behavior"
# case = "signals_enum_roundtrip"
# subject = "signal.Signals"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""signal.Signals: signal constants are typed Signals/Handlers enum members (SIGINT/SIGTERM are Signals, SIG_DFL/SIG_IGN are Handlers); Signals(2) is SIGINT and Signals(SIGINT).name == 'SIGINT'"""
import signal

# Constants are typed enum members, not bare ints.
assert isinstance(signal.SIGINT, signal.Signals), "SIGINT is Signals member"
assert isinstance(signal.SIGTERM, signal.Signals), "SIGTERM is Signals member"
assert isinstance(signal.SIG_DFL, signal.Handlers), "SIG_DFL is Handlers member"
assert isinstance(signal.SIG_IGN, signal.Handlers), "SIG_IGN is Handlers member"

# Signals members round-trip through their integer value.
assert signal.Signals(2) is signal.SIGINT, "Signals(2) is SIGINT"
assert signal.Signals(signal.SIGINT).name == "SIGINT", "Signals name lookup"

print("signals_enum_roundtrip OK")
