# comparison mixed types broad

# int vs int
print(1 < 2)
print(2 < 1)
print(1 == 1)
print(1 != 2)
print(1 <= 1)
print(2 >= 2)

# int vs float
print(1 < 2.5)
print(1 == 1.0)
print(3.14 >= 3)

# float vs float
print(1.5 < 2.5)
print(3.14 == 3.14)
print(0.0 == 0.0)

# str vs str
print("abc" < "abd")
print("abc" == "abc")
print("abc" != "abd")
print("b" > "a")
print("ab" < "abc")

# bool vs int
print(True == 1)
print(False == 0)
print(True < 2)

# chained
print(1 < 2 < 3)
print(1 < 2 > 1)
print(1 == 1 == 1)

# is / is not
x = [1, 2]
y = x
print(x is y)

# None
print(None is None)
a = None
print(a is None)
