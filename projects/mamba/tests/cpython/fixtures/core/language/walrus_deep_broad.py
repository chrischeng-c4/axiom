# walrus deep broad

# if walrus
if (n := 10) > 5:
    print(n)

# while walrus — consume input-like iterator
values = [1, 2, 3, 4, 5]
i = 0
while (v := values[i]) < 4:
    print(v)
    i += 1
print("stopped at", v)

# walrus in list comp filter
nums = [1, -2, 3, -4, 5]
pos = [y for x in nums if (y := x) > 0]
print(pos)

# walrus in elif chain
def classify(x):
    if (a := x) < 0:
        return "neg"
    elif (b := x) == 0:
        return "zero"
    elif (c := x) < 10:
        return "small"
    else:
        return "big"

print(classify(-5))
print(classify(0))
print(classify(5))
print(classify(100))

# walrus with len
data = "hello world"
if (length := len(data)) > 5:
    print("long:", length)

# walrus reuse
numbers = [3, 1, 4, 1, 5, 9, 2, 6]
if (m := max(numbers)) > 5:
    print("max:", m)

# walrus with arithmetic
if (s := 2 + 3) == 5:
    print("sum:", s)
