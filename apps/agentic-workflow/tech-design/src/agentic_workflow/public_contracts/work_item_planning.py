"""Public TD boundary for work-item planning and authoring."""

__aw_artifact_id__ = "artifact:work-item-planning/public-contract"
__aw_public_contract__ = True


def terminology_first_four_type_wi_taxonomy() -> str:
    return (
        "typed work items round-trip through authoring and graph projection, "
        "each type has one bounded authoring profile, spikes terminate with "
        "decision evidence or expiry, reports terminate through typed triage, "
        "shared CLI issue intake creates typed reports, health observes report "
        "and spike intake state, and CLI plus docs share one vocabulary"
    )


def canonical_change_issue_cache_round_trip() -> str:
    return "canonical change identity survives issue-cache rehydration"


def wi_create_help_smoke() -> str:
    return "work-item create help exposes the complete typed authoring surface"


def wi_create_remote_flag_tests() -> str:
    return "remote work-item authoring preserves typed flags"


def wi_remove_agent_estimate_unit_command() -> str:
    return "removed scheduling flags cannot re-enter the work-item contract"


def wi_close_remote_rehydration() -> str:
    return "remote create and close preserve typed work-item state"


def typed_epic_owner_authoring() -> str:
    return "change ownership is authored through typed epic identity"


def typed_priority_label_authoring() -> str:
    return "priority labels round-trip without grammar drift"


def capability_to_epic_planning() -> str:
    return "project planning separates certain epics from uncertain changes"


def epic_to_change_atomization() -> str:
    return "accepted epic plans atomize into bounded changes"


def work_item_planning_operational_efficiency() -> str:
    return "work-item planning stays within its efficiency threshold"


def work_item_planning_operational_stability() -> str:
    return "work-item planning remains stable under repeated execution"
