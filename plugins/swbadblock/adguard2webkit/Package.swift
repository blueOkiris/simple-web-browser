// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "AdGuard2WebKit",
    products: [
        .library(
            name: "AdGuard2WebKit",
            type: .static,
            targets: [ "AdGuard2WebKit" ]
        )
    ], dependencies: [
        .package(path: "./SafariConverterLib-4.1.0")
    ], targets: [
        .target(
            name: "AdGuard2WebKit",
            dependencies: [
                .product(name: "ContentBlockerConverter", package: "SafariConverterLib-4.1.0")
            ]
        )
    ]
)
