# truthy/falsy evaluation broad (avoiding known mamba bugs with empty collections)

# bool() constructor
print(bool(1))
print(bool(0))
print(bool(-1))
print(bool(100))

# bool of float
print(bool(1.0))
print(bool(0.0))
print(bool(-3.14))

# bool of str
print(bool("x"))
print(bool("hello"))
print(bool("0"))  # non-empty is True

# bool of list
print(bool([1]))
print(bool([0]))
print(bool([1, 2, 3]))

# bool of tuple
print(bool((1,)))
print(bool((0,)))

# bool of set
print(bool({1}))
print(bool({0}))

# bool of dict
print(bool({"a": 1}))
print(bool({0: 0}))

# bool of None
print(bool(None))

# bool of True/False
print(bool(True))
print(bool(False))

# conversion consistency
print(int(True))
print(int(False))
print(bool(1) == True)
print(bool(0) == False)

# and/or truth propagation
print(1 and 2)
print(2 and 1)
print(0 and 99)
print(99 and 0)
print(1 or 2)
print(2 or 1)
print(0 or 99)
print(99 or 0)

# and/or on non-bool
print("x" and "y")
print("" and "y")

# not
print(not True)
print(not False)
print(not 1)
print(not 0)

# double not
print(not not 1)
print(not not 0)

# nested and/or
print(True and (False or True))
print(False or (True and True))
print((True and False) or True)
print((False or True) and True)

# in if
if 1:
    print("if 1")
if 100:
    print("if 100")
if not 0:
    print("if not 0")
if "x":
    print("if x")
if [1]:
    print("if [1]")

# int in bool context
n = 10
if n:
    print("n truthy")
n = 0
if not n:
    print("n zero")

# list non-empty
li = [1, 2, 3]
if li:
    print("li non-empty")
if len(li) > 0:
    print("len > 0")
