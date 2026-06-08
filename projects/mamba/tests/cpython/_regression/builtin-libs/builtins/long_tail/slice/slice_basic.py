# `slice` builtin — canonical CPython 3.12 conformance (#1256 sub-task 1).
#
# Coverage:
#   - 1-arg form binds to `stop` (NOT `start`).
#   - 2-arg form: (start, stop).
#   - 3-arg form: (start, stop, step).
#   - `.start`, `.stop`, `.step` attribute access.
#   - `repr()` matches `str()` / print form.
#   - 0-arg form raises TypeError with CPython's exact message.

# 1-arg form
print(slice(5))                       # slice(None, 5, None)
print(slice(None))                    # slice(None, None, None)

# 2-arg form
print(slice(1, 5))                    # slice(1, 5, None)

# 3-arg form
print(slice(1, 5, 2))                 # slice(1, 5, 2)

# Attribute access
s = slice(1, 10, 2)
print(s.start, s.stop, s.step)        # 1 10 2

# repr() — matches print form (slice has no separate __str__)
print(repr(slice(1, 10, 2)))          # slice(1, 10, 2)

# Edge case (R2): 0-arg form raises TypeError matching CPython exactly.
try:
    slice()
except TypeError as e:
    print("TypeError:", e)
