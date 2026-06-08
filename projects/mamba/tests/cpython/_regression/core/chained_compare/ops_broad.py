# chained compare ops broad

# simple ascending
print(1 < 2 < 3)
print(1 < 2 < 2)
print(1 < 3 < 2)

# descending
print(3 > 2 > 1)
print(3 > 2 > 3)

# equal in chain
print(1 <= 1 <= 1)
print(1 <= 2 <= 3)
print(3 >= 2 >= 1)
print(1 >= 1 >= 1)

# mixed <, <=
print(1 < 2 <= 3)
print(1 <= 1 < 2)
print(1 < 1 <= 2)

# with equality
print(1 == 1 == 1)
print(1 == 1 == 2)
print(0 == 0 != 1)
print(1 != 2 != 1)

# four-way chain
print(1 < 2 < 3 < 4)
print(1 < 2 < 2 < 4)
print(1 < 2 < 3 > 2)

# with variables
a = 5
b = 10
c = 15
print(a < b < c)
print(a < c < b)
print(a <= b <= c)
print(a == a == a)

# in chain conditions
for x in [1, 5, 10]:
    if 0 < x < 10:
        print(x, "in range")
    else:
        print(x, "out of range")

# empty-ish
print(0 < 0 < 0)
print(0 <= 0 <= 0)
print(-1 < 0 < 1)

# with bools
print(True < 2)
print(False < True < 2)
