#!/usr/bin/env bash
# Huuk desktop launcher (no sudo on this box):
#  - the WebKitGTK stack lives in ~/tauri-sys/root, so it goes on the loader
#    path;
#  - this WebKit build hardcodes its helper dir (/usr/lib/webkit2gtk-4.1) with
#    no env override, so a tiny LD_PRELOAD shim rewrites that prefix to the
#    staged copy, and WEBKIT_FORCE_SANDBOX=0 makes the helpers spawn directly
#    (no bubblewrap step that would need the same hardcoded paths).
#    Trade-off: web content runs unsandboxed. A single future
#    `sudo pacman -S webkit2gtk-4.1 xdg-dbus-proxy` removes both hacks.
SYS="$HOME/.local/share/tauri-sys"
export LD_LIBRARY_PATH="$SYS/root/usr/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
export LD_PRELOAD="$SYS/lib/webkit-path-shim.so${LD_PRELOAD:+:$LD_PRELOAD}"
export HUUK_WEBKIT_DIR="$SYS/root/usr/lib/webkit2gtk-4.1"
export HUUK_DBUS_PROXY="$SYS/root/usr/bin/xdg-dbus-proxy"
# This WebKit ignores WEBKIT_FORCE_SANDBOX; this is its documented replacement.
export WEBKIT_DISABLE_SANDBOX_THIS_IS_DANGEROUS=1
# Tauri resolves the relative frontendDist (../dist) against the working
# directory, so run from the shell dir — otherwise it falls back to the
# dev-server URL and the window shows "connection refused".
cd /home/aslpooht/PROJECT/_TEENUENG-COMPANY/content-planner-huuk/app/src-tauri
exec /home/aslpooht/PROJECT/_TEENUENG-COMPANY/content-planner-huuk/app/src-tauri/target/release/huuk-desktop "$@"
