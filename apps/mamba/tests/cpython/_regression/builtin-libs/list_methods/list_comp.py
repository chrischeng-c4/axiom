# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
print([x * 2 for x in range(5)])
print([x for x in range(10) if x % 2 == 0])
print([x * y for x in range(3) for y in range(3)])