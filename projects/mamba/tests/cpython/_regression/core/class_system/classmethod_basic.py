# @classmethod: basic usage, cls access, call on class and instance

class Animal:
    species_count = 0

    def __init__(self, name):
        self.name = name
        Animal.species_count = Animal.species_count + 1

    @classmethod
    def get_count(cls):
        return cls.species_count

# Call on class
print(Animal.get_count())

a = Animal("dog")
b = Animal("cat")

# Call on class after instances
print(Animal.get_count())

# Call on instance
print(a.get_count())
