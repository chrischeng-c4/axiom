"""Text segmentation, search ranking, HTML/XML, diff, fuzzy matching, and templates."""

try:
    from cclab._text import (
        segment, rank, markup,
        DiffOp, WordDiffOp,
        FuzzyMatch, FuzzySearcher,
        TemplateEngine,
    )
except ImportError:
    pass

__all__ = [
    "segment",
    "rank",
    "markup",
    "DiffOp",
    "WordDiffOp",
    "FuzzyMatch",
    "FuzzySearcher",
    "TemplateEngine",
]
