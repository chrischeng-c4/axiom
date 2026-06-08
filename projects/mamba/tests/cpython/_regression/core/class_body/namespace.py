# mamba-xfail: nested classes inside a class body are not bound correctly.
# `Outer.Inner.flavor` returns `None` (instead of "inner") and the
# enclosing method `Outer().describe()` returns `None` because the
# nested-class lookup fails at class-body execution time, then an
# AttributeError surfaces ("'str' object has no attribute 'Inner'") on
# the subsequent access. Plain data attribute / method / scoping clauses
# already pass; the nested-class clause is the runtime gap that gates
# the xfail.
#
# Class body namespace — #2808.
#
# Covers class-body execution semantics: class attributes (data),
# methods, a nested class reference, and class-body side effects on
# a module-level list. Asserts resulting `cls.__dict__` keys and
# method invocation paths so failure points at namespace scoping bugs
# rather than method dispatch.
#
# Each clause prints with the `[class-body]` tag so failure output
# names the semantic area.

# Module-level side-effect channel that the class body writes to.
seen = []


class Outer:
    # 1. Class attribute (data).
    kind = "outer"

    # 2. Class-body side effect (records the class as it is being built).
    seen.append("Outer-body")

    # 3. Nested class reference. Inside the body, ``Outer`` is not yet
    #    bound; the nested class is built and then referenced via
    #    ``Outer.Inner`` after the class statement completes.
    class Inner:
        flavor = "inner"

        def label(self):
            return "inner:" + self.flavor

    # 4. Method using ``self`` only (no Outer lookup at class-body time).
    def describe(self):
        return self.kind + "/" + Outer.Inner.flavor


print("Outer.kind=", Outer.kind, "[class-body]")
print("Outer.Inner.flavor=", Outer.Inner.flavor, "[class-body: nested]")
print("Outer().describe()=", Outer().describe(), "[class-body: method]")
print("Outer.Inner().label()=", Outer.Inner().label(), "[class-body: nested-method]")

# 5. ``__dict__`` contains the data attribute, the nested class, and
#    the method. Use a stable sorted projection of the relevant keys
#    so the assertion is robust to dict iteration order changes.
keys = sorted(k for k in Outer.__dict__ if not k.startswith("_"))
print("Outer.__dict__ keys=", keys, "[class-body: __dict__]")

# 6. Side-effect channel captured the class-body execution.
print("seen=", seen, "[class-body: side-effect]")

# 7. Class-local name shadowing: a local name inside the class body
#    does NOT leak into the method's enclosing scope (no implicit
#    closure). The method sees the *class attribute* through ``self``,
#    not the body-local.
class Shadow:
    x = 1

    def get(self):
        return self.x


s = Shadow()
print("Shadow().get()=", s.get(), "[class-body: scoping]")
Shadow.x = 9
print("after Shadow.x=9, get()=", s.get(), "[class-body: scoping]")
