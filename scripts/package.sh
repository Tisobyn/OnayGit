#!/bin/sh
# Build OnayGit.app and a zip next to it in dist/.   Usage: scripts/package.sh
set -e
cd "$(dirname "$0")/.."

VERSION=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
APP=dist/OnayGit.app
export PATH="$HOME/.cargo/bin:$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH"

cargo build --release

rm -rf dist && mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp target/release/OnayGit "$APP/Contents/MacOS/OnayGit"
strip -x "$APP/Contents/MacOS/OnayGit"

# Icon: 512 px PNG -> .icns
ICONSET=dist/OnayGit.iconset
mkdir -p "$ICONSET"
for s in 16 32 64 128 256 512; do
  sips -z $s $s assets/icon-512.png --out "$ICONSET/icon_${s}x${s}.png" >/dev/null
done
cp "$ICONSET/icon_32x32.png"   "$ICONSET/icon_16x16@2x.png"
cp "$ICONSET/icon_64x64.png"   "$ICONSET/icon_32x32@2x.png"
cp "$ICONSET/icon_256x256.png" "$ICONSET/icon_128x128@2x.png"
cp "$ICONSET/icon_512x512.png" "$ICONSET/icon_256x256@2x.png"
cp assets/icon-512.png         "$ICONSET/icon_512x512@2x.png"
rm "$ICONSET/icon_64x64.png"
iconutil -c icns "$ICONSET" -o "$APP/Contents/Resources/OnayGit.icns"
rm -rf "$ICONSET"

cat > "$APP/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key><string>OnayGit</string>
  <key>CFBundleDisplayName</key><string>OnayGit</string>
  <key>CFBundleIdentifier</key><string>com.tisobyn.onaygit</string>
  <key>CFBundleExecutable</key><string>OnayGit</string>
  <key>CFBundleIconFile</key><string>OnayGit</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>$VERSION</string>
  <key>CFBundleVersion</key><string>$VERSION</string>
  <key>LSMinimumSystemVersion</key><string>11.0</string>
  <key>LSApplicationCategoryType</key><string>public.app-category.developer-tools</string>
  <key>NSHighResolutionCapable</key><true/>
  <key>NSHumanReadableCopyright</key><string>Copyright (c) 2026 Tisobyn. MIT License.</string>
</dict>
</plist>
PLIST

# Ad-hoc signature so macOS runs it on this Mac. Use a Developer ID for sharing.
codesign --force --deep --sign - "$APP"

ditto -c -k --keepParent "$APP" "dist/OnayGit-$VERSION-macos.zip"
echo "Built $APP"
ls -lh "dist/OnayGit-$VERSION-macos.zip"
