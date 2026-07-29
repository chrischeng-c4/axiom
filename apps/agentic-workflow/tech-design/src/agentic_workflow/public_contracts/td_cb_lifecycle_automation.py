"""Public TD boundary for TD-to-CB lifecycle automation."""

__aw_artifact_id__ = "artifact:td-cb-lifecycle-automation/public-contract"
__aw_public_contract__ = True


def remove_td_merge_command() -> str:
    return "the linear lifecycle has no TD merge phase"


def td_create_dirty_persistent_branch() -> str:
    return "TD create preserves dirty persistent work areas"


def td_existing_workspace_dirty_persistent_branch() -> str:
    return "TD and CB workspace verbs preserve dirty work areas"


def python_td_bounded_native_handwrite() -> str:
    return (
        "Python EC and TD may drive a bounded native HANDWRITE implementation; "
        "TD identity resolves from the configured project TD root, using exact WI "
        "references when present and existing flat source modules for bounded adoption; "
        "a WI-scoped TD check records an exact idempotent Td-Python-Source baseline; "
        "generator-owned targets retain cold parity while HANDWRITE targets "
        "classify every native source in every configured workspace root and "
        "fail closed on escaped paths or symlinks; Cb-Gen evidence binds the "
        "target and workspace root and commits only emitted files; HANDWRITE "
        "targets must pass committed implementation-evidence plus workspace-test gates; "
        "terminal touched-scope ownership is bounded to those native sources rather than "
        "the persistent branch's historical EC, TD, or build-artifact diff"
    )


def td_cb_lifecycle_automation_operational_efficiency() -> str:
    return "TD-to-CB automation stays within its efficiency threshold"


def td_cb_lifecycle_automation_operational_stability() -> str:
    return "TD-to-CB automation remains stable under repeated execution"
