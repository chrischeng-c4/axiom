# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# list.extend accepts any iterable (list, tuple, set, frozenset, dict, str, bytes)

l1 = [1, 2, 3]
l1.extend([4, 5])
print(l1)

# tuple
l2 = [1, 2]
l2.extend((3, 4, 5))
print(l2)

# set — iteration order unspecified, use sorted()
l3 = [10]
l3.extend({20, 30, 40})
print(sorted(l3))

# frozenset
l4 = []
l4.extend(frozenset([100, 200]))
print(sorted(l4))

# dict iterates keys
l5 = []
l5.extend({"a": 1, "b": 2, "c": 3})
print(sorted(l5))

# str iterates chars
l6 = []
l6.extend("abc")
print(l6)

# bytes iterates ints
l7 = []
l7.extend(b"AB")
print(l7)

# chained
l8 = []
l8.extend([1, 2])
l8.extend((3, 4))
l8.extend(frozenset([5]))
print(sorted(l8))

# extend empty
l9 = [1]
l9.extend([])
l9.extend(())
print(l9)