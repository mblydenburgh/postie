#!/bin/bash

# This script builds the application for MacOS
# An OSX package is laid out like:
#  AppName.app/               <- The app directory
#    Contents/
#      Info.plist             <- Application metadata xml
#      MacOS/                 <- Binary and resources
#        AppName              <- The actual binary
#      Resources/             <- Info.plist and other resources
#      Frameworks/            <- Any Non-system libraries go here, found with `otool -L binary_name`

set -e


RUST_BINARY_NAME=gui
APP_BIN_NAME=Postie
VERSION=0.0.3
MACOS_APP_DIR=$APP_BIN_NAME.app
MACOS_APP_BIN_PATH=$MACOS_APP_DIR/Contents/MacOS/$RUST_BINARY_NAME

echo "Creating application directory"
rm -rf $MACOS_APP_DIR
mkdir -p $MACOS_APP_DIR/Contents/MacOS

echo "Copying binary"
cp target/release/$RUST_BINARY_NAME $MACOS_APP_BIN_PATH
chmod +x $MACOS_APP_BIN_PATH
chmod -R 775 $MACOS_APP_DIR
chown -R $(whoami) $MACOS_APP_DIR

echo "Copying database file"
cp -r postie.sqlite $MACOS_APP_DIR/Contents/MacOS/postie.sqlite
chmod -R 777 $MACOS_APP_DIR/Contents/MacOS/postie.sqlite

# Note: CFBundleExecutable is the name of the launch script, not the binary
echo "Creating Info.plist"
cat > $MACOS_APP_DIR/Contents/Info.plist <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple Computer/DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key>
  <string>$APP_BIN_NAME</string>
  <key>CFBundleDisplayName</key>
  <string>$APP_BIN_NAME</string>
  <key>CFBundleIdentifier</key>
  <string>com.mblydenburgh.postie</string>
  <key>CFBundleGetInfoString</key>
  <string>$APP_BIN_NAME</string>
  <key>CFBundleVersion</key>
  <string>0.1.0</string>
  <key>CFBundleShortVersionString</key>
  <string>0.1.0</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>CFBundleSignature</key>
  <string>wdld</string>
  <key>CFBundleExecutable</key>
  <string>$APP_BIN_NAME</string>
  <key>CFBundleInfoDictionaryVersion</key>
  <string>6.0</string>
</dict>
</plist>
EOF


echo "Copying launcher and making it executable"
cp scripts/launcher.sh $MACOS_APP_DIR/Contents/MacOS/$APP_BIN_NAME
chmod +x $MACOS_APP_DIR/Contents/MacOS/$APP_BIN_NAME

echo "Creating .dmg"
mkdir $APP_BIN_NAME
mv $MACOS_APP_DIR $APP_BIN_NAME
echo "Making dmg binary executable"
chmod +x $APP_BIN_NAME/$MACOS_APP_DIR/Contents/MacOS/$RUST_BINARY_NAME
ln -s /Applications $APP_BIN_NAME/Applications
FULL_NAME=$APP_BIN_NAME-$VERSION
hdiutil create target/$FULL_NAME.dmg -srcfolder $APP_BIN_NAME -ov
rm -rf $APP_BIN_NAME
