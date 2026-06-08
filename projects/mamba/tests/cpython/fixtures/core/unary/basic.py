# Unary operators: -, +, ~ (from CPython test_unary.py)
# test_negative
print(-2 == 0 - 2)
print(-0)
print(--2)
print(-2.0 == 0 - 2.0)

# test_positive
print(+2)
print(+0)
print(++2)
print(+2.0)

# test_invert
print(~2 == -(2+1))
print(~0)
print(~~2)

# test_negation_of_exponentiation
print(-2 ** 3)
print((-2) ** 3)
print(-2 ** 4)
print((-2) ** 4)
