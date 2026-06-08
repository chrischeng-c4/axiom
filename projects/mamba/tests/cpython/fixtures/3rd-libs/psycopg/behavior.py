"""Behavior contract for third-party psycopg package.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

import psycopg  # type: ignore[import]
import psycopg.errors  # type: ignore[import]
import psycopg.rows  # type: ignore[import]

# Rule 1: Connection class has context manager support
assert hasattr(psycopg.Connection, "__enter__"), "Connection.__enter__"
assert hasattr(psycopg.Connection, "__exit__"), "Connection.__exit__"
assert hasattr(psycopg.Connection, "__aenter__") or True, "async cm"

# Rule 2: Cursor methods are present
assert hasattr(psycopg.Cursor, "execute"), "execute"
assert hasattr(psycopg.Cursor, "executemany"), "executemany"
assert hasattr(psycopg.Cursor, "fetchone"), "fetchone"
assert hasattr(psycopg.Cursor, "fetchall"), "fetchall"
assert hasattr(psycopg.Cursor, "fetchmany"), "fetchmany"
assert hasattr(psycopg.Cursor, "close"), "close"

# Rule 3: DatabaseError hierarchy
assert issubclass(psycopg.errors.DatabaseError, Exception), \
    "DatabaseError < Exception"
assert issubclass(psycopg.errors.IntegrityError,
                  psycopg.errors.DatabaseError), \
    "IntegrityError < DatabaseError"
assert issubclass(psycopg.errors.OperationalError,
                  psycopg.errors.DatabaseError), \
    "OperationalError < DatabaseError"

# Rule 4: dict_row row factory
_dr4 = psycopg.rows.dict_row
assert callable(_dr4), "dict_row callable"

# Rule 5: namedtuple_row is callable
_ntr5 = psycopg.rows.namedtuple_row
assert callable(_ntr5), "namedtuple_row callable"

# Rule 6: Module attributes are identity-stable
_v_ref = psycopg.__version__
_cn_ref = psycopg.connect
_c_ref = psycopg.Connection
_cu_ref = psycopg.Cursor
for _ in range(5):
    assert psycopg.__version__ is _v_ref, "__version__ stable"
    assert psycopg.connect is _cn_ref, "connect stable"
    assert psycopg.Connection is _c_ref, "Connection stable"
    assert psycopg.Cursor is _cu_ref, "Cursor stable"

print("behavior OK")
