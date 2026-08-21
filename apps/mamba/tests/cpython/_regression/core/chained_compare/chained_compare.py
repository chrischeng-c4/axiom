# Chained comparisons: a < b < c
print(1 < 2 < 3)
print(1 < 3 < 2)
print(1 < 2 < 2)

# With ==
print(1 <= 2 <= 2)
print(1 == 1 == 1)
print(1 == 1 == 2)

# Three-way chains
print(1 < 2 < 3 < 4)
print(1 < 2 < 3 < 1)
print(0 <= 0 <= 0 <= 0)

# Mixed operators
print(1 < 2 <= 2 < 3)
print(1 < 2 >= 2 < 3)
print(1 <= 1 == 1 <= 1)

# Short-circuit: b evaluated only once
x = 0
def inc():
    global x
    x = x + 1
    return x

print(0 < inc() < 10)
print(x)
