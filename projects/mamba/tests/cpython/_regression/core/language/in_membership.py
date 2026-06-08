# Membership operator: in / not in across container types

# List
print(3 in [1, 2, 3])
print(4 in [1, 2, 3])
print(3 not in [1, 2, 3])
print(4 not in [1, 2, 3])

# Tuple
print("a" in ("a", "b", "c"))
print("d" in ("a", "b", "c"))

# String
print("ell" in "hello")
print("xyz" in "hello")
print("" in "hello")
print("h" in "hello")

# Set
print(2 in {1, 2, 3})
print(5 in {1, 2, 3})

# Dict — checks keys
print("a" in {"a": 1, "b": 2})
print(1 in {"a": 1, "b": 2})

# Range
print(5 in range(10))
print(10 in range(10))
print(-1 in range(10))

# Empty containers
print(1 in [])
print("x" in "")
print(1 in {})
print(1 in set())

# Mixed numeric types — True is equal to 1
print(True in [1, 2, 3])
print(1 in [True, False])
print(1.0 in [1, 2, 3])

# Nested lists — deep equality
print([1, 2] in [[1, 2], [3, 4]])
print([1, 3] in [[1, 2], [3, 4]])

# in loop condition
def find(items, target):
    for x in items:
        if x == target:
            return True
    return False

print(find([1, 2, 3], 2))
print(find([1, 2, 3], 5))
