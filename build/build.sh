TARGET=$1
echo "current target: $TARGET"
PKG_NAME="enum-dir.$TARGET.tar.gz"

cross build --target "$TARGET" --release
mkdir -p build_tmp

if [[ "$TARGET" == *"-linux-"* ]]; then
    tar czvf "build_tmp/$PKG_NAME" -C "./target/$TARGET/release/" enum-dir
elif [[ "$TARGET" == *"-windows-"* ]]; then
    tar czvf "build_tmp/$PKG_NAME" -C "./target/$TARGET/release/" enum-dir.exe
fi

shasum -a 256 "build_tmp/$PKG_NAME" > "build_tmp/$PKG_NAME.sha256"