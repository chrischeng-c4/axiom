# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# list.index, list.count, list.remove with string / bool / nested values
# All three rely on Python-semantic equality, not pointer identity.

# index — integers
l1 = [10, 20, 30, 20, 10]
print(l1.index(20))
print(l1.index(10))
print(l1.index(30))

# index — strings
words = ["alpha", "beta", "gamma"]
print(words.index("beta"))
print(words.index("alpha"))

# index raises ValueError if missing
try:
    words.index("delta")
except ValueError as e:
    print("VE:", e)

# count — integers
nums = [1, 2, 2, 3, 2, 4]
print(nums.count(2))
print(nums.count(1))
print(nums.count(99))

# count — strings
letters = ["a", "b", "a", "c", "a"]
print(letters.count("a"))
print(letters.count("b"))
print(letters.count("x"))

# count — bool/int cross
mixed = [1, True, 0, False, 1]
print(mixed.count(1))
print(mixed.count(0))

# remove — first occurrence only
l2 = ["x", "y", "x", "z"]
l2.remove("x")
print(l2)

# remove raises ValueError if missing
try:
    l2.remove("w")
except ValueError as e:
    print("VE:", e)

# Tuple.index / count
t = ("a", "b", "c", "b")
print(t.index("b"))
print(t.count("b"))

# Nested
nested = [[1, 2], [3, 4], [1, 2]]
print(nested.count([1, 2]))
print(nested.index([3, 4]))