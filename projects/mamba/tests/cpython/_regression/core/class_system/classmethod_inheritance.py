# Inherited classmethod: subclass gets parent classmethod, cls points to subclass

class Base:
    name = "Base"

    @classmethod
    def who(cls):
        return cls.name

class Child(Base):
    name = "Child"

class GrandChild(Child):
    name = "GrandChild"

print(Base.who())
print(Child.who())
print(GrandChild.who())
