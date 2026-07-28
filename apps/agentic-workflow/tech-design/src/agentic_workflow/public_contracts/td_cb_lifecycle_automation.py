"""Public TD boundary for TD-to-CB lifecycle automation."""

__aw_artifact_id__ = "artifact:td-cb-lifecycle-automation/public-contract"
__aw_public_contract__ = True


def remove_td_merge_command() -> str:
    return "the linear lifecycle has no TD merge phase"


def td_create_dirty_persistent_branch() -> str:
    return "TD create preserves dirty persistent work areas"


def td_existing_workspace_dirty_persistent_branch() -> str:
    return "TD and CB workspace verbs preserve dirty work areas"


def td_cb_lifecycle_automation_operational_efficiency() -> str:
    return "TD-to-CB automation stays within its efficiency threshold"


def td_cb_lifecycle_automation_operational_stability() -> str:
    return "TD-to-CB automation remains stable under repeated execution"
