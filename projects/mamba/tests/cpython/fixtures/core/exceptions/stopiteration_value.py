# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Regression: StopIteration(x) must expose x via .value
# Prior to the fix, StopIteration("msg").value returned None regardless
# of the constructor argument, and `raise StopIteration(x)` surfaced
# None on e.value in the except handler.

# Direct construction
si = StopIteration("hello")
print(si.value)
print(si.args)

# Raise + catch
try:
    raise StopIteration("oops")
except StopIteration as e:
    print(e.value)
    print(e.args)

# Empty StopIteration still has .value = None
try:
    raise StopIteration
except StopIteration as e:
    print(e.value)

# Generator return value (already worked, guard against regression)
def g():
    yield 1
    return "done"

gi = g()
print(next(gi))
try:
    next(gi)
except StopIteration as e:
    print(e.value)