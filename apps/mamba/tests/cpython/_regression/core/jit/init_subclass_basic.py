# RUN: jit
# EXPECT: 42
# Integration fixture: class with __init_subclass__ compiles and runs.

class Base:
    def __init_subclass__(cls: int) -> None:
        pass

class Child(Base):
    pass

def f() -> int:
    return 42

f()
