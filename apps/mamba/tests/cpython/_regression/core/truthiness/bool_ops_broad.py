# bool / and / or / not patterns broad

# and basic
print(True and True)
print(True and False)
print(False and True)
print(False and False)

# or basic
print(True or True)
print(True or False)
print(False or True)
print(False or False)

# not
print(not True)
print(not False)

# short-circuit: and returns first falsy or last
print(1 and 2)
print(0 and 2)
print("hi" and "there")

# short-circuit: or returns first truthy or last
print(1 or 2)
print(0 or 2)
print("" or "hello")

# chains
print(1 and 2 and 3)
print(0 or 0 or 3)
print(0 or 2 or 3)

# not with numbers/strings/collections
print(not 0)
print(not 1)
print(not "")
print(not "hi")
print(not [])
print(not [1])
print(not None)

# mixed
print(True and 1)
print(False or "default")
print(None or "fallback")

# bool() conversions
print(bool(1))
print(bool(0))
print(bool("hi"))
print(bool(""))
print(bool([1]))
print(bool([]))
print(bool(None))

# all / any
print(all([True, True, True]))
print(all([True, False, True]))
print(all([]))
print(any([False, False, True]))
print(any([False, False, False]))
print(any([]))
