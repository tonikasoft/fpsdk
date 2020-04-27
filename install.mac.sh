#! /bin/sh

# installs release build into FL effects folder
# Usage:
# install.mac.sh name destination_name [gen]
# if [gen] is specified, the plugin will be installed in as a generator.

set -e

name=$1
dest_name=$2
PLUG_PATH="/Applications/FL Studio 20.app/Contents/Resources/FL/Plugins/Fruity"
MIDDLE_DIR="Effects"

if [ $# -eq 3 ]; then
    MIDDLE_DIR="Generators"
fi

INSTALL_DIR="${PLUG_PATH}/${MIDDLE_DIR}/${dest_name}"

rm -rf "${INSTALL_DIR}"
mkdir "${INSTALL_DIR}"
mv "target/release/examples/lib${name}.dylib" "${INSTALL_DIR}/${dest_name}_x64.dylib"
