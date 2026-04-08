// swift-tools-version: 6.1

import PackageDescription

let package = Package(
    name: "CorsaUtils",
    products: [
        .library(name: "CorsaUtils", targets: ["CorsaUtils"]),
        .executable(name: "CorsaUtilsBench", targets: ["CorsaUtilsBench"]),
    ],
    targets: [
        .target(name: "CorsaUtils"),
        .executableTarget(
            name: "CorsaUtilsBench",
            dependencies: ["CorsaUtils"],
            linkerSettings: [
                .unsafeFlags(["-L", "../../../../target/debug", "-lcorsa_ffi"]),
            ]
        ),
        .testTarget(
            name: "CorsaUtilsTests",
            dependencies: ["CorsaUtils"]
        ),
    ]
)
