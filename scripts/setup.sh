#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN_DIR="${ROOT_DIR}/bin"
VERSION=$(curl -fsSL https://api.github.com/repos/eigenwallet/core/releases/latest | grep '"tag_name"' | cut -d'"' -f4)
if [[ -z "${VERSION}" ]]; then
  echo "Failed to resolve latest release version from GitHub"
  exit 1
fi
BASE_URL="https://github.com/eigenwallet/core/releases/download/${VERSION}"

mkdir -p "${BIN_DIR}"

OS="$(uname -s)"
ARCH="$(uname -m)"

case "${OS}" in
  Linux) OS_NAME="Linux" ;;
  Darwin) OS_NAME="Darwin" ;;
  *)
    echo "Unsupported OS: ${OS}"
    exit 1
    ;;
esac

case "${ARCH}" in
  x86_64|amd64) ARCH_NAME="x86_64" ;;
  arm64|aarch64) ARCH_NAME="aarch64" ;;
  *)
    echo "Unsupported architecture: ${ARCH}"
    exit 1
    ;;
esac

ASB_TAR="asb_${VERSION}_${OS_NAME}_${ARCH_NAME}.tar"
SWAP_TAR="swap_${VERSION}_${OS_NAME}_${ARCH_NAME}.tar"

echo "Detected: ${OS_NAME} ${ARCH_NAME}"
echo "Downloading ASB binaries to ${BIN_DIR}"

ASB_TMP="${BIN_DIR}/${ASB_TAR}"
SWAP_TMP="${BIN_DIR}/${SWAP_TAR}"

curl -fL "${BASE_URL}/${ASB_TAR}" -o "${ASB_TMP}"
curl -fL "${BASE_URL}/${SWAP_TAR}" -o "${SWAP_TMP}"

tar -xf "${ASB_TMP}" -C "${BIN_DIR}"
tar -xf "${SWAP_TMP}" -C "${BIN_DIR}"

rm -f "${ASB_TMP}" "${SWAP_TMP}"

chmod +x "${BIN_DIR}/asb" "${BIN_DIR}/swap"

echo
echo "Download complete."
echo "Next steps:"
echo "1) Start ASB on testnet:"
echo "   ./bin/asb --testnet start"
echo "2) In another terminal, run the monitor:"
echo "   cargo run"
