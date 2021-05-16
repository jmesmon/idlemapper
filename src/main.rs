use futures::prelude::*;
use std::convert::TryInto;
use std::future::ready;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Sync + Send + 'static>> {
    let conn = zbus::azync::Connection::new_session().await?;

    let t = {
        let conn = conn.clone();
        tokio::task::spawn(async move {
            loop {
                let op = "/org/freedesktop/ScreenSaver".try_into().unwrap();

                let pred = |message: &zbus::Message| -> Result<bool, zbus::Error> {
                    // check if message is our object
                    let header = message.header()?;

                    if header.message_type()? != zbus::MessageType::MethodCall {
                        println!("type not right: {:?}", message);
                        return Ok(false);
                    }

                    // is this an object we have?
                    if header.path()? != Some(&op) {
                        println!("path unhandled: {:?}", message);
                        return Ok(false);
                    }

                    Ok(true)
                };
                let message = conn.receive_specific(|message| {
                    ready(pred(message)).boxed()
                }).await.unwrap();

                let header = message.header().unwrap();

                // do we impliment this interface for that object? If not, return an error
                if header.interface().unwrap() != Some("org.freedesktop.ScreenSaver") {
                    println!("interface unhandled: {:?}", message);
                    todo!();
                }

                // does this interface impliment the requested method? If not, return an error
                if header.member().unwrap() != Some("Inhbit") {
                    todo!();
                }

                // does the type signature match? alternately: does deserialization work?
                // "su"
                let body: (String, u32) = message.body().unwrap();

                println!("BODY: {:?}", body);
            }
        })
    };

    conn.call_method(
        Some("org.freedesktop.DBus"),
        "/org/freedesktop/DBus",
        Some("org.freedesktop.DBus"),
        "RequestName",
        &("com.codyps.IdleMapper", 0u32)
    ).await?;

    println!("got name");


    t.await?;

    Ok(())
}
