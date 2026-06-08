# F-string format specification conformance

# Width and alignment
print(f"{'x':<5}|")
print(f"{'x':>5}|")
print(f"{'x':^5}|")

# Fill character with alignment
print(f"{'y':*<5}|")
print(f"{'z':.>5}|")

# Integer base formats
print(f"{255:b}")
print(f"{255:o}")
print(f"{255:x}")
print(f"{255:X}")

# Float precision + thousands separator
print(f"{3.14159:.2f}")
print(f"{3.14159:.4f}")
print(f"{1000000:,}")
