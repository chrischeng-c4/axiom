# collections patterns broad

# list mixed ops
lst = [1, 2, 3, 4, 5]
print(sum(lst))
print(min(lst))
print(max(lst))
print(sorted(lst, reverse=True))

# set ops broad
a = {1, 2, 3, 4}
b = {3, 4, 5, 6}
print(sorted(a | b))
print(sorted(a & b))
print(sorted(a - b))
print(sorted(a ^ b))
print(4 in a)
print(10 in a)

# dict ops broad
d = {"a": 1, "b": 2, "c": 3}
print(sorted(d.keys()))
print(sorted(d.values()))
print(sorted(d.items()))
print("a" in d)
print("z" in d)
print(d.get("a"))
print(d.get("z", 0))

# dict membership
if "b" in d:
    print("has b")

# nested collections
nested = {"nums": [1, 2, 3], "strs": ["x", "y"]}
print(nested["nums"][0])
print(nested["strs"][-1])
print(len(nested["nums"]))

# list of dicts
records = [{"n": 1}, {"n": 2}, {"n": 3}]
total = 0
for r in records:
    total += r["n"]
print(total)

# conversions between collections
print(sorted(list({"c", "a", "b"})))
print(sorted(tuple([3, 1, 2])))
print(sorted(set([1, 1, 2, 2, 3])))
