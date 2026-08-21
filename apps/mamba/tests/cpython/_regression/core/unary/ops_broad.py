# unary ops broad

# negation
print(-5)
print(-0)
print(-(-5))
print(-(-(-5)))
print(-100)

# plus (nop)
print(+5)
print(+0)
print(+(-5))
print(+(+5))

# not
print(not True)
print(not False)
print(not 0)
print(not 1)
print(not None)
print(not "")
print(not "hi")
print(not [])
print(not [1])
print(not {})
print(not {"a": 1})

# double not
print(not not True)
print(not not 0)
print(not not 42)

# bitwise not
print(~0)
print(~-1)
print(~1)
print(~10)
print(~-10)

# combinations
x = 5
print(-x)
print(+x)
print(~x)

y = 0
print(-y)

# in expressions
print(-x + 3)
print(-(x + 3))
print(~x + 1)
