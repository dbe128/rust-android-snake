# The build environment for the Android version of the game.
#
# It contains Rust, the Android SDK and NDK, and the APK build tool (cargo-quad-apk),
# so none of them need to be installed on the computer itself.
# build-apk.sh builds this image automatically the first time; afterwards Docker
# reuses it, until something in this file changes.
#
# To upgrade a part, change its version below and run build-apk.sh again.
# build-apk.sh builds it for Linux/x86_64 (--platform linux/amd64), because Google
# only publishes the Android NDK for x86_64 Linux. On an Apple Silicon Mac, Docker
# runs it in emulation.
FROM archlinux:latest

# The versions used. All were the newest stable ones in October 2026.
ARG RUST_VERSION=1.99.1
ARG CMDLINE_TOOLS=16111833
ARG ANDROID_PLATFORM=36
ARG BUILD_TOOLS=37.0.0
ARG NDK_VERSION=30.0.16248370
# cargo-quad-apk isn't released on crates.io any more, so we pin a commit on GitHub.
ARG QUAD_APK_COMMIT=d411c8fe1c08339e46d7dc6b40ac04ea5e3a01b6

# Linux packages: Java (for the Android tools), unzip/wget (for downloads),
# and a C compiler + cmake + openssl, which compiling cargo-quad-apk needs.
# --disable-sandbox: pacman's download sandbox doesn't work under Docker's emulation.
RUN pacman -Syu --noconfirm --needed --disable-sandbox \
        jdk17-openjdk unzip wget zip rustup gcc cmake pkgconf openssl \
    && pacman -Scc --noconfirm

# Rust, with the standard library for 64-bit ARM Android.
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH
RUN rustup toolchain install ${RUST_VERSION} --profile minimal --target aarch64-linux-android \
    && rustup default ${RUST_VERSION}

# Android SDK command-line tools, then the parts of the SDK we need, and the NDK.
ENV ANDROID_HOME=/opt/android-sdk \
    JAVA_HOME=/usr/lib/jvm/default
RUN mkdir -p ${ANDROID_HOME}/cmdline-tools && cd ${ANDROID_HOME}/cmdline-tools \
    && wget -q https://dl.google.com/android/repository/commandlinetools-linux-${CMDLINE_TOOLS}_latest.zip \
    && unzip -q commandlinetools-linux-${CMDLINE_TOOLS}_latest.zip \
    && rm commandlinetools-linux-${CMDLINE_TOOLS}_latest.zip \
    && mv cmdline-tools latest
RUN yes | ${ANDROID_HOME}/cmdline-tools/latest/bin/sdkmanager --licenses > /dev/null \
    && ${ANDROID_HOME}/cmdline-tools/latest/bin/sdkmanager \
        "platforms;android-${ANDROID_PLATFORM}" "build-tools;${BUILD_TOOLS}" "ndk;${NDK_VERSION}" > /dev/null
ENV NDK_HOME=${ANDROID_HOME}/ndk/${NDK_VERSION} \
    PATH=${ANDROID_HOME}/build-tools/${BUILD_TOOLS}:$PATH

# The APK build tool. --locked uses the exact library versions from its Cargo.lock.
# Deleting the download cache afterwards keeps the image smaller.
RUN cargo install --git https://github.com/not-fl3/cargo-quad-apk --rev ${QUAD_APK_COMMIT} --locked \
    && rm -rf ${CARGO_HOME}/registry ${CARGO_HOME}/git

WORKDIR /root/src
