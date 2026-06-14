// SPEC-MANAGED: projects/cap/tech-design/logic/cap-hook-auto-command-optimizer-whitelist.md#cap-hook-auto-command-optimizer-whitelist
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec cap-hook-auto-command-optimizer-whitelist
// @capability agent-hook-installation
// @contract hook-payload-rewrite-adapters
// @category behavior
// @command env CC=/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/cc SDKROOT=/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk PATH=/Users/chrischeng/.rustup/toolchains/stable-aarch64-apple-darwin/bin:/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin cargo test -p cap hook -- --nocapture && env CC=/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/cc SDKROOT=/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk PATH=/Users/chrischeng/.rustup/toolchains/stable-aarch64-apple-darwin/bin:/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin cargo test -p cap command_planner -- --nocapture && env CC=/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/cc SDKROOT=/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk PATH=/Users/chrischeng/.rustup/toolchains/stable-aarch64-apple-darwin/bin:/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin cargo test -p cap active_replacements_match_success_and_error_behavior -- --nocapture && env CC=/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/cc SDKROOT=/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX.sdk PATH=/Users/chrischeng/.rustup/toolchains/stable-aarch64-apple-darwin/bin:/usr/bin:/bin:/usr/sbin:/sbin:/opt/homebrew/bin cargo bench -p cap --bench command_resources
// AW-EC-END

#[test]
#[ignore = "AW EC placeholder: implement this external contract test or keep the manifest command authoritative"]
fn cap_hook_auto_command_optimizer_whitelist() {
    panic!("AW EC placeholder for cap-hook-auto-command-optimizer-whitelist");
}
// CODEGEN-END
