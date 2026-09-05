//! Cursor position on KDE Plasma (Wayland) via KWin scripting + D-Bus.
//!
//! Same technique as `kdotool` / `wdotool`: generate a temporary JS snippet
//! that reads `workspace.cursorPos` and reports back via `callDBus` to a
//! transient service registered on the session bus.
//!
//! - Uses the bus unique name (`:1.xxx`) because scripts loaded via
//!   `loadScript` on Plasma 6 cannot reach well-known names.
//! - JS numbers arrive as int32, hence the `i32` signature.

use crate::{Error, Point};
use std::collections::HashMap;
use std::io::Write;
use std::sync::{
    atomic::{AtomicI32, Ordering},
    Arc,
};
use std::time::Duration;
use tokio::sync::{oneshot, Mutex};

static NEXT_ID: AtomicI32 = AtomicI32::new(1);

const BRIDGE_PATH: &str = "/com/mousecoords/KdeBridge";
const BRIDGE_IFACE: &str = "com.mousecoords.KdeBridge";

#[zbus::proxy(
    interface = "org.kde.kwin.Scripting",
    default_service = "org.kde.KWin",
    default_path = "/Scripting"
)]
trait KwinScripting {
    #[zbus(name = "loadScript")]
    fn load_script(&self, file_path: &str, plugin_name: &str) -> zbus::Result<i32>;
    #[zbus(name = "unloadScript")]
    fn unload_script(&self, plugin_name: &str) -> zbus::Result<bool>;
}

#[zbus::proxy(interface = "org.kde.kwin.Script", default_service = "org.kde.KWin")]
trait KwinScript {
    #[zbus(name = "run")]
    fn run(&self) -> zbus::Result<()>;
    #[zbus(name = "stop")]
    fn stop(&self) -> zbus::Result<()>;
}

#[derive(Default)]
struct Pending {
    pointer_waiters: HashMap<i32, oneshot::Sender<Option<(i32, i32)>>>,
}

struct Bridge {
    pending: Arc<Mutex<Pending>>,
}

#[zbus::interface(name = "com.mousecoords.KdeBridge")]
impl Bridge {
    async fn report_pointer(&self, req_id: i32, ok: bool, x: i32, y: i32) {
        if let Some(tx) = self.pending.lock().await.pointer_waiters.remove(&req_id) {
            let _ = tx.send(if ok { Some((x, y)) } else { None });
        }
    }
}

fn dbus_err(e: zbus::Error) -> Error {
    Error::Query(format!("dbus: {e}"))
}

fn io_err(e: std::io::Error) -> Error {
    Error::Query(format!("io: {e}"))
}

fn pointer_script(req_id: i32, bridge_addr: &str) -> String {
    format!(
        r#"
(function() {{
  var p = workspace.cursorPos;
  var ok = (p && typeof p.x === "number" && typeof p.y === "number");
  callDBus(
    "{service}", "{path}", "{iface}", "ReportPointer",
    {id}, ok, ok ? (p.x | 0) : 0, ok ? (p.y | 0) : 0
  );
}})();
"#,
        service = bridge_addr,
        path = BRIDGE_PATH,
        iface = BRIDGE_IFACE,
        id = req_id
    )
}

pub async fn get_position_async() -> Result<Point, Error> {
    let conn = zbus::Connection::session().await.map_err(dbus_err)?;

    let pending = Arc::new(Mutex::new(Pending::default()));
    conn.object_server()
        .at(
            BRIDGE_PATH,
            Bridge {
                pending: pending.clone(),
            },
        )
        .await
        .map_err(dbus_err)?;

    let bridge_addr = conn
        .unique_name()
        .ok_or_else(|| Error::Query("dbus has no unique name".into()))?
        .to_string();

    let req_id: i32 = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let (tx, rx) = oneshot::channel::<Option<(i32, i32)>>();
    pending.lock().await.pointer_waiters.insert(req_id, tx);

    // Avoid plugin_name collisions across calls and processes.
    let plugin_name = format!(
        "mousecoords-{}-{req_id}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    );

    let mut tmp = tempfile::Builder::new()
        .prefix("mousecoords-")
        .suffix(".js")
        .tempfile()
        .map_err(io_err)?;
    tmp.write_all(pointer_script(req_id, &bridge_addr).as_bytes())
        .map_err(io_err)?;
    tmp.flush().map_err(io_err)?;
    let path = tmp.into_temp_path();
    let path_str = path
        .to_str()
        .ok_or_else(|| Error::Query("invalid tmp path".into()))?;

    let scripting = KwinScriptingProxy::new(&conn).await.map_err(dbus_err)?;
    let script_id = scripting
        .load_script(path_str, &plugin_name)
        .await
        .map_err(dbus_err)?;
    if script_id < 0 {
        return Err(Error::Query(format!("loadScript returned {script_id}")));
    }

    let script_path = format!("/Scripting/Script{script_id}");
    let script = KwinScriptProxy::builder(&conn)
        .path(script_path)
        .map_err(dbus_err)?
        .build()
        .await
        .map_err(dbus_err)?;
    script.run().await.map_err(dbus_err)?;

    let wait = tokio::time::timeout(Duration::from_secs(3), rx).await;
    let _ = scripting.unload_script(&plugin_name).await;

    let result = wait
        .map_err(|_| Error::Query("timeout waiting for KWin callback".into()))?
        .map_err(|_| Error::Query("KWin script exited before callback".into()))?;

    match result {
        Some((x, y)) => Ok(Point { x, y }),
        None => Err(Error::Query("KWin returned an invalid cursor".into())),
    }
}

/// Sync wrapper: runs the runtime on a dedicated thread so it works
/// both outside and inside an existing tokio runtime.
pub fn get_position_blocking() -> Result<Point, Error> {
    std::thread::spawn(move || {
        match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(rt) => rt.block_on(get_position_async()),
            Err(e) => Err(Error::Query(format!("tokio rt: {e}"))),
        }
    })
    .join()
    .map_err(|_| Error::Query("kwin thread panicked".into()))?
}
