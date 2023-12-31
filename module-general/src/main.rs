mod command;
mod config;
mod state;

use anyhow::Result;
use poketwo_command_framework::client::{CommandClient, CommandClientOptions};
use poketwo_protobuf::poketwo::database::v1::database_client::DatabaseClient;
use twilight_http::Client;
use twilight_model::id::Id;

use crate::command::{pick, start};
use crate::config::CONFIG;
use crate::state::State;

pub type Context<'a> = poketwo_command_framework::context::Context<'a, State>;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let options = CommandClientOptions {
        amqp_url: CONFIG.amqp_url.clone(),
        amqp_exchange: CONFIG.amqp_exchange.clone(),
        amqp_queue: CONFIG.amqp_queue.clone(),
        amqp_routing_keys_extra: vec![],
        commands: vec![start(), pick()],
        guild_ids: vec![Id::new(967272023845929010), Id::new(787517653211938877)],
    };

    let http = Client::new(CONFIG.token.clone());
    let database = DatabaseClient::connect(CONFIG.database_service_url.clone()).await?;
    let state = State { database };
    let mut client = CommandClient::connect(&http, state, options).await?;

    client.register_commands().await?;
    client.run().await?;

    Ok(())
}
