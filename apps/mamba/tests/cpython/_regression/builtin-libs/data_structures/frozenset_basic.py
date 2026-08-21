# frozenset() — immutable, hashable

fs = frozenset([1, 2, 3])
print(sorted(fs))
print(len(fs))

# Membership
print(2 in fs)
print(5 in fs)

# Operations (read-only)
fs2 = frozenset([3, 4, 5])
print(sorted(fs | fs2))
print(sorted(fs & fs2))
print(sorted(fs - fs2))
print(sorted(fs ^ fs2))

# issubset / issuperset
print(frozenset([1, 2]).issubset(fs))
print(fs.issuperset(frozenset([1, 2])))
print(fs.isdisjoint(frozenset([9, 8])))

# Immutability — add/remove/discard must raise AttributeError
try:
    fs.add(4)
except AttributeError as e:
    print("AE:", e)

try:
    fs.remove(1)
except AttributeError as e:
    print("AE:", e)

# Equality between frozensets
print(frozenset([1, 2]) == frozenset([2, 1]))
print(frozenset([1, 2]) == frozenset([1, 2, 3]))

# String members
fs_s = frozenset(["a", "b", "c"])
print("a" in fs_s)
print("z" in fs_s)
