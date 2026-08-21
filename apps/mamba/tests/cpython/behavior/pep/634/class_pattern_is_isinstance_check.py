# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "pep"
# lib = "634"
# dimension = "behavior"
# case = "class_pattern_is_isinstance_check"
# subject = "match.class_pattern"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""match.class_pattern: a class pattern is an isinstance check; a subclass matches the base-class pattern"""

# A class pattern is an isinstance check; subclasses match the base pattern.
class Animal:
    pass


class Dog(Animal):
    pass


def is_animal(x):
    match x:
        case Animal():
            return True
    return False


assert is_animal(Dog()) is True
assert is_animal(object()) is False
print("class_pattern_is_isinstance_check OK")
