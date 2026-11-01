# apex on Linux (the `apex-linux` branch)

The daemon (`apexd`), the CLI (`apex`) and the app (`apex-ui`) all build
and run on Linux.

## Building

Needs Rust, a C toolchain, zig 0.16 (for libghostty-vt, built by
`ghostty-vt-sys`) and WebKitGTK 4.1 with GTK 3 (`webkit2gtk-4.1` and `gtk3`
on Arch):

    ZIG=/path/to/zig-0.16/zig cargo build --release -p apex-client -p apex-server -p apex-cli

which gives `target/release/{apex,apexd,apex-ui}`.

## How the app runs here

- **X11, through XWayland in a Wayland session.** `apex-ui` hides
  `WAYLAND_DISPLAY` from gpui at startup (and puts it back afterwards, so
  whatever apex runs gets the session as it was). Only X11 allows the three
  things below. `APEX_WAYLAND=1` runs gpui on Wayland instead: no web views
  and no pointer warp.
- **Web views** are wry's WebKitGTK views, made as children of gpui's X
  window (`build_as_child`, which wry supports only on X11). GTK is
  started before gpui and its loop is pumped from a gpui task
  (`src/xorg.rs`).
- **The pointer**: acme's warps (`warp::move_to`) use `XWarpPointer` on
  gpui's window, and the position over a web view comes from
  `XQueryPointer` (gpui gets no events there). The pointer styles are
  gpui's own system cursors; over a page WebKit sets its own.
- **Nerd Font symbols**: the bundled `SymbolsNerdFontMono` goes to gpui
  with the other bundled faces. gpui's Linux text system (cosmic-text)
  drops a face with no `m`, so the build uses
  `assets/SymbolsNerdFontMono-Linux.ttf`, the same font plus a blank `m`,
  made by `linux/symbols-with-m.py` (fontTools). No system font is needed.
- **State** (last sessions, recently closed files) is kept in
  `$XDG_STATE_HOME/apex`, or `~/.local/state/apex`, instead of
  `~/Library/Application Support/apex`.

## Mac-only parts, left out here

The Dock icon and its badge (`attention.rs`; the bell is a BEL on
stderr), the plan9port bitmap cursors and the page's no-op cursor
(`cursor.rs`; `cursor_other.rs` stands in), CoreText's slashed-zero
lookup (`zero_for`), AppKit's state restoration
(`persistence_ignored`), the rounded foot and veil layers of web views,
the native title bar and traffic lights.
