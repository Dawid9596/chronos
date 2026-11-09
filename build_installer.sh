#!/usr/bin/env bash
set -euo pipefail

# Paths are overridable so CI can redirect them, defaults keep the local
# layout (everything on C: so the packaging happens on a short native path).
BIN_NAME="${BIN_NAME:-chronos}"
: "${PKG_NAME:=desktop}"
: "${CARGO_TARGET_DIR:=C:/build/chronos-target}"
export CARGO_TARGET_DIR
EXE_UNIX_PATH="$(cygpath -u "$CARGO_TARGET_DIR")/release/${BIN_NAME}.exe"

: "${PKG_WORK_DIR:=/c/build/chronos-pkg}"
STAGE_DIR="${PKG_WORK_DIR}/dll_stage"

echo "==> 1. Compiling the release binary..."
cargo build --package desktop --release

if [ ! -f "$EXE_UNIX_PATH" ]; then
    echo "Error: Executable not found at $EXE_UNIX_PATH"
    exit 1
fi

echo "==> 2. Preparing local packaging workspace on C: drive..."
rm -rf "$PKG_WORK_DIR"
mkdir -p "$STAGE_DIR/schemas"
mkdir -p "$PKG_WORK_DIR/icons"

cp Cargo.toml Cargo.lock "$PKG_WORK_DIR/"
cp -r "$PKG_NAME" "$PKG_WORK_DIR/"
cp -r src "$PKG_WORK_DIR/"
cp -r icons/* "$PKG_WORK_DIR/icons/"

echo "==> 3. Copying DLLs to $STAGE_DIR..."
cp "$EXE_UNIX_PATH" "${PKG_WORK_DIR}/temp_bin.exe"
SVG_LOADER="/ucrt64/lib/gdk-pixbuf-2.0/2.10.0/loaders/libpixbufloader-svg.dll"

for bin_to_scan in "${PKG_WORK_DIR}/temp_bin.exe" "$SVG_LOADER"; do
    for win_path in $(ntldd -R "$bin_to_scan" | grep -i "ucrt64" | awk -F'=>' '{print $2}' | awk '{print $1}'); do
        unix_path=$(cygpath -u "$win_path")
        if [ -f "$unix_path" ]; then
            cp -u "$unix_path" "$STAGE_DIR/"
        fi
    done
done
rm "${PKG_WORK_DIR}/temp_bin.exe"

for runtime_dll in libwinpthread-1.dll libgcc_s_seh-1.dll libstdc++-6.dll; do
    if [ -f "/ucrt64/bin/$runtime_dll" ]; then
        cp -u "/ucrt64/bin/$runtime_dll" "$STAGE_DIR/"
    fi
done

echo "==> 4. Staging GTK/GDK runtime assets locally..."
cp /ucrt64/share/glib-2.0/schemas/gschemas.compiled "$STAGE_DIR/schemas/"
cp -r /ucrt64/lib/gdk-pixbuf-2.0 "$STAGE_DIR/"
gdk-pixbuf-query-loaders > "$STAGE_DIR/gdk-pixbuf-2.0/2.10.0/loaders.cache"

mkdir -p "$STAGE_DIR/Adwaita"
cp /ucrt64/share/icons/Adwaita/index.theme "$STAGE_DIR/Adwaita/"
cp -r /ucrt64/share/icons/Adwaita/symbolic "$STAGE_DIR/Adwaita/"

mkdir -p "$STAGE_DIR/hicolor/256x256/apps"
cp /ucrt64/share/icons/hicolor/index.theme "$STAGE_DIR/hicolor/" 2>/dev/null || true
cp icons/bitmap.png "$STAGE_DIR/hicolor/256x256/apps/chronos.png"

echo "==> 5. Running cargo-packager from native C: drive..."
pushd "$PKG_WORK_DIR" > /dev/null
cargo packager --release -v
popd > /dev/null

echo "==> Done! Installer is ready in ${CARGO_TARGET_DIR}/release/"