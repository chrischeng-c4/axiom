# Dunder conformance: __bool__ truthiness, fallback to __len__.
class Truthy:
    def __bool__(self):
        return True

class Falsy:
    def __bool__(self):
        return False

class HasLen:
    def __len__(self):
        return 0

class HasLenNonZero:
    def __len__(self):
        return 5

t = Truthy()
f = Falsy()
l0 = HasLen()
l5 = HasLenNonZero()
print(bool(t))
print(bool(f))
print(bool(l0))
print(bool(l5))
if t:
    print('truthy ok')
if not f:
    print('falsy ok')
