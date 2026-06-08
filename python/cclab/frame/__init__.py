"""DataFrame and Series with pandas-compatible API."""

try:
    from cclab._frame import (
        DataFrame, Series, GroupBy,
        Expanding, Ewm, Rolling,
        MultiIndex, Workbook,
        read_csv_file,
    )
except ImportError:
    pass

__all__ = [
    "DataFrame",
    "Series",
    "GroupBy",
    "Expanding",
    "Ewm",
    "Rolling",
    "MultiIndex",
    "Workbook",
    "read_csv_file",
]
