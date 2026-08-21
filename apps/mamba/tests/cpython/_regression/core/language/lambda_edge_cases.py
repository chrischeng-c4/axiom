# Lambda and closure edge cases

# Nested lambda
compose = lambda f, g: lambda x: f(g(x))
double = lambda x: x * 2
add1 = lambda x: x + 1
print(compose(double, add1)(3))

# Lambda in list
ops = [lambda x: x + 1, lambda x: x * 2, lambda x: x * 3]
print([op(5) for op in ops])

# Lambda as sort key
words = ['banana', 'apple', 'cherry', 'date']
print(sorted(words, key=lambda w: len(w)))

# Lambda used with map
nums = list(map(lambda x: x * 2, [1, 2, 3]))
print(nums)

# Lambda used with filter
evens = list(filter(lambda x: x % 2 == 0, range(10)))
print(evens)
