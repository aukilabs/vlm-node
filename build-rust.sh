#!/usr/bin/env sh

set -e

if ! command -v rustup >/dev/null 2>&1; then
  curl https://sh.rustup.rs -sSf | sh -s -- -y
  . "${HOME}/.cargo/env"
fi

host_os="$(uname -s)"
case "$host_os" in
  Darwin) host_os=darwin; find_cmd="find" ;;
  Linux)  host_os=linux;  find_cmd="find"  ;;
esac

host_arch="$(uname -m)"
case "$host_arch" in
  x86_64)  host_arch=amd64 ;;
  aarch64) host_arch=arm64 ;;
  armv7l)  host_arch=arm; host_variant=v7 ;;
  armv6l)  host_arch=arm; host_variant=v6 ;;
esac

host_platform="${host_os}/${host_arch}"
echo "host_platform: ${host_platform}"
test -n "${host_variant}" && host_platform="${host_platform}/${host_variant}"

test -n "${TARGETPLATFORM}" || TARGETPLATFORM="${host_platform}"

case "${TARGETPLATFORM}" in
  linux/386)     rust_target=i686-unknown-linux-gnu ;;
  linux/amd64)   rust_target=x86_64-unknown-linux-gnu ;;
  linux/arm64)   rust_target=aarch64-unknown-linux-gnu ;;
  linux/arm/v7)  rust_target=armv7-unknown-linux-gnueabihf ;;
  linux/arm/v6)  rust_target=arm-unknown-linux-gnueabihf ;;
  darwin/amd64)  rust_target=x86_64-apple-darwin ;;
  darwin/arm64)  rust_target=aarch64-apple-darwin ;;
  windows/amd64) rust_target=x86_64-pc-windows-gnu ;;
  windows/arm64) rust_target=aarch64-pc-windows-msvc ;;
  *)             rust_target=unknown ;;
esac

platform="$(echo "$TARGETPLATFORM" | sed 's|/|-|g')"
mkdir -p "target/${platform}/release"

platform_file="target/last-platform.txt"
if [ -f "$platform_file" ]; then
  last_platform="$(cat "$platform_file")"
  if [ "$last_platform" != "$platform" ]; then
    echo "Platform changed from ${last_platform} to ${platform}. Cleaning target..."
    rm -Rf target/release/
  fi
fi
echo "$platform" > "$platform_file"

features=""
if [ "${CLIENT}" != "default" ] && [ "${CLIENT}" != "" ]; then
  features="--features ${CLIENT} --no-default-features "
fi

if [ "${host_platform}" != "${TARGETPLATFORM}" ]; then
  cargo install cross --git https://github.com/cross-rs/cross
  cross build ${features}--release --target "${rust_target}"
  $find_cmd "target/${rust_target}/release/" -maxdepth 1 -type f -executable -exec cp {} "target/${platform}/release/" \;
  if [ -n "${CI}" ]; then
    cat target/${rust_target}/release/*.d
    rm -Rf "target/${rust_target}"
    docker image rm "ghcr.io/cross-rs/${rust_target}:main" || true
  fi
else
  cargo build ${features}--release
  $find_cmd target/release/ -maxdepth 1 -type f -executable -exec cp {} "target/${platform}/release/" \;
  test -n "${CI}" && cat target/release/*.d
fi
