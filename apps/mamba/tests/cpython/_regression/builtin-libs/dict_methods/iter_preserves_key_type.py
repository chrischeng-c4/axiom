# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# dict iteration must preserve the original key type. Mamba previously
# stringified every key when iter(d) / d.__iter__() was used (whereas
# d.keys() was correct), so `for k in d:` made every key a str.

# Int keys
d = {1: 'a', 2: 'b', 10: 'c'}
print([type(k).__name__ for k in d])           # ['int', 'int', 'int']
print(sorted(iter(d)))                         # [1, 2, 10]
print(list(d.__iter__()))                      # [1, 2, 10]

# Bool keys (note: True == 1 in CPython, so {True: 'a', 1: 'b'} → {True: 'b'})
d = {True: 'x', False: 'y'}
print([type(k).__name__ for k in d])           # ['bool', 'bool']

# None key
d = {None: 'n', 'k': 'v'}
print([k for k in d])                          # [None, 'k']

# Mixed
d = {1: 'i', 'two': 's', None: 'n'}
print([type(k).__name__ for k in d])           # ['int', 'str', 'NoneType']

# String keys (the path that always worked) still works
d = {'a': 1, 'b': 2}
print(sorted(d))                               # ['a', 'b']
print(sorted(iter(d)))                         # ['a', 'b']
