# Dict edge cases: empty dict, dict comprehension, nested dict
d = {}
print(len(d))
print(list(d.keys()))
# Dict comprehension
d = {x: x ** 2 for x in range(3)}
print(d)
# Nested dict access
d = {'a': {'b': 1}}
print(d['a']['b'])
# Dict edge cases: KeyError on missing key and pop without default
try:
    {}['x']
except KeyError:
    print('caught')
try:
    {}.pop('x')
except KeyError:
    print('caught')
