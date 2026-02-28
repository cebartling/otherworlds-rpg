// swift-tools-version: 6.0
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
            path: "Sources/OtherworldsApp"
        ),
        .testTarget(
            name: "OtherworldsAppTests",
            dependencies: ["OtherworldsApp"],
            path: "Tests/OtherworldsAppTests"
        ),
    ]
)
