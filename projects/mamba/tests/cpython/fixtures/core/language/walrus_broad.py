# walrus operator broad

# in if
if (n := 10) > 5:
    print(f"got {n}")

# in while
data = [1, 2, 3, 4, 5]
i = 0
while (val := data[i] if i < len(data) else None) is not None:
    print(val)
    i += 1

# in list comp filter
nums = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
print([x for x in nums if (s := x * x) > 20])

# in elif chain
x = 15
if (lo := x // 10) == 0:
    print("small")
elif lo == 1:
    print(f"teen: {lo}")
else:
    print(f"big: {lo}")

# combined
def find_first(items, pred):
    if (matches := [x for x in items if pred(x)]):
        return matches[0]
    return None

print(find_first([1, 2, 3, 4, 5], lambda x: x > 3))
print(find_first([1, 2], lambda x: x > 100))

# repeated use
def countup():
    i = 0
    results = []
    while (i := i + 1) <= 5:
        results.append(i)
    return results

print(countup())
