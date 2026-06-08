# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
"""core/exception_group: class hierarchy and generic-alias subscription (3.12)."""

import types

# Hierarchy: ExceptionGroup is both an Exception and a BaseExceptionGroup;
# BaseExceptionGroup is a BaseException.
assert issubclass(ExceptionGroup, Exception)
assert issubclass(ExceptionGroup, BaseExceptionGroup)
assert issubclass(BaseExceptionGroup, BaseException)
assert not issubclass(BaseExceptionGroup, Exception)

# Both group types are subscriptable, producing a types.GenericAlias.
assert isinstance(ExceptionGroup[OSError], types.GenericAlias)
assert isinstance(BaseExceptionGroup[ValueError], types.GenericAlias)

# A plain exception class is not a generic type.
try:
    Exception[OSError]
    raise AssertionError("expected TypeError")
except TypeError as e:
    assert "Exception" in str(e), e
    assert "not subscriptable" in str(e), e

print("type_hierarchy OK")
