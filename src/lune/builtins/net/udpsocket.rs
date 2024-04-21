use bstr::{BString, ByteSlice};
use std::{net::SocketAddr, rc::Weak, str::FromStr, sync::Arc};
use tokio::{io::ReadBuf, net::UdpSocket as AsyncUdpSocket};

use mlua::prelude::*;

const MAX_PACKET_SIZE: usize = 1024;

#[derive(Debug)]
pub struct UdpSocket(Arc<AsyncUdpSocket>);

pub trait UdpListenerExt {
    async fn bind(addr: SocketAddr) -> LuaResult<UdpSocket>;
    async fn next(&self, lua: &Lua) -> LuaResult<LuaRegistryKey>;
}

impl UdpSocket {
    pub async fn connect(&self, remote_addr: SocketAddr) -> LuaResult<()> {
        self.0.connect(remote_addr).await.into_lua_err()
    }

    pub async fn send(&self, buf: BString) -> LuaResult<()> {
        self.0.send(buf.as_bytes()).await?;

        Ok(())
    }
}

impl UdpListenerExt for UdpSocket {
    async fn bind(addr: SocketAddr) -> LuaResult<UdpSocket> {
        Ok(Self(Arc::new(AsyncUdpSocket::bind(addr).await?)))
    }

    async fn next(&self, lua: &Lua) -> LuaResult<LuaRegistryKey> {
        let lua_inner = lua
            .app_data_ref::<Weak<Lua>>()
            .expect("bad")
            .upgrade()
            .expect("fucking upgrade pls");

        let buf = &mut [0; MAX_PACKET_SIZE];
        let mut buf_read = ReadBuf::new(buf);

        self.0.recv_buf(&mut buf_read).await?;

        let reg = lua_inner.create_registry_value(lua_inner.create_buffer(buf_read.filled())?)?;

        Ok(reg)
    }
}

impl LuaUserData for UdpSocket {
    fn add_methods<'lua, M: LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_async_method("connect", |_, this, remote_addr: String| async move {
            this.connect(SocketAddr::from_str(remote_addr.as_str()).map_err(|err| {
                mlua::Error::FromLuaConversionError {
                    from: "String",
                    to: "SocketAddr",
                    message: Some(err.to_string().to_string()),
                }
            })?)
            .await
        });

        methods.add_async_method("send", |_, this, buf| async move { this.send(buf).await });
        methods.add_async_method("next", |lua, this, _: ()| async move {
            let reg = UdpSocket::next(this, lua).await?;
            let val: LuaAnyUserData<'lua> = lua.registry_value(&reg)?;
            Ok(val)
        });
    }
}
