# map / filter patterns broad

# map with lambda
print(list(map(lambda x: x * 2, [1, 2, 3, 4, 5])))
print(list(map(lambda x: x + 10, range(5))))

# filter with lambda
print(list(filter(lambda x: x > 2, [1, 2, 3, 4, 5])))
print(list(filter(lambda x: x % 2 == 0, range(10))))

# filter + sum
print(sum(filter(lambda x: x % 2 == 0, range(10))))

# map + sum
print(sum(map(lambda x: x * x, range(5))))
