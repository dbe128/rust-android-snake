#!/bin/sh
# Builds the Android app (APK) inside a Docker image that contains Rust, the
# Android SDK and NDK, and the cargo-quad-apk tool. The image is described in
# android/Dockerfile and is built automatically the first time (that takes a while).
#
# Needs Docker running. Result: target/android-artifacts/release/apk/snake-mobile.apk
# Install it on a phone connected by USB with:
#   adb install -r target/android-artifacts/release/apk/snake-mobile.apk
#
# The two extra folders keep things between builds:
#   .android/                  the debug signing key. Android only installs an update
#                              signed with the same key as the installed app.
#   target/docker-registry/    downloaded libraries, so they aren't downloaded every time

set -e                  # stop at the first command that fails
cd "$(dirname "$0")"    # run from this script's folder, wherever it's called from

# Build the image, or reuse it if android/Dockerfile hasn't changed.
docker build --quiet --platform linux/amd64 -t snake-android-builder android > /dev/null

mkdir -p .android target/docker-registry

docker run --rm --platform linux/amd64 \
    -v "$PWD":/root/src \
    -v "$PWD/.android":/root/.android \
    -v "$PWD/target/docker-registry":/usr/local/cargo/registry \
    snake-android-builder \
    cargo quad-apk build --release

echo "Built: target/android-artifacts/release/apk/snake-mobile.apk"
