# How to build?
The Example of Android(arm64):
## 1. Config cargo
Edit ~/.cargo/config
```
[target.aarch64-linux-android]
linker="/home/sfdex/Android/Sdk/ndk/25.2.9519653/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android33-clang++"
```

## 2. Config rustup
```
rustup target add aarch64-linux-android
```

## 3. Run build
```
cargo build --release --target aarch64-linux-android
```