#!/bin/bash

# This script builds the application for MacOS
# An OSX package is laid out like:
#  AppName.app/               <- The app directory
#    Contents/
#      Info.plist             <- Application metadata xml
#      MacOS/                 <- Binary and resources
#        AppName               <- The actual binary
#      Resources/             <- Info.plist and other resources
#      Frameworks/            <- SDL2 and other frameworks

set -e


MACOS_BIN_NAME=gui
MACOS_APP_NAME=Postie
MACOS_APP_DIR=$MACOS_APP_NAME.app

echo "Creating app directory structure"
rm ./postie.dmg
rm -rf $MACOS_APP_DIR
mkdir -p $MACOS_APP_DIR/Contents/MacOS

echo "Copying frameworks"
cp -r /Library/Frameworks/SDL2.framework $MACOS_APP_DIR/Contents

echo "Copying binary"
MACOS_APP_BIN=$MACOS_APP_DIR/Contents/MacOS/$MACOS_BIN_NAME
cp target/release/gui $MACOS_APP_BIN

echo "Linking binary with frameworks"
for old in `otool -L $MACOS_APP_BIN | grep @rpath | cut -f2 | cut -d' ' -f1`; do
    new=`echo $old | sed -e "s/@rpath/@executable_path\/..\/Frameworks/"`
    echo "Replacing '$old' with '$new'"
    install_name_tool -change $old $new $MACOS_APP_BIN
done

echo "Copying database"
cp ./postie.sqlite $MACOS_APP_DIR/Contents/MacOS

# $MACOS_APP_BIN --help

echo "Copying launcher"
cp scripts/launch.sh $MACOS_APP_DIR/Contents/MacOS/$MACOS_APP_NAME

echo "Creating dmg"
mkdir $MACOS_APP_NAME
mv $MACOS_APP_DIR $MACOS_APP_NAME
ln -s /Applications $MACOS_APP_NAME/Applications
rm -rf $MACOS_APP_NAME/.Trashes

FULL_NAME=$APP_NAME-$OS-$MACHINE-$SUFFIX

hdiutil create postie.dmg -srcfolder $MACOS_APP_NAME -ov
rm -rf $MACOS_APP_NAME
