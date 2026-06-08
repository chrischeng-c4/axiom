# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# filter, map, any, all, sum builtins

# any / all
print(any([False, False, True]))
print(any([False, 0, ""]))
print(all([True, 1, "x"]))
print(all([True, 0, True]))
print(any([]))
print(all([]))

# filter with None drops falsy
print(list(filter(None, [0, 1, 0, 2, 0, 3])))

# filter with predicate
print(list(filter(lambda x: x > 0, [-1, 0, 1, 2, -3])))

# filter on strings
print(list(filter(lambda s: len(s) > 1, ["", "a", "hi", "x"])))

# map with function
print(list(map(abs, [-1, 2, -3])))

# map with built-in type coercion
print(list(map(str, [1, 2, 3])))

# map with lambda
print(list(map(lambda x: x * x, range(5))))

# sum with optional start
print(sum([1, 2, 3]))
print(sum([1, 2, 3], 10))
print(sum([], 5))
print(sum(range(1, 11)))

# sum with tuples
print(sum((1.0, 2.0, 3.0)))
