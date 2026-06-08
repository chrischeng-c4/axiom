# T1.1: @classmethod receives cls, returns cls.species
# Conformance test: must produce identical output under CPython 3.12 and Mamba.

class Animal:
    species = "unknown"

    @classmethod
    def get_species(cls):
        return cls.species

class Dog(Animal):
    species = "canine"

print(Dog.get_species())     # Expected: canine
print(Animal.get_species())  # Expected: unknown

# Also test calling classmethod on an instance
d = Dog()
print(d.get_species())       # Expected: canine
