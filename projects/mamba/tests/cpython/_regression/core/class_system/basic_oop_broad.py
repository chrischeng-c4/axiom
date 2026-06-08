# class basic oop broad

# simple class
class Point:
    def __init__(self, x, y):
        self.x = x
        self.y = y

    def distance_from_origin(self):
        return (self.x * self.x + self.y * self.y) ** 0.5

    def __str__(self):
        return "(" + str(self.x) + ", " + str(self.y) + ")"

p = Point(3, 4)
print(p.x)
print(p.y)
print(p.distance_from_origin())
print(p)

# inheritance
class Animal:
    def __init__(self, name):
        self.name = name

    def speak(self):
        return "generic sound"

    def identify(self):
        return self.name + ": " + self.speak()

class Dog(Animal):
    def speak(self):
        return "woof"

class Cat(Animal):
    def speak(self):
        return "meow"

d = Dog("Rex")
c = Cat("Whiskers")
print(d.identify())
print(c.identify())

# super()
class Puppy(Dog):
    def speak(self):
        return super().speak() + "!"

pup = Puppy("Buddy")
print(pup.speak())
print(pup.identify())

# class attribute
class Counter:
    count = 0

    def increment(self):
        Counter.count += 1
        return Counter.count

c1 = Counter()
c2 = Counter()
print(c1.increment())
print(c1.increment())
print(c2.increment())
print(Counter.count)

# method with args
class Calculator:
    def __init__(self):
        self.result = 0

    def add(self, x):
        self.result += x
        return self

    def mul(self, x):
        self.result *= x
        return self

calc = Calculator()
print(calc.add(5).add(3).mul(2).result)

# isinstance / issubclass
print(isinstance(d, Dog))
print(isinstance(d, Animal))
print(isinstance(d, Cat))
print(issubclass(Dog, Animal))
print(issubclass(Cat, Animal))
print(issubclass(Puppy, Dog))
print(issubclass(Puppy, Animal))
