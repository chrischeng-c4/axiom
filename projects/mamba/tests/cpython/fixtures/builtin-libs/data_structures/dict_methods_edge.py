# Dict method edge cases — setdefault, get, pop, dict-from-pairs, |= merge

# setdefault: existing key keeps value, missing key inserts
d = {"a": 1}
print(d.setdefault("a", 99))
print(d.setdefault("b", 2))

# get with explicit and implicit default
print(d.get("c", "missing"))
print(d.get("a"))

# pop with/without default
print(d.pop("b"))
print(d.pop("z", "nope"))

# dict(...) constructor from list of pairs
d4 = dict([("x", 1), ("y", 2)])
print(sorted(d4.items()))

# |= merge (PEP 584): right wins on conflict
d6 = {"a": 1}
d6 |= {"b": 2, "a": 10}
print(sorted(d6.items()))
