"""Public TD boundary for work-item planning and authoring."""

__aw_artifact_id__ = "artifact:work-item-planning/public-contract"
__aw_public_contract__ = True


def canonical_four_type_wi_taxonomy() -> str:
    return "typed work items round-trip through authoring and graph projection"


def canonical_change_cache_round_trip() -> str:
    return "canonical change identity survives issue-cache rehydration"


def typed_work_item_authoring_profiles() -> str:
    return "each work-item type has one bounded authoring profile"


def spike_terminal_convergence() -> str:
    return "spikes terminate with decision evidence or expiry"


def report_terminal_triage() -> str:
    return "reports terminate through typed triage"


def shared_cli_report_intake() -> str:
    return "shared CLI issue intake creates typed reports"


def typed_intake_health() -> str:
    return "health observes report and spike intake state"


def terminology_first_work_item_vocabulary() -> str:
    return "CLI and docs share one work-item vocabulary"


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
