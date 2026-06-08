"""Behavior contract for third-party sqlalchemy package.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

import sqlalchemy  # type: ignore[import]
from sqlalchemy import (  # type: ignore[import]
    create_engine, Column, Integer, String, MetaData, Table,
    select, insert, update, delete, text,
)

# Rule 1: In-memory SQLite supports create/insert/select
_engine1 = create_engine("sqlite://")
_meta1 = MetaData()
_items1 = Table("items", _meta1,
    Column("id", Integer, primary_key=True),
    Column("name", String),
    Column("price", sqlalchemy.Float),
)
_meta1.create_all(_engine1)
with _engine1.connect() as _c1:
    _c1.execute(insert(_items1).values(id=1, name="Apple", price=0.99))
    _c1.execute(insert(_items1).values(id=2, name="Banana", price=0.59))
    _c1.commit()
    _rows1 = _c1.execute(select(_items1).order_by(_items1.c.id)).fetchall()
assert len(_rows1) == 2, f"row count = {len(_rows1)!r}"
assert _rows1[0].name == "Apple", f"first = {_rows1[0].name!r}"
assert abs(_rows1[1].price - 0.59) < 1e-6, f"price = {_rows1[1].price!r}"

# Rule 2: UPDATE changes rows; SELECT reflects changes
_engine2 = create_engine("sqlite://")
_meta2 = MetaData()
_tbl2 = Table("t", _meta2, Column("id", Integer), Column("val", String))
_meta2.create_all(_engine2)
with _engine2.connect() as _c2:
    _c2.execute(insert(_tbl2).values(id=1, val="old"))
    _c2.execute(update(_tbl2).where(_tbl2.c.id == 1).values(val="new"))
    _c2.commit()
    _r2 = _c2.execute(select(_tbl2.c.val).where(_tbl2.c.id == 1)).scalar()
    assert _r2 == "new", f"updated val = {_r2!r}"

# Rule 3: DELETE removes rows
_engine3 = create_engine("sqlite://")
_meta3 = MetaData()
_tbl3 = Table("t", _meta3, Column("id", Integer), Column("name", String))
_meta3.create_all(_engine3)
with _engine3.connect() as _c3:
    for _n in ["A", "B", "C"]:
        _c3.execute(insert(_tbl3).values(name=_n))
    _c3.execute(delete(_tbl3).where(_tbl3.c.name == "B"))
    _c3.commit()
    _count3 = _c3.execute(select(sqlalchemy.func.count()).select_from(_tbl3)).scalar()
    assert _count3 == 2, f"count after delete = {_count3!r}"

# Rule 4: text() allows raw SQL execution
_engine4 = create_engine("sqlite://")
with _engine4.connect() as _c4:
    _r4 = _c4.execute(text("SELECT 2 + 3")).scalar()
    assert _r4 == 5, f"raw SQL = {_r4!r}"

# Rule 5: select() returns ordered results with ORDER BY
_engine5 = create_engine("sqlite://")
_meta5 = MetaData()
_tbl5 = Table("t", _meta5, Column("n", Integer))
_meta5.create_all(_engine5)
with _engine5.connect() as _c5:
    for _v in [3, 1, 4, 1, 5]:
        _c5.execute(insert(_tbl5).values(n=_v))
    _c5.commit()
    _rows5 = [_r[0] for _r in
              _c5.execute(select(_tbl5.c.n).order_by(_tbl5.c.n)).fetchall()]
assert _rows5 == sorted(_rows5), f"ordered = {_rows5!r}"

print("behavior OK")
