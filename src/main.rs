use zbus::{MessageHeader, ObjectServer, SignalContext, dbus_interface, Connection};
use std::collections::HashMap;
use std::time::Duration;
use std::sync::Mutex;

#[derive(Debug, Default)]
struct IdleMapper {
    shared: Mutex<IdleMapperShared>,
}

#[derive(Debug, Clone, Default)]
struct IdleMapperShared {
    cookie_idx: u32,
    inhibits_by_cookie: HashMap<u32, FdoInhibit>
}

#[derive(Debug, Clone)]
struct FdoInhibit {
    // FIXME: we need to track the peer which created this inhibit, otherwise they'll leak

    application_name: String,
    reason_for_inhibit: String,
    cookie: u32,
}

#[dbus_interface(name = "org.freedesktop.ScreenSaver")]
impl IdleMapper {
    #[dbus_interface(out_args("cookie"))]
    async fn inhibit(
        &self,
        application_name: &str,
        reason_for_inhibit: &str,
    ) -> zbus::fdo::Result<u32> {
        let mut ims = self.shared.lock().unwrap();
        let cookie = ims.cookie_idx;
        ims.cookie_idx += 1;
        let inhibit = FdoInhibit {
            application_name: application_name.to_owned(),
            reason_for_inhibit: reason_for_inhibit.to_owned(),
            cookie,
        };
        if let Some(old_inhibit) = ims.inhibits_by_cookie.insert(cookie, inhibit) {
            panic!("cookie conflict: {} -> {:?}", cookie, old_inhibit);
        }
        Ok(cookie)
    }

    async fn uninhibit(
        &self,
        cookie: u32,
    ) -> zbus::fdo::Result<()> {
        let mut ims = self.shared.lock().unwrap();

        if let None = ims.inhibits_by_cookie.remove(&cookie) {
            Err(zbus::fdo::Error::FileNotFound(format!("cookie {} not in inhibits", cookie)))
        } else {
            Ok(())
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Sync + Send + 'static>> {
    let conn = Connection::session().await?;
    let interface = IdleMapper::default();
    conn
        .object_server_mut()
        .await
        .at("/org/freedesktop/ScreenSaver", interface)?;

    conn.request_name("com.codyps.IdleMapper").await?;

    println!("got name");

    // TODO: find a nicer way to do this without awaiting on timers
    loop {
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}
