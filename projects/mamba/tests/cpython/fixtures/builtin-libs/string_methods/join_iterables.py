# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# str.join accepts any iterable (list, tuple, set, frozenset, str, iterator)

# list
print("-".join(["a", "b", "c"]))

# tuple
print(",".join(("x", "y", "z")))

# str iterates chars
print("|".join("abc"))

# set / frozenset — unordered, use sorted() input for determinism
print("".join(sorted({"b", "a", "c"})))
print("".join(sorted(frozenset(["d", "e", "f"]))))

# reversed (iterator)
print("".join(reversed("abcdef")))
print("-".join(reversed(["a", "b", "c"])))

# map (iterator)
print(",".join(map(str, [1, 2, 3])))

# filter (iterator)
print("/".join(filter(lambda s: len(s) > 0, ["a", "", "b", "", "c"])))

# empty
print("X".join([]))
print("X".join(""))

# single
print(",".join(["only"]))
