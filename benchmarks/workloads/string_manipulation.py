# Benchmark: String construction and manipulation.
# Measures: string concatenation, splitting, joining, formatting.

words: list = ["hello", "world", "foo", "bar", "baz", "qux", "python", "mamba"]

# Build a large string from repeated joining.
joined: str = ""
for _i in range(1000):
    joined = " ".join(words)

# Count characters by splitting and re-joining.
parts: list = joined.split(" ")
upper_parts: list = [p.upper() for p in parts]
result: str = "-".join(upper_parts)
print(len(result))

# String formatting.
total: int = 0
for i in range(500):
    s: str = "item_{:04d}".format(i)
    total += len(s)
print(total)
