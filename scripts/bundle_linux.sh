#!/bin/bash
rm -rf ./target/release ./squash-root
NO_STRIP=1 cargo packager --release --formats appimage,deb
./target/release/bundle/gui_0.0.12_x86_64.AppImage --appimage-extract
