// swift-tools-version: 6.0
// NOTE: The primary build system is the Xcode project (generated via XcodeGen from project.yml).
// This Package.swift is retained for tooling compatibility (e.g., SourceKit-LSP).
import PackageDescription

let package = Package(
    name: "OtherworldsApp",
    platforms: [
        .iOS(.v18),
        .macOS(.v15),
    ],
    targets: [
        .executableTarget(
            name: "OtherworldsApp",
            path: "Sources/OtherworldsApp",
            resources: [
                .process("Resources"),
            ]
        ),
        .testTarget(
            name: "OtherworldsAppTests",
            dependencies: ["OtherworldsApp"],
            path: "Tests/OtherworldsAppTests"
        ),
    ]
)
