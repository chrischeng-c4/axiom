# T3.1: getattr returns existing attribute value
# Conformance test: must produce identical output under CPython 3.12 and Mamba.

class Box:
    pass

b = Box()
setattr(b, 'size', 10)
print(getattr(b, 'size'))     # Expected: 10

setattr(b, 'color', 'red')
print(getattr(b, 'color'))    # Expected: red

# getattr with default (3-arg form)
print(getattr(b, 'size', 0))    # Expected: 10 (attr exists)
print(getattr(b, 'weight', 99)) # Expected: 99 (attr missing, returns default)

# delattr
delattr(b, 'size')
print(hasattr(b, 'size'))     # Expected: False
print(hasattr(b, 'color'))    # Expected: True
