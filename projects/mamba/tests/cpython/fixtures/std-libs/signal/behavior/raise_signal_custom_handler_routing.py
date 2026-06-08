# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "signal"
# dimension = "behavior"
# case = "raise_signal_custom_handler_routing"
# subject = "signal.raise_signal"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""signal.raise_signal: a custom SIGINT handler intercepts raise_signal(SIGINT); each call routes to the handler and raise_signal itself returns None"""
import signal

seen = []
signal.signal(signal.SIGINT, lambda s, f: seen.append(s))

signal.raise_signal(signal.SIGINT)
assert seen == [signal.SIGINT], f"custom handler saw {seen!r}"

# raise_signal returns None on success and the handler fires a second time.
result = signal.raise_signal(signal.SIGINT)
assert result is None, f"raise_signal returns None, got {result!r}"
assert seen == [signal.SIGINT, signal.SIGINT], f"handler fired twice: {seen!r}"

signal.signal(signal.SIGINT, signal.default_int_handler)
print("raise_signal_custom_handler_routing OK")
