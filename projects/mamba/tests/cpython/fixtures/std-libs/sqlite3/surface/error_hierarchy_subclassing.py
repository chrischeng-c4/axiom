# /// script
# requires-python = ">=3.12"
# dependencies = []
#
# [tool.mamba]
# bucket = "std-libs"
# lib = "sqlite3"
# dimension = "surface"
# case = "error_hierarchy_subclassing"
# subject = "sqlite3.Error"
# kind = "semantic"
# xfail = ""
# mem_carveout = ""
# source = ""
# status = "filled"
# ///
"""sqlite3.Error: the documented exception hierarchy holds: Error subclasses Exception; DatabaseError/OperationalError/IntegrityError/ProgrammingError/DataError/InterfaceError all subclass Error; the *Error DB-API exceptions subclass DatabaseError"""
import sqlite3

assert issubclass(sqlite3.Error, Exception), "Error is an Exception"
for exc in (
    sqlite3.DatabaseError,
    sqlite3.OperationalError,
    sqlite3.IntegrityError,
    sqlite3.ProgrammingError,
    sqlite3.DataError,
    sqlite3.InterfaceError,
):
    assert issubclass(exc, sqlite3.Error), f"{exc.__name__} subclasses Error"

# The DB-API "*Error" exceptions are all DatabaseError subclasses.
for exc in (
    sqlite3.OperationalError,
    sqlite3.IntegrityError,
    sqlite3.ProgrammingError,
):
    assert issubclass(exc, sqlite3.DatabaseError), f"{exc.__name__} subclasses DatabaseError"

print("error_hierarchy_subclassing OK")
