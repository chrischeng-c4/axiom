# dunder / protocol broad

# __eq__ / __ne__
class Pair:
    def __init__(self, a, b):
        self.a = a
        self.b = b
    def __eq__(self, other):
        if not isinstance(other, Pair):
            return False
        return self.a == other.a and self.b == other.b
    def __hash__(self):
        return hash((self.a, self.b))
    def __repr__(self):
        return f"Pair({self.a}, {self.b})"

p1 = Pair(1, 2)
p2 = Pair(1, 2)
p3 = Pair(1, 3)
print(p1 == p2)
print(p1 == p3)
print(p1 != p3)
print(p1 == "hi")

# __lt__ for sorting
class Weight:
    def __init__(self, kg):
        self.kg = kg
    def __lt__(self, other):
        return self.kg < other.kg
    def __str__(self):
        return f"{self.kg}kg"
    def __repr__(self):
        return f"{self.kg}kg"

w10 = Weight(10)
w20 = Weight(20)
w30 = Weight(30)
print(w10 < w20)
print(w20 < w10)
print(w20 < w30)

# __getitem__
class Alphabet:
    def __getitem__(self, i):
        return chr(ord('a') + i)

a = Alphabet()
print(a[0])
print(a[25])
print(a[13])

# __len__ used by bool
class Empty:
    def __len__(self):
        return 0

class Full:
    def __len__(self):
        return 5

print(bool(Empty()))
print(bool(Full()))

# __str__ vs __repr__
class Dual:
    def __str__(self):
        return "str form"
    def __repr__(self):
        return "repr form"

d = Dual()
print(str(d))
print(repr(d))
print(d)
