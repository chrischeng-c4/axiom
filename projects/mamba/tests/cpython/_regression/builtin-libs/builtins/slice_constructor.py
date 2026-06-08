# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# `slice(stop)` / `slice(start, stop[, step])` — Python slice constructor.
# Was undefined-name (#1256 long-tail tracker, sub-priority 1). Slice
# literals like `a[1:5:2]` already work; only the explicit constructor
# was missing. Fixed by routing the name through a `mb_slice(start,
# stop, step)` runtime that returns an Instance with class_name="slice"
# and fields start/stop/step. Lower-pass adapts the 1- and 2-arg forms
# (`slice(stop)` → `mb_slice(None, stop, None)`).

# Canonical 1-arg form binds to `stop`, NOT `start`.
print(slice(5))                 # slice(None, 5, None)
print(slice(None))              # slice(None, None, None)

# 2-arg form: start, stop.
print(slice(1, 10))             # slice(1, 10, None)
print(slice(0, 0))              # slice(0, 0, None)

# 3-arg form: start, stop, step.
print(slice(1, 10, 2))          # slice(1, 10, 2)
print(slice(None, None, -1))    # slice(None, None, -1)
print(slice(0, 100, 5))         # slice(0, 100, 5)

# Field access — `.start`, `.stop`, `.step` are exposed as attributes.
s = slice(2, 8, 3)
print(s.start)                  # 2
print(s.stop)                   # 8
print(s.step)                   # 3

# `.start` defaults to None on the 1-arg form.
s1 = slice(7)
print(s1.start, s1.stop, s1.step)   # None 7 None

# `repr()` matches the print form (slice has no separate __str__).
print(repr(slice(1, 10, 2)))    # slice(1, 10, 2)
print(repr(slice(None)))        # slice(None, None, None)

# Container repr threads through `__repr__` per element.
print([slice(1, 5), slice(None)])
# [slice(1, 5, None), slice(None, None, None)]

# Negative + zero values pass through verbatim — slice doesn't validate.
print(slice(-3, -1, -1))        # slice(-3, -1, -1)
print(slice(0))                 # slice(None, 0, None)
