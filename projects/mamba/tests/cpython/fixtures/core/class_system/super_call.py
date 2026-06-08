# Class system conformance: super() in single inheritance (R4.3).
# Tests super().__init__ for attribute setup in inheritance chains.

class Animal:
    def __init__(self, name):
        self.name = name

    def speak(self):
        return f"{self.name} says ..."

class Dog(Animal):
    def __init__(self, name, breed):
        super().__init__(name)
        self.breed = breed

    def speak(self):
        return f"{self.name} says woof"

d = Dog("Rex", "Labrador")
print(d.name)
print(d.breed)
print(d.speak())
print(isinstance(d, Dog))
print(isinstance(d, Animal))

# Another subclass
class Cat(Animal):
    def __init__(self, name):
        super().__init__(name)

    def speak(self):
        return f"{self.name} says meow"

c = Cat("Whiskers")
print(c.name)
print(c.speak())

# Deeper chain via super().__init__
class Puppy(Dog):
    def __init__(self, name, breed, toy):
        super().__init__(name, breed)
        self.toy = toy

p = Puppy("Buddy", "Golden", "ball")
print(p.name)
print(p.breed)
print(p.toy)
