# inheritance broad

# single inheritance
class Animal:
    def __init__(self, name):
        self.name = name
    def greet(self):
        return f"Hello from {self.name}"
    def kind(self):
        return "animal"

class Dog(Animal):
    def kind(self):
        return "dog"

class Puppy(Dog):
    def kind(self):
        return "puppy"

a = Animal("Rex")
d = Dog("Rex")
p = Puppy("Rex")
print(a.greet())
print(d.greet())
print(p.greet())
print(a.kind())
print(d.kind())
print(p.kind())

# super() chain
class A:
    def go(self):
        return "A"

class B(A):
    def go(self):
        return super().go() + "+B"

class C(B):
    def go(self):
        return super().go() + "+C"

print(A().go())
print(B().go())
print(C().go())

# __init__ chain
class Parent:
    def __init__(self, x):
        self.x = x

class Child(Parent):
    def __init__(self, x, y):
        super().__init__(x)
        self.y = y

class Grandchild(Child):
    def __init__(self, x, y, z):
        super().__init__(x, y)
        self.z = z

gc = Grandchild(1, 2, 3)
print(gc.x, gc.y, gc.z)

# isinstance / issubclass
print(isinstance(p, Puppy))
print(isinstance(p, Dog))
print(isinstance(p, Animal))
print(isinstance(p, str))
print(issubclass(Puppy, Dog))
print(issubclass(Puppy, Animal))
print(issubclass(Dog, Puppy))

# __init_subclass__ (simple)
# skip: complex hook

# multiple inheritance basic
class Swim:
    def move(self):
        return "swim"

class Fly:
    def move(self):
        return "fly"

class Duck(Swim, Fly):
    pass

print(Duck().move())
