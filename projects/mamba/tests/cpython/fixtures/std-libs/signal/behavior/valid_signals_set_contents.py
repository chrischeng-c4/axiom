# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "behavior"
# case = "valid_signals_set_contents"
# subject = "signal.valid_signals"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""signal.valid_signals: valid_signals() returns a set holding SIGTERM and SIGINT but excluding the 0 and NSIG boundary markers, smaller than NSIG and with many entries"""
import signal

vs = signal.valid_signals()
assert isinstance(vs, (set, frozenset)), f"valid_signals type = {type(vs)!r}"
assert signal.SIGTERM in vs, "SIGTERM in valid_signals"
assert signal.SIGINT in vs, "SIGINT in valid_signals"

# Boundaries: 0 is not a signal, NSIG is the past-the-end marker.
assert 0 not in vs, "0 not in valid_signals"
assert signal.NSIG not in vs, "NSIG not in valid_signals"
assert len(vs) < signal.NSIG, "valid_signals smaller than NSIG"
assert len(vs) >= 6, f"valid_signals has many entries: {len(vs)}"

print("valid_signals_set_contents OK")
