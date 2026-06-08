# unary ops broad

# minus
print(-5)
print(-(-5))
print(-0)

# plus (no-op for int)
print(+5)
print(+(-5))

# negation via expression
x = 10
print(-x)
print(+x)

# bitwise not
print(~0)
print(~1)
print(~-1)
print(~5)
print(~0xff)

# logical not
print(not True)
print(not False)
print(not 0)
print(not 1)
print(not [])
print(not [1])
print(not "")
print(not "x")
print(not None)

# double not
print(not not 1)
print(not not 0)
print(not not [])

# chained unary minus
print(- - 5)
print(- - -5)
print(- - - -5)

# unary on float
print(-1.5)
print(+1.5)
print(abs(-1.5))

# unary in expressions
print(5 + -3)
print(5 - -3)
print(5 * -2)
print(-5 + -3)

# unary with function call
def neg(x):
    return -x

print(neg(5))
print(neg(-5))
print(neg(0))

# unary with compound
print(-(1 + 2))
print(-(3 * 4))
print(-((-1) + 5))

# conditional unary
def pos_neg(x):
    if x >= 0:
        return x
    else:
        return -x

print(pos_neg(5))
print(pos_neg(-5))
print(pos_neg(0))
