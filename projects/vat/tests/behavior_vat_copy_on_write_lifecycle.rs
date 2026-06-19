// SPEC-MANAGED: projects/vat/tech-design/logic/external-contracts.md#vat-copy-on-write-lifecycle
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec vat-copy-on-write-lifecycle
// @capability agent-native-gpu-native-dev-containers
// @claim copy-on-write-fork-and-snapshot-lifecycle
// @contract copy-on-write-fork-and-snapshot-lifecycle
// @category behavior
// @required_for_production true
// @command rg -n -e copy-on-write -e fork -e snapshot -e clonefile -e APFS projects/vat/README.md
// AW-EC-END

#[test]
#[ignore = "AW EC placeholder: implement this external contract test or keep the manifest command authoritative"]
fn vat_copy_on_write_lifecycle() {
    panic!("AW EC placeholder for vat-copy-on-write-lifecycle");
}
// CODEGEN-END
