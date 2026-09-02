"""Focused fixtures for the WIS gap reader."""

from __future__ import annotations

from pathlib import Path

from aw.scripts import wis


def test_roadmap_ids_read_near_term_outcomes_not_non_goals(tmp_path: Path) -> None:
    project = "apps/demo"
    roadmap = tmp_path / project / "ROADMAP.md"
    roadmap.parent.mkdir(parents=True)
    roadmap.write_text(
        """# Demo roadmap

## Near-term outcomes

### Future outcome

- ID: `future-outcome`

## Non-goals

### Explicit boundary

- ID: `not-a-future-outcome`
""",
        encoding="utf-8",
    )

    assert wis.roadmap_ids(tmp_path, project) == ["future-outcome"]
