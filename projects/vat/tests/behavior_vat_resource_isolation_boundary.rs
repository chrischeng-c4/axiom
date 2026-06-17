// SPEC-MANAGED: projects/vat/tech-design/logic/external-contracts.md#vat-resource-isolation-boundary
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec vat-resource-isolation-boundary
// @capability agent-native-gpu-native-dev-containers
// @claim resource-isolation-boundary
// @contract resource-isolation-boundary
// @category behavior
// @required_for_production true
// @command rg -n -e sandbox -e isolation -e seatbelt projects/vat/README.md projects/vat/src/sandbox
// AW-EC-END

#[test]
#[ignore = "AW EC placeholder: implement this external contract test or keep the manifest command authoritative"]
fn vat_resource_isolation_boundary() {
    panic!(
        "AW EC placeholder for {}",
        "vat-resource-isolation-boundary"
    );
}
// CODEGEN-END
