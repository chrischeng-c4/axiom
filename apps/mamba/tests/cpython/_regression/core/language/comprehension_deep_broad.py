# comprehensions deep broad

# list comp basic
print([x * 2 for x in range(5)])
print([x for x in range(10) if x % 3 == 0])

# nested list comp
matrix = [[1, 2, 3], [4, 5, 6], [7, 8, 9]]
flat = [n for row in matrix for n in row]
print(flat)

# nested with filter
only_even = [n for row in matrix for n in row if n % 2 == 0]
print(only_even)

# cross product
cross = [(x, y) for x in range(3) for y in range(3)]
print(cross)

# string-comp
vowels = [c for c in "hello world" if c in "aeiou"]
print(vowels)

# dict comp
squares = {x: x * x for x in range(5)}
for k in sorted(squares):
    print(k, squares[k])

# dict comp from pairs (note: using explicit nested)
pairs = [("a", 1), ("b", 2), ("c", 3)]
d = {k: v for k, v in pairs}
print(sorted(d.items()))

# dict comp with condition
evens_sq = {x: x * x for x in range(10) if x % 2 == 0}
for k in sorted(evens_sq):
    print(k, evens_sq[k])

# set comp
letters = {c for c in "banana"}
print(sorted(letters))

# set comp with filter
primes = {n for n in range(2, 20) if all(n % d != 0 for d in range(2, n))}
print(sorted(primes))

# generator expression
g = (x * x for x in range(5))
print(list(g))
print(sum(x for x in range(10)))
print(sum(x * x for x in range(5)))

# comp in function call
print(max(len(s) for s in ["a", "abc", "ab"]))
print(min(abs(x) for x in [3, -5, 1, -1]))

# nested dict comp
nested = {row: {col: row * col for col in range(3)} for row in range(3)}
for r in sorted(nested):
    for c in sorted(nested[r]):
        print(r, c, nested[r][c])

# comprehension with conditional expr
signs = ["pos" if x > 0 else "neg" for x in [1, -2, 3, -4, 5]]
print(signs)
