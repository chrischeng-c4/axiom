# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""core/exception_group: EG/BEG type-promotion and subclass wrapping rules (3.12)."""


# ExceptionGroup wrapping only non-base exceptions stays an ExceptionGroup.
eg = ExceptionGroup("eg", [ValueError(1), TypeError(2)])
assert type(eg) is ExceptionGroup

# BaseExceptionGroup downgrades to ExceptionGroup when all leaves are Exceptions.
beg = BaseExceptionGroup("beg", [ValueError(1), TypeError(2)])
assert type(beg) is ExceptionGroup

# BaseExceptionGroup stays a BaseExceptionGroup when a leaf is a BaseException.
beg = BaseExceptionGroup("beg", [ValueError(1), KeyboardInterrupt(2)])
assert type(beg) is BaseExceptionGroup

# Plain ExceptionGroup refuses to nest a BaseException leaf.
try:
    ExceptionGroup("eg", [ValueError(1), KeyboardInterrupt(2)])
    raise AssertionError("expected TypeError")
except TypeError as e:
    assert "Cannot nest BaseExceptions in an ExceptionGroup" in str(e), e


# An ExceptionGroup subclass still refuses BaseException leaves.
class MyEG(ExceptionGroup):
    pass


assert type(MyEG("eg", [ValueError(12), TypeError(42)])) is MyEG
try:
    MyEG("eg", [ValueError(12), KeyboardInterrupt(42)])
    raise AssertionError("expected TypeError")
except TypeError as e:
    assert "Cannot nest BaseExceptions in 'MyEG'" in str(e), e


# A BaseExceptionGroup subclass wraps anything, including BaseExceptions.
class MyBEG(BaseExceptionGroup):
    pass


assert type(MyBEG("eg", [ValueError(12), TypeError(42)])) is MyBEG
assert type(MyBEG("eg", [ValueError(12), KeyboardInterrupt(42)])) is MyBEG


# A class deriving from BOTH the group and a concrete Exception wraps any
# non-base exception without downgrading its type.
class DualEG(ExceptionGroup, ValueError):
    pass


dual = DualEG("eg", [ValueError(12), Exception()])
assert type(dual) is DualEG
assert isinstance(dual, ValueError)

print("subclass_creation OK")
