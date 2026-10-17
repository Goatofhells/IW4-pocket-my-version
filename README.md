# IW4 Pocket
An incremental iOS launcher foundation, **not an IW4 engine port**.

## Current milestone
SwiftUI dark launcher with Home, sandboxed document import, verification, unavailable practice/multiplayer screens, persistent future graphics preferences, and diagnostics. Rust provides streaming SHA-256, versioned manifest validation, bounded ZIP extraction, memory budget recommendations, and a C ABI.
No MW2 hashes have been invented: the trusted catalog is empty, so all installations are unsupported. Import does not prove ownership. No copyrighted game files are included.

## GitHub-only builds
`.github/workflows/ios-release.yml` uses macos-15, Rust stable, aarch64-apple-ios, XcodeGen and xcodebuild. Push the foundation branch or run **iOS release** manually; an optional new release tag publishes a prerelease only after success. No local builds are required. The supplied baseline was not attached; this workflow follows the requested requirements.
The workflow runs Rust and Python tests, builds the ARM64 static library and native application, packages Payload/IW4Pocket.app, verifies Mach-O architecture/metadata/resources, and uploads an unsigned IPA plus logs. A generated Cargo.lock is retained as an artifact; check it into the repository after the first successful run for reproducibility.
**Unsigned IPA artifacts require appropriate Apple code signing and provisioning before installation on physical iPhones.** They are not directly installable.
Xcode project is generated from ios/project.yml; Rust strings cross a documented C ABI and are freed by Rust. No Swift package or third-party UI dependency is used.

## Verification status
See GitHub Actions for actual results. No local build or test execution is authorized. Source review is not proof of compilation, device execution or gameplay. Synthetic test hashes belong only to original test fixtures.

## Scope and next milestone
Finish and verify steps 1–5 in GitHub before proceeding. Step 6 is the actual IPA build gate. Only after a green build, review signed device launch and import behavior; then implement a minimal Metal original arena, first-person camera and touch movement as a separately tested change. Bots and networking follow playable device testing. Do not treat the placeholder directories or navigation destinations as implemented gameplay.

## Limitations and licensing
No renderer, arena, controls, bots, networking, server compatibility, controller support, resource cache or device performance benchmark exists yet. Settings are persistent preferences only. Memory diagnostics currently report physical RAM and a recommended cache ceiling, not process resident memory.
MW2 feasibility research remains a later milestone: source availability and licenses, DirectX and Windows dependencies, ARM64 portability, asset formats and server authentication/protocols must be investigated with cited sources before any compatibility claims. No lawful compatible runtime has been established; original gameplay stays disabled. Do not copy proprietary engine code or assets, bypass DRM, anti-cheat or authentication. MW2 trademarks belong to their respective owners; this project is independent and unaffiliated.
Original project code is MIT licensed; Rust dependencies retain their own licenses. Audit dependencies before distribution.
