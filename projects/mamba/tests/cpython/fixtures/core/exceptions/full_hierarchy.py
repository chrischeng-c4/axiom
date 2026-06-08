# /// script
# requires-python = ">=3.12"
# dependencies = []
# ///
# Full exception hierarchy: catch, isinstance, inheritance (#755)

# 1. ValueError catch and isinstance
try:
    raise ValueError("bad")
except ValueError as e:
    print("ValueError caught")
    print(isinstance(e, ValueError))
    print(isinstance(e, Exception))
    print(isinstance(e, BaseException))

# 2. TypeError
try:
    raise TypeError("wrong type")
except TypeError as e:
    print("TypeError caught")
    print(isinstance(e, TypeError))
    print(isinstance(e, Exception))

# 3. IndexError is subclass of LookupError
try:
    raise IndexError("out of range")
except LookupError as e:
    print("IndexError caught as LookupError")
    print(isinstance(e, IndexError))
    print(isinstance(e, LookupError))

# 4. KeyError is subclass of LookupError
try:
    raise KeyError("missing key")
except LookupError as e:
    print("KeyError caught as LookupError")
    print(isinstance(e, KeyError))
    print(isinstance(e, LookupError))

# 5. AttributeError
try:
    raise AttributeError("no attr")
except AttributeError as e:
    print("AttributeError caught")
    print(isinstance(e, AttributeError))

# 6. RuntimeError
try:
    raise RuntimeError("runtime")
except RuntimeError as e:
    print("RuntimeError caught")
    print(isinstance(e, RuntimeError))

# 7. NotImplementedError is subclass of RuntimeError
try:
    raise NotImplementedError("not impl")
except RuntimeError as e:
    print("NotImplementedError caught as RuntimeError")
    print(isinstance(e, NotImplementedError))
    print(isinstance(e, RuntimeError))

# 8. ZeroDivisionError is subclass of ArithmeticError
try:
    raise ZeroDivisionError("division by zero")
except ArithmeticError as e:
    print("ZeroDivisionError caught as ArithmeticError")
    print(isinstance(e, ZeroDivisionError))
    print(isinstance(e, ArithmeticError))

# 9. FileNotFoundError is subclass of OSError
try:
    raise FileNotFoundError("no file")
except OSError as e:
    print("FileNotFoundError caught as OSError")
    print(isinstance(e, FileNotFoundError))
    print(isinstance(e, OSError))

# 10. ImportError
try:
    raise ImportError("bad import")
except ImportError as e:
    print("ImportError caught")
    print(isinstance(e, ImportError))

# 11. NameError
try:
    raise NameError("undefined")
except NameError as e:
    print("NameError caught")
    print(isinstance(e, NameError))

# 12. OverflowError is subclass of ArithmeticError
try:
    raise OverflowError("too big")
except ArithmeticError as e:
    print("OverflowError caught as ArithmeticError")
    print(isinstance(e, OverflowError))
    print(isinstance(e, ArithmeticError))

# 13. RecursionError is subclass of RuntimeError
try:
    raise RecursionError("too deep")
except RuntimeError as e:
    print("RecursionError caught as RuntimeError")
    print(isinstance(e, RecursionError))
    print(isinstance(e, RuntimeError))

# 14. AssertionError
try:
    raise AssertionError("assertion failed")
except AssertionError as e:
    print("AssertionError caught")
    print(isinstance(e, AssertionError))
    print(isinstance(e, Exception))

# 15. StopIteration
try:
    raise StopIteration()
except StopIteration as e:
    print("StopIteration caught")
    print(isinstance(e, StopIteration))
    print(isinstance(e, Exception))