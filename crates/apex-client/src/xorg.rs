//! Linux: the X server gpui draws through (XWayland, under a Wayland
//! session; see `LINUX.md`), for what gpui has no call for: the pointer
//! moved (`warp`), the pointer asked where it is over a web view, which
//! keeps its moves to itself (`pointer`), and the window handed to wry
//! as the Xlib window it wants (`Parent`).
//!
//! Its own connection to the server, opened once: a window's id is the
//! server's, good on any connection.

use std::sync::OnceLock;

use gpui::{px, Pixels, Point, Window};
use raw_window_handle::{HandleError, HasWindowHandle, RawWindowHandle, WindowHandle, XlibWindowHandle};
use x11rb::connection::Connection;
use x11rb::protocol::xproto::ConnectionExt as _;
use x11rb::rust_connection::RustConnection;

/// Before gpui starts: draw through X11 even in a Wayland session,
/// unless `APEX_WAYLAND=1` (gpui's own Wayland: no web views, no warp),
/// and start GTK for the web views on that same server. The variables
/// gpui decides by are given back once it has (`restore`), so what is
/// run from here gets the session as it was.
pub fn prepare() -> Saved {
    let wayland = std::env::var_os("WAYLAND_DISPLAY");
    let native = std::env::var("APEX_WAYLAND").is_ok_and(|v| v == "1");
    let x = std::env::var_os("DISPLAY").is_some_and(|d| !d.is_empty());
    let saved = Saved { wayland: if native || !x { None } else { wayland } };
    if saved.wayland.is_some() {
        std::env::remove_var("WAYLAND_DISPLAY");
    }
    if x && !native {
        gtk::gdk::set_allowed_backends("x11");
        if let Err(e) = gtk::init() {
            eprintln!("apex-ui: GTK (for web views): {e}");
        }
    }
    saved
}

/// What `prepare` took from the environment.
pub struct Saved {
    wayland: Option<std::ffi::OsString>,
}

impl Saved {
    /// gpui has chosen: the environment as it was.
    pub fn restore(self) {
        if let Some(w) = self.wayland {
            std::env::set_var("WAYLAND_DISPLAY", w);
        }
    }
}

/// GTK's events, turned for the web views: WebKitGTK draws and answers
/// only from GTK's loop, which nothing else here runs.
pub fn pump(cx: &mut gpui::App) {
    if !gtk::is_initialized_main_thread() {
        return;
    }
    cx.spawn(async move |cx| loop {
        cx.background_executor().timer(std::time::Duration::from_millis(8)).await;
        while gtk::events_pending() {
            gtk::main_iteration_do(false);
        }
    })
    .detach();
}

fn conn() -> Option<&'static RustConnection> {
    static CONN: OnceLock<Option<RustConnection>> = OnceLock::new();
    CONN.get_or_init(|| x11rb::connect(None).ok().map(|(c, _)| c)).as_ref()
}

/// The X window gpui draws `window` in, if it draws through X11.
fn xid(window: &Window) -> Option<u32> {
    let h = HasWindowHandle::window_handle(window).ok()?;
    match h.as_raw() {
        RawWindowHandle::Xcb(h) => Some(h.window.get()),
        RawWindowHandle::Xlib(h) => Some(h.window as u32),
        _ => None,
    }
}

/// Put the pointer at `p`, a point in this window's coordinates.
pub fn warp(window: &Window, p: Point<Pixels>) {
    let (Some(c), Some(w)) = (conn(), xid(window)) else { return };
    let s = window.scale_factor();
    let (x, y) = ((f32::from(p.x) * s).round() as i16, (f32::from(p.y) * s).round() as i16);
    let _ = c.warp_pointer(x11rb::NONE, w, 0, 0, 0, 0, x, y);
    let _ = c.flush();
}

/// Where the pointer is, in the window's own coordinates, asked of the
/// server (gpui hears nothing while it is over a web view).
pub fn pointer(window: &Window) -> Option<Point<Pixels>> {
    let (c, w) = (conn()?, xid(window)?);
    let r = c.query_pointer(w).ok()?.reply().ok()?;
    if !r.same_screen {
        return None;
    }
    let s = window.scale_factor();
    Some(Point { x: px(f32::from(r.win_x) / s), y: px(f32::from(r.win_y) / s) })
}

/// The window as wry takes a parent on X11: an Xlib window (gpui's
/// handle is XCB's; the id is the same).
pub struct Parent(XlibWindowHandle);

impl Parent {
    pub fn of(window: &Window) -> Option<Parent> {
        Some(Parent(XlibWindowHandle::new(xid(window)? as std::ffi::c_ulong)))
    }
}

impl HasWindowHandle for Parent {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        // SAFETY: the id is of gpui's window, alive while the web view
        // made from it is (a web view goes with its window)
        Ok(unsafe { WindowHandle::borrow_raw(RawWindowHandle::Xlib(self.0)) })
    }
}
