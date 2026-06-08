# mamba-xfail: slice assignment on a user __setitem__ container is rejected
# at type-check ("type mismatch in assignment: expected `Recorder`, got
# `list[int]`") instead of being routed through __setitem__. Integer-key
# get/set/del and slice get clauses already pass on mamba; the slice-key
# __setitem__ / __delitem__ clauses are the runtime gap that gates the
# xfail.
#
# Slicing/index protocol — #2810.
#
# Covers user-defined __getitem__ / __setitem__ / __delitem__ with both
# integer and slice keys, plus slice object attribute inspection
# (.start, .stop, .step). Negative integer indices are exercised under
# __getitem__.
#
# A `Recorder` user-sequence captures the actual key passed to each
# dunder so we can confirm the runtime dispatches to the user method
# with the right operand (integer vs slice object). Every print line is
# tagged with `[slicing-index]` so failure output names the area.

class Recorder:
    def __init__(self, data):
        self.data = list(data)
        self.log = []

    def __getitem__(self, key):
        if isinstance(key, slice):
            self.log.append(("get-slice", key.start, key.stop, key.step))
            return self.data[key]
        self.log.append(("get-int", key))
        return self.data[key]

    def __setitem__(self, key, value):
        if isinstance(key, slice):
            self.log.append(("set-slice", key.start, key.stop, key.step, list(value)))
            self.data[key] = list(value)
        else:
            self.log.append(("set-int", key, value))
            self.data[key] = value

    def __delitem__(self, key):
        if isinstance(key, slice):
            self.log.append(("del-slice", key.start, key.stop, key.step))
            del self.data[key]
        else:
            self.log.append(("del-int", key))
            del self.data[key]


r = Recorder([10, 20, 30, 40, 50])

# 1. Integer read.
print("r[0]=", r[0], "[slicing-index]")
print("r[2]=", r[2], "[slicing-index]")

# 2. Negative integer read.
print("r[-1]=", r[-1], "[slicing-index: negative]")

# 3. Slice read with explicit start/stop.
print("r[1:4]=", r[1:4], "[slicing-index: slice]")

# 4. Slice read with step.
print("r[::2]=", r[::2], "[slicing-index: step]")

# 5. Slice attribute inspection through last logged entry.
last = r.log[-1]
print("last log tag=", last[0], "[slicing-index]")
print("slice start/stop/step=", last[1], last[2], last[3], "[slicing-index]")

# 6. Integer assignment.
r[0] = 99
print("after r[0]=99 ->", r.data[0], "[slicing-index]")

# 7. Slice assignment.
r[1:3] = [21, 31]
print("after r[1:3]=[21,31] ->", r.data[:4], "[slicing-index: slice-set]")

# 8. Integer delete.
del r[0]
print("after del r[0] ->", r.data, "[slicing-index]")

# 9. Slice delete.
del r[0:2]
print("after del r[0:2] ->", r.data, "[slicing-index: slice-del]")

# 10. Confirm dispatch tags accumulated in order.
print("log tags=", [entry[0] for entry in r.log], "[slicing-index]")
