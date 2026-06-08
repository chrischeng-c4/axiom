# Regression: map()/filter() must accept instance-shaped callables —
# unbound method wrappers (`str.upper`), functools.partial, and any
# object with __call__. Previously the dispatch only checked for TAG_FUNC
# addresses and named-string callables, so `map(str.upper, ...)` hit the
# fall-through and produced an empty list.

print(list(map(str.upper, ["a", "b", "c"])))
print(list(map(str.lower, ["AB", "CD"])))
print(list(map(str.strip, ["  hi  ", " bye "])))

# filter with unbound predicates
print(list(filter(str.isdigit, ["12", "ab", "3"])))
print(list(filter(str.isalpha, ["hi", "7", "bye", "!"])))

# Combined
print(list(map(str.upper, filter(str.isalpha, ["a", "1", "b", "@", "c"]))))
