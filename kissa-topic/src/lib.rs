#![feature(unboxed_closures, trait_upcasting, trait_alias)]
#![warn(missing_docs)]
#![doc = "该包是 kissabot 的基础包。"]

/// kokoro 是一个元框架，它是 kissabot 的基础
pub use kokoro_neo as kokoro;

pub use log;

pub use mlua as lua;

/// 上下文概念来自于 kokoro
pub mod context {
    use crate::kissa::Kissa;
    use std::sync::Arc;
    /// 携带类型的上下文
    pub type Context<T> = kokoro_neo::context::Context<T, Arc<dyn kokoro_neo::any::KAny>, Kissa>;
    /// 抹去类型的上下文
    pub type RawContext = kokoro_neo::context::RawContext<Arc<dyn kokoro_neo::any::KAny>, Kissa>;
    pub use kokoro_neo::context::{ChildHandle, Children, RawContextExt};
}

/// 事件通道
pub mod channel {
    pub use flume::*;
    use kokoro_neo::any::KAny;
    use std::sync::Arc;
    /// kissabot 的发送者
    pub type Sender = flume::Sender<Arc<dyn KAny>>;
    /// kissabot 的接收者
    pub type Receiver = flume::Receiver<Arc<dyn KAny>>;
}

/// 可用性概念来自于 kokoro
pub mod avail {
    use kokoro_neo::any::KAny;
    use std::sync::Arc;

    /// 事件基本类型
    pub type Event = Arc<dyn KAny>;
    /// kissabot 的可用例
    pub type Availed<T, Param, Func> =
        kokoro_neo::avail::Availed<T, Param, Func, Arc<dyn KAny>, Kissa>;
    /// kissabot 的可用例句柄
    pub type AvailHandle<T, Param, Func> =
        kokoro_neo::avail::AvailHandle<T, Param, Func, Arc<dyn KAny>, Kissa>;
    /// 参数特征
    pub trait Params<T: KAny> = kokoro_neo::avail::Params<T, Arc<dyn KAny>, Kissa>;
    pub use kokoro_neo::avail::Avail;
    pub use kokoro_neo::avail::Avails;

    use crate::prelude::Kissa;
}

/// kissabot 的适配器
pub mod adapter;
/// 对 kokoro 上下文的扩展
pub mod context_ext;
/// kissabot 的主结构体
pub mod kissa;
/// kissabot plugin
pub mod plugin;

pub mod message;

pub mod event;

/// 主要模块
pub mod prelude {
    pub use crate::adapter::*;
    pub use crate::avail::*;
    pub use crate::channel;
    pub use crate::context::*;
    pub use crate::context_ext::*;
    pub use crate::export_plugin;
    pub use crate::kissa::*;
    pub use crate::plugin::*;
    pub use kokoro_neo::any::*;
    pub use kokoro_neo::plugin::dynamic::{
        changelog, library_filename, os, Error, Library, Symbol,
    };
    pub use kokoro_neo::result::Result;
    pub use log::{debug, error, info, trace, warn};
}

/// 订阅常规事件
#[macro_export]
macro_rules! subscribe {
    ($ctx:expr,$event:ty,$subscriber:expr) => {
        $crate::context_ext::ContextExt::observe(
            &$ctx,
            |ctx: $crate::context::Context<_>, event: $crate::avail::Event| {
                if let Some(se) =
                    <dyn $crate::kokoro::any::KAny>::downcast_ref::<$event>(event.as_ref())
                {
                    $subscriber(ctx, se);
                }
            },
        );
    };
}

/// 导出插件
#[macro_export]
macro_rules! export_plugin {
    ($plugin_type:ty) => {
        #[no_mangle]
        pub extern "Rust" fn __setup_logger__(
            logger: &'static dyn $crate::log::Log,
            level: $crate::log::LevelFilter,
        ) {
            $crate::log::set_max_level(level);
            let _ = $crate::log::set_logger(logger);
        }
        #[no_mangle]
        extern "Rust" fn __load__(
            ctx: ::std::sync::Arc<$crate::context::RawContext>,
            global: $crate::kissa::Kissa,
        ) -> $crate::kokoro::result::Result<()> {
            let ctx = unsafe {
                $crate::context::RawContextExt::downcast_unchecked::<$plugin_type>(
                    ctx, None, global,
                )
            };
            <$plugin_type as $crate::plugin::Plugin>::load(ctx)?;
            Ok(())
        }
        #[no_mangle]
        extern "Rust" fn __create__(
            lua: &$crate::lua::Lua,
            value: $crate::lua::Value,
        ) -> $crate::kokoro::result::Result<::std::sync::Arc<dyn $crate::kokoro::any::KAny>, String>
        {
            let config: <$plugin_type as $crate::plugin::Plugin>::Config =
                $crate::lua::LuaSerdeExt::from_value(lua, value)
                    .map_err(|err| format!("配置序列化错误: {}", err))?;
            let plugin = <$plugin_type as $crate::plugin::Plugin>::create(config)
                .map_err(|err| format!("插件实例创建失败: {}", err))?;
            Ok(::std::sync::Arc::new(plugin))
        }
    };
}
