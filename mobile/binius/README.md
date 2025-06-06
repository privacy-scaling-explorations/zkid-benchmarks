# Binius SHA256 benchmarks

These benchmarks use official circuits from the Binius project: https://github.com/IrreducibleOSS/binius/blob/main/examples/sha256_circuit.rs and https://github.com/IrreducibleOSS/binius/blob/main/examples/sha256_circuit_with_lookup.rs.

## The use of mopro

The mobile apps are built using mopro.

**📚 To learn more about mopro, visit: https://zkmopro.org**

## Getting Started

To set up and build bindings, follow these steps.

### 1. Install the Mopro CLI Tool

```sh
git clone https://github.com/zkmopro/mopro
cd mopro/cli
cargo install --path .
```

### 2. Initialize adapter

Navigate to the Mopro example app directory and initialize setup by running:

```sh
mopro init
```

### 3. Generate Native Bindings

Build bindings for your project by executing:

```sh
mopro build
```

### 4. Create Platform-Specific Templates

To generate templates tailored to your target platform, use:

```sh
mopro create
```

### 5. Open the project

Follow the instructions to open the development tools

For iOS:

```sh
open ios/MoproApp.xcodeproj
```

For Android:

```sh
open android -a Android\ Studio
```

For Web:

```sh
cd web && yarn && yarn start
```

For React Native:
Follow the README in the `react-native` directory. Or [zkmopro/react-native-app/README.md](https://github.com/zkmopro/react-native-app/blob/main/README.md)

For Flutter:
Follow the README in the `flutter` directory. Or [zkmopro/flutter-app/README.md](https://github.com/zkmopro/flutter-app/blob/main/README.md)

### 6. Update bindings

When build the bindings again, manually update the bindings in the folder

For iOS:

```sh
cp -r MoproiOSBindings ios
```

For Android:

```sh
cp -r MoproAndroidBindings/uniffi android/app/src/main/java && \
cp -r MoproAndroidBindings/jniLibs android/app/src/main
```

## Advanced: Customize Builds Using Rust

For advanced usage, you can manually run Rust commands to build in either debug or release mode.

### iOS

- Debug Mode:
  ```sh
  cargo run --bin ios  # Debug mode
  ```
- Release Mode:
  ```sh
  CONFIGURATION=release cargo run --bin ios # Release mode
  ```

### Android

- Debug Mode:
  ```sh
  cargo run --bin android  # Debug mode
  ```
- Release Mode:
  ```sh
  CONFIGURATION=release cargo run --bin android # Release mode
  ```

### Web

- Debug Mode:
  ```sh
  cargo run --bin web  # Debug mode
  ```
- Release Mode:
  ```sh
  CONFIGURATION=release cargo run --bin web # Release mode
  ```

## Community

- X account: <a href="https://twitter.com/zkmopro"><img src="https://img.shields.io/twitter/follow/zkmopro?style=flat-square&logo=x&label=zkmopro"></a>
- Telegram group: <a href="https://t.me/zkmopro"><img src="https://img.shields.io/badge/telegram-@zkmopro-blue.svg?style=flat-square&logo=telegram"></a>
