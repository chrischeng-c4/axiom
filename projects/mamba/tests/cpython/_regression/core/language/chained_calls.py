# method chaining

# string
print("  hello  ".strip().upper())
print(" Hello World ".strip().lower().replace("world", "there"))

# list
print(sorted([3, 1, 4, 1, 5, 9, 2, 6])[:5])
print(list(reversed(sorted([3, 1, 4, 1, 5]))))

# dict
d = {"c": 3, "a": 1, "b": 2}
print(sorted(d.items()))

# mix
names = ["Alice", "Bob", "Charlie"]
print([n.upper() for n in sorted(names, key=len)])

# chained comparisons
print(1 < 2 < 3 < 4)
print(1 < 2 < 0)
print(10 >= 10 >= 10)

# nested expressions
print([x for x in range(10) if x % 2 == 0][::2])
print(sorted(set([3, 1, 4, 1, 5, 9, 2, 6])))
print(min([x * 2 for x in range(5)]))
print(max([x * x for x in range(10)]))

# fluent-style helpers
words = ["foo", "bar", "baz", "qux"]
print(",".join(sorted(set(words))))
print(",".join([w.upper() for w in words]))

# multiple method calls
print("a-b-c".split("-"))
print("-".join("a-b-c".split("-")))

# generator expressions
print(sum(x for x in range(10) if x % 2 == 0))
print(list(x * 2 for x in range(5)))
