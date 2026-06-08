# divmod() returns (quotient, remainder) tuple; pow(a,b,c) is modular exp

# divmod — positive
print(divmod(17, 5))
print(divmod(20, 4))

# divmod — negative (Python floor division)
print(divmod(-17, 5))
print(divmod(17, -5))

# pow(a, b) — two-arg
print(pow(2, 10))
print(pow(3, 4))

# pow(a, b, c) — modular exponentiation
print(pow(2, 10, 1000))
print(pow(7, 13, 19))

# pow with negative base
print(pow(-2, 3))

# 0 ** 0
print(pow(0, 0))
