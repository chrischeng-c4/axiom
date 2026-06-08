"""PostgreSQL array operators."""
from typing import List, Any

__all__ = [
    "ArrayOps",
]


class ArrayOps:
    """PostgreSQL array operators and functions."""

    @staticmethod
    def contains(column: str, value: List[Any]) -> str:
        """Array contains (@> operator).

        Args:
            column: Array column name
            value: List of values to check

        Returns:
            SQL WHERE clause

        Example:
            >>> ArrayOps.contains("tags", ["python", "database"])
            "tags @> ARRAY['python', 'database']"
        """
        array_str = ArrayOps._format_array(value)
        return f"{column} @> {array_str}"

    @staticmethod
    def contained_by(column: str, value: List[Any]) -> str:
        """Array contained by (<@ operator).

        Args:
            column: Array column name
            value: List of values

        Returns:
            SQL WHERE clause
        """
        array_str = ArrayOps._format_array(value)
        return f"{column} <@ {array_str}"

    @staticmethod
    def overlap(column: str, value: List[Any]) -> str:
        """Array overlap (&& operator).

        Args:
            column: Array column name
            value: List of values to check overlap

        Returns:
            SQL WHERE clause

        Example:
            >>> ArrayOps.overlap("tags", ["python", "rust"])
            "tags && ARRAY['python', 'rust']"
        """
        array_str = ArrayOps._format_array(value)
        return f"{column} && {array_str}"

    @staticmethod
    def any(column: str, value: Any) -> str:
        """ANY operator for array containment.

        Args:
            column: Array column name
            value: Value to check if in array

        Returns:
            SQL WHERE clause

        Example:
            >>> ArrayOps.any("tags", "python")
            "'python' = ANY(tags)"
        """
        if isinstance(value, str):
            return f"'{value}' = ANY({column})"
        return f"{value} = ANY({column})"

    @staticmethod
    def length(column: str) -> str:
        """Get array length.

        Args:
            column: Array column name

        Returns:
            SQL expression for array_length
        """
        return f"array_length({column}, 1)"

    @staticmethod
    def _format_array(value: List[Any]) -> str:
        """Format Python list as PostgreSQL array literal.

        Args:
            value: Python list

        Returns:
            PostgreSQL ARRAY syntax
        """
        if not value:
            return "ARRAY[]"

        # Check if all elements are strings
        if all(isinstance(v, str) for v in value):
            escaped = [f"'{v.replace(chr(39), chr(39) + chr(39))}'" for v in value]
            return f"ARRAY[{', '.join(escaped)}]"
        else:
            return f"ARRAY[{', '.join(map(str, value))}]"
