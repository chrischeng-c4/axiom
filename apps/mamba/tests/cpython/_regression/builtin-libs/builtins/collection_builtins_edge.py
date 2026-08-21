# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Collection builtins edge cases: sorted with key/reverse, min/max with key/default
# Collection builtins edge cases conformance (S8-S10)
# sorted with key, all/any, min/max with default

# S8: sorted with key
print(sorted([3, 1, 4, 1, 5]))
print(sorted('hello'))
print(sorted([3, 1, 2], reverse=True))
print(sorted(['banana', 'apple'], key=len))

# sorted with various key functions
print(sorted([-3, 1, -2, 4], key=abs))
print(sorted(['Hello', 'world', 'Abc'], key=str.lower))
print(sorted([]))
print(sorted([1]))
print(sorted([(1, 'b'), (2, 'a'), (1, 'a')]))

# S9: all/any with generators
print(all([True, True, True]))
print(all([True, False, True]))
print(any([False, False, True]))
print(any([]))
print(all([]))

# all/any with various iterables
print(all([1, 2, 3]))
print(all([1, 0, 3]))
print(any([0, 0, 0]))
print(any([0, 0, 1]))
print(all('abc'))
print(any(''))

# all/any with generators
print(all(x > 0 for x in [1, 2, 3]))
print(all(x > 0 for x in [1, -1, 3]))
print(any(x > 5 for x in [1, 2, 3]))
print(any(x > 2 for x in [1, 2, 3]))

# S10: min/max with key and default
print(min([3, 1, 4], key=lambda x: -x))
print(max([], default='empty'))
print(min('hello'))

# min/max additional
print(max([3, 1, 4, 1, 5]))
print(min([3, 1, 4, 1, 5]))
print(max('hello'))
print(min([], default=0))
print(max(['apple', 'banana'], key=len))
print(min(['apple', 'banana'], key=len))
