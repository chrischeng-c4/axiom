# Language conformance: descriptor protocol (P2-R3).
# Tests user-defined __get__, __set__, __delete__ descriptors.

# TC-3.1: Non-data descriptor __get__ invoked on attribute read.
class Verbose:
    def __get__(self, obj, objtype):
        print("descriptor get called")
        return 42

class MyClass:
    attr = Verbose()

obj = MyClass()
result = obj.attr
print(result)

# TC-3.2: Data descriptor __set__ enforces validation.
class Validated:
    def __get__(self, obj, objtype):
        # Read the backing value from instance __dict__
        val = obj._val
        return val

    def __set__(self, obj, value):
        if value < 0:
            print("rejected negative")
            return
        obj._val = value

class Item:
    price = Validated()

item = Item()
item._val = 0
item.price = 10
print(item.price)
item.price = 25
print(item.price)
item.price = -5

# TC-3.3: Data descriptor __delete__ clears backing store.
class Deletable:
    def __get__(self, obj, objtype):
        return obj._v

    def __set__(self, obj, value):
        obj._v = value

    def __delete__(self, obj):
        obj._v = 0

class Holder:
    val = Deletable()

h = Holder()
h.val = 99
print(h.val)
del h.val
print(h.val)
