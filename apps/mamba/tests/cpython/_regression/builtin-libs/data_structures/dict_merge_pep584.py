# PEP 584: dict merge operators (| and |=)
d1 = {'a': 1, 'b': 2}
d2 = {'b': 3, 'c': 4}
# | creates new dict, d2 wins on conflict
result = d1 | d2
print(sorted(result.items()))
# d1 unchanged
print(sorted(d1.items()))
# |= merges in place
d1 |= d2
print(sorted(d1.items()))
# empty merge
print(sorted(({} | {'x': 1}).items()))
print(sorted(({'x': 1} | {}).items()))
