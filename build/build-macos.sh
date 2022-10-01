TARGET=$1
PKG_NAME="enum-dir.$TARGET.tar.gz"

cargo build --release --target "${TARGET}"
mkdir build_tmp

tar czvf "build_tmp/$PKG_NAME" -C "./target/$TARGET/release/" enum-dir
shasum -a 256 "build_tmp/$PKG_NAME" > "build_tmp/$PKG_NAME.sha256"