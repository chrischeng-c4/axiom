# map/filter deep broad

# map basic
print(list(map(lambda x: x * 2, [1, 2, 3, 4])))
print(list(map(str, [1, 2, 3])))
print(list(map(len, ["a", "bb", "ccc"])))

# filter basic
print(list(filter(lambda x: x > 2, [1, 2, 3, 4, 5])))
print(list(filter(lambda x: x % 2 == 0, [1, 2, 3, 4, 5, 6])))

# filter empty result
print(list(filter(lambda x: x > 99, [1, 2, 3])))

# map + filter composition
print(list(map(lambda x: x * 2, filter(lambda x: x > 2, [1, 2, 3, 4, 5]))))
print(list(filter(lambda x: x > 5, map(lambda x: x * 2, [1, 2, 3, 4]))))

# map + sum
print(sum(map(lambda x: x * 2, [1, 2, 3, 4])))
print(sum(map(abs, [-1, 2, -3, 4])))

# filter + sum
print(sum(filter(lambda x: x > 2, [1, 2, 3, 4, 5])))

# map + list comprehension equivalent
print(list(map(lambda x: x + 1, range(5))))
print([x + 1 for x in range(5)])
