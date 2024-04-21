use bstr::{BString, ByteSlice};
use std::sync::Arc;
use tokio::net::UdpSocket as AsyncUdpSocket;

use mlua::prelude::*;

pub struct UdpSocket(Arc<AsyncUdpSocket>);

impl UdpSocket {
    pub fn new(sock: AsyncUdpSocket) -> Self {
        Self(Arc::new(sock))
    }

    pub async fn send(&self, buf: BString) -> LuaResult<()> {
        self.0.send(buf.as_bytes()).await?;

        Ok(())
    }

    pub async fn next(&self) -> LuaResult<BString> {
        let mut buf = [0; 1024];

        self.0.recv_from(&mut buf).await?;

        Ok(BString::from(buf))
    }
}

impl Clone for UdpSocket {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl LuaUserData for UdpSocket {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_async_method("send", |_, this, buf| async move { this.send(buf).await });

        methods.add_async_method("next", |_, this, _: ()| async move { this.next().await });
    }
}
