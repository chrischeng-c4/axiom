# Operational AssertionPass seed for `lambda` expressions.
# Surface: basic positional lambda, default-arg lambda, lambda inside
# filter/map, lambda returning a constant, nested lambda (curried).
# Closure-capture-of-loop-var lambdas are NOT exercised here — they
# currently share state across instances (Task #14).
_ledger: list[int] = []

# Basic positional-arg lambda — `r - 5 == 0` dodges the
# int-identity-through-return drop (Task #15) on the lambda return.
add = lambda x, y: x + y
r = add(2, 3)
assert r - 5 == 0; _ledger.append(1)

# Lambda with default arg
greet = lambda name="world": f"hello {name}"
assert greet() == "hello world"; _ledger.append(1)
assert greet("foo") == "hello foo"; _ledger.append(1)

# Lambda inside filter — keeps only even numbers
nums = [1, 2, 3, 4, 5, 6, 7, 8]
evens = list(filter(lambda x: x % 2 == 0, nums))
assert evens == [2, 4, 6, 8]; _ledger.append(1)

# Lambda inside map — applies a unary transform across the iterable
squares = list(map(lambda x: x * x, [1, 2, 3, 4]))
assert squares == [1, 4, 9, 16]; _ledger.append(1)

# Zero-arg lambda returning a constant
const = lambda: 42
c = const()
assert c - 42 == 0; _ledger.append(1)

# Nested lambda — currying: outer lambda returns a new inner lambda
mult = lambda x: lambda y: x * y
m3 = mult(3)
v = m3(4)
assert v - 12 == 0; _ledger.append(1)

# Lambda as a comparison predicate-bearing argument: pairs satisfy
# a binary relation via filter
pairs = [(1, 2), (5, 3), (4, 4), (2, 7)]
asc = list(filter(lambda p: p[0] < p[1], pairs))
assert asc == [(1, 2), (2, 7)]; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: lang_lambda_expressions {sum(_ledger)} asserts")
