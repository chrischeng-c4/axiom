// swift-tools-version: 5.9
// HANDWRITE-BEGIN gap="missing-generator:logic:cd75b93b" tracker="pending-tracker" reason="Declare the macOS Swift package, SwiftTerm dependency, native executable, model library, and XCTest target."

import PackageDescription

let package = Package(
    name: "WorkbenchMac",
    platforms: [.macOS(.v14)],
    products: [
        .library(name: "WorkbenchMacCore", targets: ["WorkbenchMacCore"]),
        .executable(name: "WorkbenchMac", targets: ["WorkbenchMac"]),
    ],
    dependencies: [
        .package(
            url: "https://github.com/migueldeicaza/SwiftTerm.git",
            exact: "1.15.0"
        ),
    ],
    targets: [
        .target(name: "WorkbenchMacCore"),
        .executableTarget(
            name: "WorkbenchMac",
            dependencies: [
                "WorkbenchMacCore",
                .product(name: "SwiftTerm", package: "SwiftTerm"),
            ]
        ),
        .testTarget(
            name: "WorkbenchMacCoreTests",
            dependencies: ["WorkbenchMacCore"]
        ),
    ],
    swiftLanguageVersions: [.v5]
)
// HANDWRITE-END
