// SPEC-MANAGED: projects/vat/tech-design/logic/external-contracts.md#vat-agent-state-and-diff-surface
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec vat-agent-state-and-diff-surface
// @capability agent-native-gpu-native-dev-containers
// @claim agent-legible-state-and-diff-surface
// @contract agent-legible-state-and-diff-surface
// @category behavior
// @required_for_production true
// @command rg -n -e 'vat state' -e 'vat diff' -e '--json' -e structured projects/vat/README.md
// AW-EC-END

#[test]
#[ignore = "AW EC placeholder: implement this external contract test or keep the aw.toml inventory command authoritative"]
fn vat_agent_state_and_diff_surface() {
    panic!(
        "AW EC placeholder for {}",
        "vat-agent-state-and-diff-surface"
    );
}
// CODEGEN-END
