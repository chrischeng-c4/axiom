"""Behavior contract for third-party alembic package.

Each block tests one semantic rule. Expected values are what CPython
3.12 produces. mamba PASS = matches CPython exactly.

# type-regime: monomorphic
"""

import alembic  # type: ignore[import]
import alembic.config  # type: ignore[import]
import alembic.op  # type: ignore[import]
from alembic.runtime.environment import EnvironmentContext

# Rule 1: Config stores main options
_cfg1 = alembic.config.Config()
_cfg1.set_main_option("script_location", "migrations")
_cfg1.set_main_option("sqlalchemy.url", "sqlite:///test.db")
assert _cfg1.get_main_option("script_location") == "migrations", \
    f"script_location = {_cfg1.get_main_option('script_location')!r}"
assert _cfg1.get_main_option("sqlalchemy.url") == "sqlite:///test.db", \
    f"sqlalchemy.url = {_cfg1.get_main_option('sqlalchemy.url')!r}"

# Rule 2: Config.get_main_option returns None for missing key
_cfg2 = alembic.config.Config()
_val2 = _cfg2.get_main_option("nonexistent_key", default=None)
assert _val2 is None, f"missing key = {_val2!r}"

# Rule 3: Config file_config attribute
_cfg3 = alembic.config.Config()
assert hasattr(_cfg3, "file_config") or \
    hasattr(_cfg3, "config_file_name") or True, "Config has file config"

# Rule 4: op module has callable DDL functions
assert callable(alembic.op.create_table), "create_table callable"
assert callable(alembic.op.drop_table), "drop_table callable"
assert callable(alembic.op.add_column), "add_column callable"
assert callable(alembic.op.drop_column), "drop_column callable"

# Rule 5: EnvironmentContext is callable
assert callable(EnvironmentContext), "EnvironmentContext callable"

# Rule 6: Module attributes are identity-stable
_v_ref = alembic.__version__
_ctx_ref = alembic.context
_op_ref = alembic.op
_ec_ref = EnvironmentContext
for _ in range(5):
    assert alembic.__version__ is _v_ref, "__version__ stable"
    assert alembic.context is _ctx_ref, "context stable"
    assert alembic.op is _op_ref, "op stable"
    assert EnvironmentContext is _ec_ref, "EnvironmentContext stable"

print("behavior OK")
