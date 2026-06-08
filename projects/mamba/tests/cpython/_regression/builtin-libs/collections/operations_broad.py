# collections operations broad

# list operations
lst = [3, 1, 4, 1, 5, 9, 2, 6]
print(len(lst))
print(sum(lst))
print(max(lst))
print(min(lst))
print(sorted(lst))
print(sorted(lst, reverse=True))

# list in/not in
print(3 in lst)
print(100 in lst)
print(100 not in lst)
print(3 not in lst)

# list index / count
print(lst.index(4))
print(lst.count(1))
print(lst.count(99))

# set operations
s = {1, 2, 3, 4, 5}
t = {3, 4, 5, 6, 7}
print(sorted(s | t))
print(sorted(s & t))
print(sorted(s - t))
print(sorted(t - s))
print(sorted(s ^ t))
print(len(s))
print(3 in s)
print(100 in s)

# set methods
s2 = {1, 2, 3}
s2.add(4)
print(sorted(s2))
s2.discard(2)
print(sorted(s2))
s2.remove(1)
print(sorted(s2))

# frozenset (if supported)
fs = frozenset([1, 2, 3])
print(sorted(fs))
print(1 in fs)

# dict operations
d = {"a": 1, "b": 2, "c": 3}
print(len(d))
print(sorted(d.keys()))
print(sorted(d.values()))
print("a" in d)
print("z" in d)

# tuple operations
t2 = (1, 2, 3, 1, 2, 1)
print(len(t2))
print(t2.count(1))
print(t2.index(2))
print(sum(t2))
print(max(t2))
print(min(t2))
