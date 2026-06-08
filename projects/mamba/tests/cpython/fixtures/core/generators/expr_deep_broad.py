# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# generator expression deep broad

# basic gen exprs as args
print(sum(x for x in range(10)))
print(sum(x * 2 for x in range(5)))
print(sum(x for x in range(100) if x % 2 == 0))

# list from gen expr
print(list(x for x in range(5)))
print(list(x + 1 for x in range(3)))
print(list(x * 10 for x in range(4)))

# tuple from gen expr
print(tuple(x for x in range(5)))
print(tuple(x * 2 for x in [1, 2, 3]))

# set from gen expr
print(sorted(set(x % 3 for x in range(10))))

# dict from gen expr (via tuples)
print(dict((x, x * 2) for x in range(5)))

# max/min on gen expr
print(max(x for x in [3, 1, 2]))
print(min(x for x in [3, 1, 2]))
print(min(x for x in range(5)))

# sorted on gen expr
print(sorted(x for x in [3, 1, 4, 1, 5, 9]))
print(sorted(x * 2 for x in [3, 1, 2]))

# gen expr with condition
print(list(x for x in range(20) if x > 10))
print(list(x for x in range(20) if x % 3 == 0))

# gen expr in any/all
print(any(x > 5 for x in [1, 2, 3]))
print(any(x > 5 for x in [1, 2, 10]))
print(all(x > 0 for x in [1, 2, 3]))
print(all(x > 0 for x in [1, -2, 3]))

# gen expr with method
words = ["hello", "world", "python"]
print(list(w.upper() for w in words))
print(list(len(w) for w in words))

# gen expr over dict
d = {"a": 1, "b": 2, "c": 3}
print(sorted(v * 10 for v in d.values()))
print(sorted(k for k in d.keys()))

# gen expr string ops
print("-".join(str(x) for x in range(5)))
