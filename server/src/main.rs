#[macro_use]
extern crate log;

use anyhow::Result;
use tokio::runtime::Builder as RuntimeBuilder;

mod client;
mod server;

fn main() -> Result<()> {
    let runtime = RuntimeBuilder::new_multi_thread().enable_all().build()?;
    runtime.block_on(server::Server::new().start_server())
}
