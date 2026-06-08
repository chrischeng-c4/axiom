"""PostgreSQL Full-Text Search support."""
from typing import Optional

__all__ = [
    "FullTextSearch",
    "fts",
]


class FullTextSearch:
    """PostgreSQL full-text search helpers."""

    @staticmethod
    def to_tsvector(column: str, config: str = "english") -> str:
        """Create tsvector for full-text indexing.

        Args:
            column: Column name or expression
            config: Text search configuration (default: english)

        Returns:
            SQL expression for to_tsvector

        Example:
            >>> FullTextSearch.to_tsvector("content")
            "to_tsvector('english', content)"
        """
        return f"to_tsvector('{config}', {column})"

    @staticmethod
    def to_tsquery(query: str, config: str = "english") -> str:
        """Create tsquery for full-text search.

        Args:
            query: Search query string
            config: Text search configuration (default: english)

        Returns:
            SQL expression for to_tsquery

        Example:
            >>> FullTextSearch.to_tsquery("python & database")
            "to_tsquery('english', 'python & database')"
        """
        # Escape single quotes
        escaped_query = query.replace("'", "''")
        return f"to_tsquery('{config}', '{escaped_query}')"

    @staticmethod
    def plainto_tsquery(query: str, config: str = "english") -> str:
        """Create tsquery from plain text.

        Args:
            query: Plain text search query
            config: Text search configuration

        Returns:
            SQL expression for plainto_tsquery
        """
        escaped_query = query.replace("'", "''")
        return f"plainto_tsquery('{config}', '{escaped_query}')"

    @staticmethod
    def match(column: str, query: str, config: str = "english") -> str:
        """Create full-text match expression.

        Args:
            column: Column name or expression
            query: Search query
            config: Text search configuration

        Returns:
            SQL WHERE clause for full-text matching

        Example:
            >>> FullTextSearch.match("content", "python database")
            "to_tsvector('english', content) @@ plainto_tsquery('english', 'python database')"
        """
        tsvector = FullTextSearch.to_tsvector(column, config)
        tsquery = FullTextSearch.plainto_tsquery(query, config)
        return f"{tsvector} @@ {tsquery}"

    @staticmethod
    def rank(column: str, query: str, config: str = "english") -> str:
        """Create ranking expression for search results.

        Args:
            column: Column name or expression
            query: Search query
            config: Text search configuration

        Returns:
            SQL expression for ts_rank
        """
        tsvector = FullTextSearch.to_tsvector(column, config)
        tsquery = FullTextSearch.plainto_tsquery(query, config)
        return f"ts_rank({tsvector}, {tsquery})"


# Convenience alias
fts = FullTextSearch
