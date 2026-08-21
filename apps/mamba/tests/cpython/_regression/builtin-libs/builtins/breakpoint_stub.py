# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# breakpoint(*args, **kwargs) — PEP 553 (#1256 long-tail tracker, sub-priority 5).
# Mamba has no pdb runtime, so the default sys.breakpointhook is a
# silent no-op returning None — matching CPython when run with
# PYTHONBREAKPOINT=0 (which is how this fixture's expected output
# is captured). The name itself was previously undefined.

# Bare call returns None silently.
print("before")
result = breakpoint()
print(result)                   # None
print("after")

# Positional args are accepted and ignored.
breakpoint(1, 2, 3)
print("after-args")

# Multi-call works without leaking state.
for _ in range(3):
    breakpoint()
print("loop-done")
