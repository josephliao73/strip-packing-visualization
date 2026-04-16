#!/usr/bin/env bash
set -euo pipefail

# winit/mesa expect a writable runtime dir even on X11-only sessions.
if [ -z "${XDG_RUNTIME_DIR:-}" ] || [ ! -d "${XDG_RUNTIME_DIR:-}" ]; then
  export XDG_RUNTIME_DIR=/tmp/runtime-root
fi

mkdir -p "$XDG_RUNTIME_DIR"
chmod 700 "$XDG_RUNTIME_DIR"

# If the host passed an X11 display, steer the app away from Wayland probing.
if [ -n "${DISPLAY:-}" ] && [ -z "${WAYLAND_DISPLAY:-}" ] && [ -z "${WAYLAND_SOCKET:-}" ]; then
  export WINIT_UNIX_BACKEND=x11
  export XDG_SESSION_TYPE=x11
fi

# Avoid MIT-SHM/X11 shared-memory paths that frequently misbehave in containers.
export _X11_NO_MITSHM=1
export QT_X11_NO_MITSHM=1
export LIBGL_DRI3_DISABLE=1

exec "$@"
