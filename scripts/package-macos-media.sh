#!/bin/bash
# Assemble the macOS media bundle from the Legacy Viewer's CMake targets and
# autobuild packages (indra/newview/viewer_manifest.py, DarwinManifest).
set -euo pipefail

if [[ $# != 3 ]]; then
    echo "Usage: $0 <source checkout> <CMake build directory> <new output directory>" >&2
    exit 2
fi

source_dir=$1
build_dir=$2
out=$3
if [[ -e "$out" ]]; then
    echo "Output already exists: $out" >&2
    exit 1
fi

app="$out/SLPlugin.app"
frameworks="$app/Contents/Frameworks"
libs="$build_dir/packages/lib/release"
mkdir -p "$out"
ditto "$build_dir/llplugin/slplugin/Release/SLPlugin.app" "$app"
mkdir -p "$frameworks"
for plugin in cef libvlc; do
    cp -p "$build_dir/media_plugins/$plugin/Release/media_plugin_$plugin.dylib" "$frameworks/"
done
for item in 'Chromium Embedded Framework.framework' 'DullahanHelper.app' \
    'DullahanHelper (Alerts).app' 'DullahanHelper (GPU).app' \
    'DullahanHelper (Renderer).app' 'DullahanHelper (Plugin).app' plugins; do
    ditto "$libs/$item" "$frameworks/$item"
done
for lib in "$libs"/libvlc*.dylib*; do
    [[ -f "$lib" ]] || { echo "Missing VLC libraries in $libs" >&2; exit 1; }
    cp -p "$lib" "$frameworks/"
done
ditto "$build_dir/packages/LICENSES" "$out/LICENSES"
cp -p "$source_dir/LICENSE" "$out/LICENSES/legacy-viewer-LGPL-2.1.txt"
