# type() 3-arg dynamic class creation (#974)

# Basic class creation with attributes
MyClass = type('MyClass', (object,), {'x': 42, 'y': 'hello'})
obj = MyClass()
print(obj.x)
print(obj.y)

# type() returns a type object with correct __name__
print(type(obj).__name__)

# isinstance works
print(isinstance(obj, MyClass))

# Class with no extra attributes
Empty = type('Empty', (object,), {})
e = Empty()
print(type(e).__name__)

# Verify issubclass
print(issubclass(MyClass, object))
