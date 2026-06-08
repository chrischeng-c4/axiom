# Regression: print([obj]) must call obj.__repr__() for each element.
# Prior to the fix, list/tuple/set repr on user-class instances fell back
# to "{ClassName}()" regardless of the class's __repr__ method.

class Box:
    def __init__(self, v):
        self.v = v
    def __repr__(self):
        return "B(" + str(self.v) + ")"

# list
print([Box(1), Box(2)])

# tuple
print((Box(3), Box(4)))

# nested
print([[Box(5)], (Box(6),)])

# dict value
print({"k": Box(7)})

# print direct instance (should already work)
print(Box(8))

# __str__ still wins over __repr__ for top-level str() / print() of instance
class Named:
    def __init__(self, n):
        self.n = n
    def __str__(self):
        return "str:" + self.n
    def __repr__(self):
        return "repr:" + self.n

# print (top-level) uses __str__
print(Named("a"))
# but in a container, repr is used
print([Named("a"), Named("b")])
