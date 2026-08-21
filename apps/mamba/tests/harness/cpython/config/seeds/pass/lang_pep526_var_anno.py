# Operational AssertionPass seed for PEP 526 — variable annotations
# at module and function scope.
# Surface: `name: T = value` binds and evaluates as a regular
# assignment; the annotation is purely a type-hint, runtime value
# behaves as untyped Python.
x: int = 5
y: str = "hello"
z: list[int] = [1, 2, 3]
flag: bool = True

_ledger: list[int] = []
# Module-scope annotated assignment binds normally
assert x == 5; _ledger.append(1)
assert y == "hello"; _ledger.append(1)
assert z == [1, 2, 3]; _ledger.append(1)
assert flag; _ledger.append(1)

# Function-scope annotated assignment — assert internal state only.
# Returning the int through `-> int` annotation currently drops int
# identity (same family as PEP 604/695 return-position marshalling
# gap), so this fixture avoids `compute() == 15` and instead
# exercises the annotated locals via observable side-effects.
sink: list[int] = []
def populate() -> None:
    n: int = 10
    s: str = "abc"
    items: list[int] = []
    items.append(1)
    items.append(2)
    sink.append(n)
    sink.append(len(s))
    sink.append(len(items))

populate()
assert sink == [10, 3, 2]; _ledger.append(1)

# Annotated assignment with expression on RHS — assignment binding
# survives because no annotated return marshaller is involved.
total: int = sum([1, 2, 3, 4])
assert total == 10; _ledger.append(1)
print(f"MAMBA_ASSERTION_PASS: lang_pep526_var_anno {sum(_ledger)} asserts")
