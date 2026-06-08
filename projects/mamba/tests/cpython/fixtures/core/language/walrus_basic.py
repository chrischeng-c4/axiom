# Walrus operator (:=) basic usage

# In if condition
if (n := 10) > 5:
    print(f"n is {n}")

# In while loop
items = [1, 2, 3, 0, 4, 5]
i = 0
while (val := items[i]) != 0:
    print(val)
    i = i + 1
print(f"stopped at index {i}")

# In expression
data = [1, 2, 3, 4, 5]
filtered = [y for x in data if (y := x * 2) > 4]
print(filtered)
