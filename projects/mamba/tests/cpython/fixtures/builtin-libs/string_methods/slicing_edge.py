# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# String slicing: negative indices, steps, boundary cases

s = "abcdefgh"

# Positive slices
print(s[0:3])
print(s[2:5])
print(s[:4])
print(s[4:])
print(s[:])

# Negative indices
print(s[-3:])
print(s[:-3])
print(s[-5:-2])

# Out-of-bounds — clamps, doesn't raise
print(s[0:100])
print(s[-100:3])
print(s[10:20])

# Step
print(s[::2])
print(s[1::2])
print(s[::-1])
print(s[::-2])

# Step with start/stop
print(s[1:6:2])
print(s[6:1:-1])
print(s[-1:-6:-1])

# Empty result
print(s[5:3])
print(s[3:3])
print(s[100:200])

# Single char
print(s[0])
print(s[-1])
print(s[3])

# String length-agnostic
empty = ""
print(empty[:])
print(empty[0:1])
print(empty[::-1])
