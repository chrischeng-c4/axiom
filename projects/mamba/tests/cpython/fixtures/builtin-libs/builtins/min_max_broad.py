# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# min / max patterns broad

# basic
print(min(1, 2, 3))
print(max(1, 2, 3))
print(min(5, -3, 8, 0))
print(max(5, -3, 8, 0))

# min/max of list
print(min([3, 1, 4, 1, 5, 9, 2, 6]))
print(max([3, 1, 4, 1, 5, 9, 2, 6]))

# min/max of tuple
print(min((7, 2, 9, 4)))
print(max((7, 2, 9, 4)))

# min/max of string
print(min("hello"))
print(max("hello"))
print(min("xyzabc"))
print(max("xyzabc"))

# min/max of strings
print(min(["banana", "apple", "cherry"]))
print(max(["banana", "apple", "cherry"]))

# min/max with key=abs
print(min([-5, 3, -1, 4], key=abs))
print(max([-5, 3, -1, 4], key=abs))

# min/max with key=lambda
words = ["banana", "apple", "kiwi", "strawberry"]
print(min(words, key=len))
print(max(words, key=len))

# min/max with negation
print(min([1, 2, 3, 4, 5], key=lambda x: -x))
print(max([1, 2, 3, 4, 5], key=lambda x: -x))

# min/max of single
print(min([42]))
print(max([42]))

# min/max with default
print(min([], default=0))
print(max([], default=-1))
print(min([], default="empty"))

# min/max of range
print(min(range(10)))
print(max(range(10)))
print(min(range(5, 15)))
print(max(range(5, 15)))

# min/max two args
print(min(5, 10))
print(max(5, 10))
print(min(-1, 1))
print(max(-1, 1))

# with floats (no mixing)
print(min(1.5, 2.5, 0.5))
print(max(1.5, 2.5, 0.5))
print(min([3.14, 2.71, 1.41]))
print(max([3.14, 2.71, 1.41]))
