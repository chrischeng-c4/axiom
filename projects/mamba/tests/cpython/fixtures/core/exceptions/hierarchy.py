# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Built-in exception hierarchy
print(issubclass(Exception, BaseException))
print(issubclass(ValueError, Exception))
print(issubclass(TypeError, Exception))
print(issubclass(KeyError, LookupError))
print(issubclass(IndexError, LookupError))
print(issubclass(ZeroDivisionError, ArithmeticError))
print(issubclass(FileNotFoundError, OSError))
print(issubclass(StopIteration, Exception))
print(issubclass(RuntimeError, Exception))
print(issubclass(NameError, Exception))
print(issubclass(AttributeError, Exception))
# Not subclass
print(issubclass(ValueError, TypeError))
print(issubclass(KeyError, ValueError))