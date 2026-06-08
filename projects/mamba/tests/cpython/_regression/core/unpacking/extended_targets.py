# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""core/unpacking: very wide unpack targets (CPython 3.12 oracle).

Unpacking into a large number of targets exercises the extended-argument
encoding of the UNPACK_SEQUENCE bytecode. CPython once had a bug where the
high bits of the operand were dropped past 255 targets; here we unpack 400
positions and confirm the last element survives the round trip.
"""

# Build `(y, y, ... y,) = x` with 400 targets, then return the final binding.
target = "(" + "y," * 400 + ")"
code = f"def unpack_400(x):\n    {target} = x\n    return y\n"
ns = {}
exec(code, ns)
unpack_400 = ns["unpack_400"]

# Repeat so a JIT/optimizer cannot specialize away the wide unpack on iter 1.
for _ in range(30):
    last = unpack_400(range(400))
    assert last == 399, last

print("extended-targets: last =", last)
