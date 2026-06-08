# Operational AssertionPass seed for `super()` and single-base
# inheritance method-resolution.
# Surface: super().__init__(...) delegates to the parent constructor;
# super().method() delegates to the parent's method body even when the
# subclass also defines a same-named method; the parent's return value
# is usable as a normal expression.
class Animal:
    def __init__(self, name):
        self.name = name

    def speak(self):
        return "..."

class Dog(Animal):
    def __init__(self, name, breed):
        super().__init__(name)
        self.breed = breed

    def speak(self):
        parent = super().speak()
        return parent + " woof"

class GoldenRetriever(Dog):
    def __init__(self, name):
        super().__init__(name, "Golden Retriever")

    def speak(self):
        return super().speak() + " (golden)"

_ledger: list[int] = []
d = Dog("Rex", "Lab")
# super().__init__ delegated to the parent constructor
assert d.name == "Rex"; _ledger.append(1)
# subclass __init__ also set its own attribute
assert d.breed == "Lab"; _ledger.append(1)
# super().speak() result is usable inside the override
assert d.speak() == "... woof"; _ledger.append(1)
# Three-level chain: GoldenRetriever -> Dog -> Animal
g = GoldenRetriever("Buddy")
assert g.name == "Buddy"; _ledger.append(1)
assert g.breed == "Golden Retriever"; _ledger.append(1)
# speak chains through both supers
assert g.speak() == "... woof (golden)"; _ledger.append(1)
# isinstance walks the chain
assert isinstance(g, Dog); _ledger.append(1)
assert isinstance(g, Animal); _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: lang_super_inheritance {sum(_ledger)} asserts")
