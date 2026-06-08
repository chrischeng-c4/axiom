# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# sorted / min / max with key and reverse parameters

# sorted with key
print(sorted([(1, 'c'), (3, 'a'), (2, 'b')], key=lambda t: t[1]))

# sorted reverse
print(sorted([3, 1, 4, 1, 5, 9, 2, 6], reverse=True))

# sorted with len key
print(sorted(["apple", "banana", "kiwi"], key=len))

# sorted with key + reverse
print(sorted([(1, 'c'), (3, 'a'), (2, 'b')], key=lambda t: t[1], reverse=True))

# sorted on strings — case-insensitive via lambda
print(sorted(["Banana", "apple", "Cherry"], key=lambda s: s.lower()))

# min / max with key
print(min([(1, 'c'), (3, 'a'), (2, 'b')], key=lambda t: t[1]))
print(max([(1, 'c'), (3, 'a'), (2, 'b')], key=lambda t: t[1]))

# min / max with default on empty
print(max([], default=-1))
print(min([], default=99))

# min / max with multiple args (no list)
print(min(3, 1, 4, 1, 5))
print(max(3, 1, 4, 1, 5))
print(min("x", "a", "m"))

# min / max with key on multiple args
print(min("apple", "pi", "banana", key=len))
print(max("apple", "pi", "banana", key=len))
