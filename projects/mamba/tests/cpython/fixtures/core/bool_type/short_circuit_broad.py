# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# boolean short-circuit / truthiness broad

# and short circuit
print(True and True)
print(True and False)
print(False and True)
print(False and False)

# or short circuit
print(True or True)
print(True or False)
print(False or True)
print(False or False)

# chained and/or
print(True and True and True)
print(True and False and True)
print(False or False or True)
print(True or False or False)

# not
print(not True)
print(not False)
print(not 0)
print(not 1)
print(not "")
print(not "x")
print(not [])
print(not [1])
print(not None)

# bool()
print(bool(True))
print(bool(False))
print(bool(0))
print(bool(1))
print(bool(-1))
print(bool(""))
print(bool("x"))
print(bool([]))
print(bool([0]))
print(bool({}))
print(bool({1: 2}))
print(bool(None))
print(bool(0.0))
print(bool(1.5))

# truthy in if
def cls(x):
    if x:
        return "truthy"
    else:
        return "falsy"

print(cls(0))
print(cls(1))
print(cls(-1))
print(cls(100))
print(cls("x"))
print(cls([1]))

# short circuit doesn't eval RHS
called = [0]
def effect():
    called[0] += 1
    return True

# True or X shouldn't call X
_ = True or effect()
print(called[0])
# False and X shouldn't call X
_ = False and effect()
print(called[0])
# False or X calls X
_ = False or effect()
print(called[0])
# True and X calls X
_ = True and effect()
print(called[0])
