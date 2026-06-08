# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# dict.fromkeys — classmethod constructor

# 1-arg form: value defaults to None
d1 = dict.fromkeys(["a", "b", "c"])
print(sorted(d1.items()))

# 2-arg form: explicit fill value
d2 = dict.fromkeys(["x", "y", "z"], 0)
print(sorted(d2.items()))

# integer keys
d3 = dict.fromkeys([1, 2, 3], "init")
print(sorted(d3.items()))

# duplicate keys collapse to a single entry
d4 = dict.fromkeys(["a", "b", "a", "c", "b"], 1)
print(sorted(d4.items()))
print(len(d4))

# empty iterable yields empty dict
d5 = dict.fromkeys([])
print(d5)
print(len(d5))

# tuple as iterable
d6 = dict.fromkeys(("p", "q"), 7)
print(sorted(d6.items()))

# string is an iterable of chars
d7 = dict.fromkeys("abc", 1)
print(sorted(d7.items()))

# range is an iterable
d8 = dict.fromkeys(range(3), "v")
print(sorted(d8.items()))

# fill value is shared (same object) — mutate via one key visible via another
shared = []
d9 = dict.fromkeys(["k1", "k2"], shared)
d9["k1"].append(42)
print(d9["k1"])
print(d9["k2"])
print(d9["k1"] is d9["k2"])
