# Plonky2 SHA256 Benchmark

This code is a fork of https://github.com/polymerdao/plonky2-sha256/tree/main

> [!NOTE]  
> The custom dependency for `plonky2_u32` is necessary because the original outdated `plonky2_u32` depended on an old version of `plonky2` and did not support serialization, so it would be impossible to measure PK/VK/proof size. Awaiting PR merge (https://github.com/0xPolygonZero/plonky2-u32/pull/7)

## Building and Running the Mobile Apps

## Generating ZK Artifacts

To generate the necessary ZK artifacts, run the following command:

```sh
cargo run --bin compile_sha256_circuit
```

### iOS

Generate bindings using mopro:

```sh
CONFIGURATION=release cargo run --bin ios # Release mode
```

Open the project in Xcode:

```sh
open ios/MoproApp.xcodeproj
```

### Android

Generate bindings using mopro:

```sh
CONFIGURATION=release cargo run --bin android # Release mode
```

Open the project in Android Studio:

```sh
open android -a Android\ Studio
```
