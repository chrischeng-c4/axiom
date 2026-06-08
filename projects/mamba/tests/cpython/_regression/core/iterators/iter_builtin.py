# iter() and next() builtins on various sources

# iter() on list
it = iter([1, 2, 3])
print(next(it))
print(next(it))
print(next(it))

# iter() on tuple
it2 = iter((10, 20, 30))
print(next(it2))
print(next(it2))

# iter() on string
it3 = iter("abc")
print(next(it3))
print(next(it3))
print(next(it3))

# iter() on dict — iterates keys
d = {"a": 1, "b": 2}
it4 = iter(d)
# Order preserved in 3.7+
print(next(it4))
print(next(it4))

# iter() on range
it5 = iter(range(5, 8))
print(next(it5))
print(next(it5))
print(next(it5))

# iter() on set (order unpredictable but single elements work)
s = {42}
print(next(iter(s)))

# next() with default value
it6 = iter([1])
print(next(it6))
print(next(it6, "fallback"))
print(next(it6, -1))

# Manual iteration via for-loop works with iter() objects
it7 = iter([10, 20, 30])
total = 0
for x in it7:
    total += x
print(total)

# iter() on empty source + default in next
it8 = iter([])
print(next(it8, "empty"))
