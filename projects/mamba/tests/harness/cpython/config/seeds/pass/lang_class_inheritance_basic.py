# Operational AssertionPass seed for basic class-and-inheritance
# surfaces beyond lang_super_inheritance / lang_multiple_inheritance.
# Surface: a subclass inherits an undeclared parent method; a
# subclass overrides a method on its own instance; a grand-child
# class can reach the original via super().method(); isinstance
# walks the inheritance chain; issubclass walks the chain in both
# directions; type(obj).__name__ reflects the runtime class; __mro__
# enumerates the linearised method-resolution order.
_ledger: list[int] = []


class _Animal:
    def greet(self):
        return "Animal"
    def kind(self):
        return "animal"


class _Dog(_Animal):
    def greet(self):
        return "Dog"


class _Puppy(_Dog):
    def greet(self):
        # super().method() reaches the parent class' implementation
        return "Puppy:" + super().greet()


a = _Animal()
d = _Dog()
p = _Puppy()

# Each instance dispatches to its own class' method
assert a.greet() == "Animal"; _ledger.append(1)
assert d.greet() == "Dog"; _ledger.append(1)
# super() chains: Puppy.greet calls Dog.greet which returns "Dog"
assert p.greet() == "Puppy:Dog"; _ledger.append(1)

# A subclass inherits any method it doesn't override
assert d.kind() == "animal"; _ledger.append(1)
assert p.kind() == "animal"; _ledger.append(1)

# isinstance walks up the inheritance chain
assert isinstance(d, _Dog) == True; _ledger.append(1)
assert isinstance(d, _Animal) == True; _ledger.append(1)
assert isinstance(p, _Puppy) == True; _ledger.append(1)
assert isinstance(p, _Dog) == True; _ledger.append(1)
assert isinstance(p, _Animal) == True; _ledger.append(1)
# Parent is NOT an instance of its child class
assert isinstance(a, _Dog) == False; _ledger.append(1)
assert isinstance(a, _Puppy) == False; _ledger.append(1)
assert isinstance(d, _Puppy) == False; _ledger.append(1)

# issubclass walks the chain — child is a subclass of every ancestor
assert issubclass(_Dog, _Animal) == True; _ledger.append(1)
assert issubclass(_Puppy, _Dog) == True; _ledger.append(1)
assert issubclass(_Puppy, _Animal) == True; _ledger.append(1)
# Parent is NOT a subclass of its child
assert issubclass(_Animal, _Dog) == False; _ledger.append(1)
assert issubclass(_Dog, _Puppy) == False; _ledger.append(1)
# Every class is a subclass of itself
assert issubclass(_Dog, _Dog) == True; _ledger.append(1)
assert issubclass(_Animal, _Animal) == True; _ledger.append(1)

# type(obj).__name__ reflects the runtime class name
assert type(d).__name__ == "_Dog"; _ledger.append(1)
assert type(p).__name__ == "_Puppy"; _ledger.append(1)
assert type(a).__name__ == "_Animal"; _ledger.append(1)

# __mro__ linearises the inheritance chain: most-derived first,
# then each ancestor, then `object`
mro_names = [cls.__name__ for cls in _Puppy.__mro__]
assert mro_names == ["_Puppy", "_Dog", "_Animal", "object"]; _ledger.append(1)
# A class with no explicit parent still has object in its MRO
assert _Animal.__mro__[-1].__name__ == "object"; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: lang_class_inheritance_basic {sum(_ledger)} asserts")
