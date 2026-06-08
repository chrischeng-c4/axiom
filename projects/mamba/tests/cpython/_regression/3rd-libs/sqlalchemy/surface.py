"""Surface contract for third-party sqlalchemy package.

# type-regime: monomorphic

Probes: sqlalchemy.create_engine, sqlalchemy.Column, sqlalchemy.Integer,
sqlalchemy.String, sqlalchemy.MetaData, sqlalchemy.Table, sqlalchemy.select,
sqlalchemy.insert, sqlalchemy.update, sqlalchemy.delete, sqlalchemy.text.
CPython 3.12 is the oracle.
"""

import sqlalchemy
from sqlalchemy import (
    create_engine, Column, Integer, String, MetaData, Table,
    select, insert, update, delete, text,
)

# Core constructs
assert hasattr(sqlalchemy, "create_engine"), "create_engine"
assert hasattr(sqlalchemy, "Column"), "Column"
assert hasattr(sqlalchemy, "Integer"), "Integer"
assert hasattr(sqlalchemy, "String"), "String"
assert hasattr(sqlalchemy, "MetaData"), "MetaData"
assert hasattr(sqlalchemy, "Table"), "Table"
assert hasattr(sqlalchemy, "select"), "select"
assert hasattr(sqlalchemy, "insert"), "insert"
assert hasattr(sqlalchemy, "update"), "update"
assert hasattr(sqlalchemy, "delete"), "delete"
assert hasattr(sqlalchemy, "text"), "text"
assert hasattr(sqlalchemy, "func"), "func"
assert hasattr(sqlalchemy, "and_"), "and_"
assert hasattr(sqlalchemy, "or_"), "or_"
assert hasattr(sqlalchemy, "not_"), "not_"

# Type system
assert hasattr(sqlalchemy, "Float"), "Float"
assert hasattr(sqlalchemy, "Boolean"), "Boolean"
assert hasattr(sqlalchemy, "DateTime"), "DateTime"
assert hasattr(sqlalchemy, "Text"), "Text"

# create_engine with in-memory SQLite
_engine = create_engine("sqlite://")
assert hasattr(_engine, "connect"), "engine.connect"
assert hasattr(_engine, "execute") or hasattr(_engine, "begin"), "engine exec"
assert hasattr(_engine, "dispose"), "engine.dispose"

# MetaData + Table
_meta = MetaData()
_users = Table(
    "users", _meta,
    Column("id", Integer, primary_key=True),
    Column("name", String),
)
assert _users.name == "users", f"table name = {_users.name!r}"
assert "id" in _users.c, "id column"
assert "name" in _users.c, "name column"

# Create table and insert data
_meta.create_all(_engine)
with _engine.connect() as _conn:
    _conn.execute(insert(_users).values(id=1, name="Alice"))
    _conn.execute(insert(_users).values(id=2, name="Bob"))
    _conn.commit()
    _result = _conn.execute(select(_users).order_by(_users.c.id))
    _rows = _result.fetchall()
    assert len(_rows) == 2, f"row count = {len(_rows)!r}"
    assert _rows[0].name == "Alice", f"first name = {_rows[0].name!r}"

# text() for raw SQL
_t = text("SELECT 1 + 1")
assert hasattr(_t, "__str__") or hasattr(_t, "text"), "text object"

# Module attributes stable
_engine_ref = sqlalchemy.create_engine
assert sqlalchemy.create_engine is _engine_ref, "create_engine stable"

print("surface OK")
