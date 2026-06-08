# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""AttributeError.name/.obj and NameError.name attributes (CPython 3.12 oracle)."""

# AttributeError carries optional name/obj keyword fields.
exc = AttributeError("Ouch!")
assert exc.name is None
assert exc.obj is None

sentinel = object()
exc = AttributeError("Ouch", name="carry", obj=sentinel)
assert exc.name == "carry"
assert exc.obj is sentinel
print("attribute_error_kwargs: name+obj carried")


# A failed attribute access populates .name and .obj automatically.
class Holder:
    blech = None


holder = Holder()
try:
    holder.bluch
except AttributeError as e:
    assert e.name == "bluch"
    assert e.obj is holder
    print("attribute_access_miss: name=", e.name, "obj_is_holder=", e.obj is holder)


# Same for a missing method call.
class WithMethod:
    def blech(self):
        return None


wm = WithMethod()
try:
    wm.bluch()
except AttributeError as e:
    assert e.name == "bluch"
    assert e.obj is wm
    print("method_miss: name=", e.name)


# NameError populates .name with the unresolved identifier.
try:
    bluch  # noqa: F821
except NameError as e:
    assert e.name == "bluch"
    print("name_error: name=", e.name)


# Referencing the class body name before assignment raises NameError.
def make_class():
    class TestClass:
        TestClass  # noqa: F821  references the name mid-definition

    return TestClass


try:
    make_class()
    raise AssertionError("expected NameError")
except NameError:
    print("class_body_name: NameError raised")

print("attribute_name_obj OK")
