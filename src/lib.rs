pub mod error;
pub mod utils;
use {data::Data, std::sync::Arc};

pub mod commands;
pub mod data;
pub mod game;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type CmdRet = std::result::Result<(), Error>;
pub type Context<'a> = poise::Context<'a, Arc<Data>, Error>;
pub type FrameworkError<'a> = poise::FrameworkError<'a, Arc<Data>, Error>;

pub const DEFAULT_COLOR: u32 = 0x87CEEB;
