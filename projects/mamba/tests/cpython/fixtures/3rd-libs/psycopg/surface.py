"""Surface contract for third-party psycopg package.

# type-regime: monomorphic

Probes: psycopg.__version__, psycopg.connect, psycopg.Connection,
psycopg.Cursor, psycopg.errors, psycopg.rows.
CPython 3.12 is the oracle.
"""

import psycopg  # type: ignore[import]
import psycopg.errors  # type: ignore[import]
import psycopg.rows  # type: ignore[import]

# Core API
assert hasattr(psycopg, "__version__"), "__version__"
assert hasattr(psycopg, "connect"), "connect"
assert hasattr(psycopg, "Connection"), "Connection"
assert hasattr(psycopg, "Cursor"), "Cursor"
assert hasattr(psycopg, "errors"), "errors"
assert hasattr(psycopg, "rows"), "rows"
assert hasattr(psycopg, "AsyncConnection"), "AsyncConnection"
assert hasattr(psycopg, "AsyncCursor"), "AsyncCursor"

# Version
assert isinstance(psycopg.__version__, str), \
    f"version type = {type(psycopg.__version__)!r}"

# Classes are callable
assert callable(psycopg.Connection), "Connection callable"
assert callable(psycopg.Cursor), "Cursor callable"
assert callable(psycopg.connect), "connect callable"

# Connection has expected methods
assert hasattr(psycopg.Connection, "cursor"), "Connection.cursor"
assert hasattr(psycopg.Connection, "commit"), "Connection.commit"
assert hasattr(psycopg.Connection, "rollback"), "Connection.rollback"
assert hasattr(psycopg.Connection, "close"), "Connection.close"
assert hasattr(psycopg.Connection, "execute"), "Connection.execute"

# Cursor has expected methods
assert hasattr(psycopg.Cursor, "execute"), "Cursor.execute"
assert hasattr(psycopg.Cursor, "fetchone"), "Cursor.fetchone"
assert hasattr(psycopg.Cursor, "fetchall"), "Cursor.fetchall"
assert hasattr(psycopg.Cursor, "fetchmany"), "Cursor.fetchmany"

# errors module
assert hasattr(psycopg.errors, "DatabaseError"), "DatabaseError"
assert hasattr(psycopg.errors, "IntegrityError"), "IntegrityError"
assert hasattr(psycopg.errors, "OperationalError"), "OperationalError"

# rows module
assert hasattr(psycopg.rows, "dict_row"), "dict_row"
assert hasattr(psycopg.rows, "tuple_row"), "tuple_row"
assert hasattr(psycopg.rows, "namedtuple_row"), "namedtuple_row"

# Module attributes stable
_v_ref = psycopg.__version__
assert psycopg.__version__ is _v_ref, "__version__ stable"
_cn_ref = psycopg.connect
assert psycopg.connect is _cn_ref, "connect stable"
_c_ref = psycopg.Connection
assert psycopg.Connection is _c_ref, "Connection stable"
_cu_ref = psycopg.Cursor
assert psycopg.Cursor is _cu_ref, "Cursor stable"

print("surface OK")
