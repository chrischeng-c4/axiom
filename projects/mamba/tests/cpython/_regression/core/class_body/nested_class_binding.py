"""Nested classes defined in a class body bind as attributes of the outer class."""


class Outer:
    kind = "outer"

    class Inner:
        flavor = "inner"

        def label(self):
            return "inner:" + self.flavor

    def describe(self):
        return self.kind + "/" + Outer.Inner.flavor


assert Outer.kind == "outer"
assert Outer.Inner.flavor == "inner"
assert Outer().describe() == "outer/inner"
assert Outer.Inner().label() == "inner:inner"

print("nested_class_binding OK")
