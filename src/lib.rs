pub mod checks;
pub mod error;
pub mod models;
use std::sync::Arc;

use data::Data;

pub mod commands;
pub mod data;
pub mod game;

pub type Error = Box<dyn std::error::Error + Send + Sync>;

/// Abbreviated for "CommandReturn"
pub type CmdRet = std::result::Result<(), Error>;
pub type Context<'a> = poise::ApplicationContext<'a, Arc<Data>, Error>;
pub type ContextEnum<'a> = poise::Context<'a, Arc<Data>, Error>;
pub type FrameworkError<'a> = poise::FrameworkError<'a, Arc<Data>, Error>;

pub const DEFAULT_COLOR: u32 = 0x87CEEB;

pub trait IntoAppContext<'a> {
    fn into_app_context(self) -> Context<'a>;
}

impl<'a> IntoAppContext<'a> for ContextEnum<'a> {
    fn into_app_context(self) -> Context<'a> {
        if let ContextEnum::Application(ctx) = self {
            ctx
        } else {
            unreachable!()
        }
    }
}
