# comparison broad

# int vs int
print(1 < 2)
print(2 < 1)
print(1 == 1)
print(1 != 2)
print(1 >= 1)
print(1 <= 1)

# chained
print(1 < 2 < 3)
print(1 < 2 < 3 < 4)
print(1 < 3 < 2)
print(5 >= 5 >= 5)
print(1 == 1 == 1)
print(1 == 1 != 2)

# float vs int
print(1.0 == 1)
print(1.5 > 1)
print(1.5 < 2)

# negative
print(-1 < 0)
print(-1 > -2)

# zero
print(0 == 0)
print(-0 == 0)
print(0.0 == 0)

# string
print("a" < "b")
print("abc" < "abd")
print("abc" == "abc")
print("" == "")
print("a" < "aa")

# tuple
print((1, 2) < (1, 3))
print((1, 2) == (1, 2))
print((1, 2) < (2, 0))

# list
print([1, 2] < [1, 3])
print([1, 2] == [1, 2])
print([1, 2, 3] < [1, 2, 3, 4])

# min/max result type preserved
x = min(1, 2)
print(x + 1)
y = max(1.5, 2.0)
print(y + 1)

# boolean is int
print(True == 1)
print(False == 0)
print(True + True)
