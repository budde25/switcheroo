#!/bin/sh

rm Switcheroo*.AppImage
cargo clean

cargo build --release

mkdir -p target/AppDir/usr/bin
cp target/release/switcheroo target/AppDir/usr/bin

mkdir -p target/AppDir/usr/share/metainfo/
cp extra/linux/io.ebudd.Switcheroo.appdata.xml target/AppDir/usr/share/metainfo

# desktop file
mkdir -p target/AppDir/usr/share/applications/
cp extra/linux/io.ebudd.Switcheroo.desktop target/AppDir/usr/share/applications

# icon
mkdir -p target/AppDir/usr/share/icons/hicolor/512x512/apps
cp extra/logo/io.ebudd.Switcheroo.png target/AppDir/usr/share/icons/hicolor/512x512/apps

appimage-builder --recipe appimage.yml --appdir target/AppDir --skip-tests --log DEBUG