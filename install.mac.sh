#! /bin/sh

# installs release build into FL effects folder
# Usage:
# install.mac.sh name destination_name

name=$1
dest_name=$2
PLUG_PATH="/Applications/FL Studio 20.app/Contents/Resources/FL/Plugins/Fruity"

rm -rf "${PLUG_PATH}/Effects/${dest_name}"
mkdir "${PLUG_PATH}/Effects/${dest_name}"
mv "target/release/examples/lib${name}.dylib" "${PLUG_PATH}/Effects/${dest_name}/${dest_name}_x64.dylib"
