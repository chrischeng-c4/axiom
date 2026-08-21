
# Dict operations
d = {'a': 1, 'b': 2, 'c': 3}
print(d['a'])
print(d.get('b'))
print(d.get('z', 99))
d['d'] = 4
print(d)
d.setdefault('e', 5)
print(d['e'])
d.setdefault('a', 100)
print(d['a'])
# pop
print(d.pop('d'))
print(d.pop('z', -1))
# update
d.update({'f': 6, 'g': 7})
print(len(d))
# keys, values, items
print(sorted(d.keys()))
print(sorted(d.values()))
# popitem
d2 = {'x': 10}
print(d2.popitem())
print(len(d2))
# copy and clear
d3 = d.copy()
d.clear()
print(len(d))
print(len(d3))
