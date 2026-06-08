# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""core/<area>: scope / import error paths (CPython 3.12 oracle)."""


# Access undefined name raises NameError.
try:
    no_such_name_xyzzy  # type: ignore[name-defined]  # noqa: F821
    print("undef: no_raise")
except NameError as e:
    print("undef:", type(e).__name__, str(e)[:60])


# UnboundLocalError: read before write in a local scope.
def f():
    try:
        print(x)
        x = 1  # type: ignore[used-before-def]  # noqa: F841
        return "no_raise"
    except UnboundLocalError as e:
        return f"UnboundLocalError: {str(e)[:40]}"


print("unbound:", f())


# Importing non-existent module.
try:
    __import__("no_such_module_for_import_xyzzy")
    print("missing_import: no_raise")
except ModuleNotFoundError as e:
    print("missing_import:", type(e).__name__, str(e)[:60])


# ModuleNotFoundError IS ImportError.
print("mnfe_is_ie:", issubclass(ModuleNotFoundError, ImportError))
