#!/bin/bash
set -e


if [ -z "$1" ] | [ -z "$2" ]; then
  echo "\$1 parameter must be the elements-consensus-sys depend directory"
  echo "\$2 parameter must be the rust-secp256k1-sys version code (M_m_p format)"
  echo "\$3 parameter (optional) can be the revision to check out"
  exit 1
fi

PARENT_DIR=$1
VERSIONCODE=$2
REV=$3
DIR=elements
ORIGDIR=$(pwd)

while true; do
    read -r -p "$PARENT_DIR/$DIR will be deleted [yn]: " yn
    case $yn in
        [Yy]* ) break;;
        [Nn]* ) exit;;
        * ) echo "Please answer yes or no.";;
    esac
done

cd "$PARENT_DIR" || exit 1
rm -rf "$DIR"
git clone https://github.com/ElementsProject/elements.git "$DIR" --depth=1
cd "$DIR"
if [ -n "$REV" ]; then
    git checkout "$REV"
fi
HEAD=$(git rev-parse HEAD)

# enable global elements variable
git apply "../../patches/enable-elements.diff"

# trim the fat
git apply "../../patches/trim-elements.diff"
git apply "../../patches/remove-secp256k1.diff"

rm -rf "src/qt"
rm -rf "src/test"
rm -rf "test"
rm -rf "src/secp256k1/src" # we link against rust-secp256k1
rm -rf "src/leveldb" # not needed for consensus
rm -rf "src/wallet" # not needed for consensus

cd ..
echo "# This file was automatically created by $0" > ./elements-HEAD-revision.txt
echo "$HEAD" >> ./elements-HEAD-revision.txt

# We need to make some source changes to the files.

# To support compiling for WASM, we need to remove all methods that use malloc.
# To compensate, the secp_context_create and _destroy methods are redefined in Rust.

# Make sure none of the includes have a space
find "$DIR" -not -path '*/\.*' -type f -print0 | xargs -0 sed -i "s/# include/#include/g"

# Prefix all methods with rustsecp and a version prefix
find "$DIR" -not -path '*/\.*' -type f -print0 | xargs -0 sed -i "/^#include/! s/secp256k1_/rustsecp256k1_v${VERSIONCODE}_/g"

# special rule for a method that is not prefixed in libsecp
find "$DIR" -not -path '*/\.*' -type f -print0 | xargs -0 sed -i "/^#include/! s/ecdsa_signature_parse_der_lax/rustsecp256k1zkp_v${VERSIONCODE}_ecdsa_signature_parse_der_lax/g"

# Undo makefile changes
(cd "$DIR"; git restore "src/secp256k1/Makefile.am")

rm -rf "$DIR/.git"
