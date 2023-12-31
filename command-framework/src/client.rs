use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{bail, Error, Result};
use futures_util::lock::Mutex;
use futures_util::StreamExt;
use lapin::message::Delivery;
use lapin::options::BasicAckOptions;
use poketwo_gateway_client::{GatewayClient, GatewayClientOptions};
use tracing::{error, info};
use twilight_http::client::InteractionClient;
use twilight_http::Client;
use twilight_model::application::interaction::Interaction;
use twilight_model::channel::message::MessageFlags;
use twilight_model::gateway::payload::incoming::InteractionCreate;
use twilight_model::http::interaction::{
    InteractionResponse, InteractionResponseData, InteractionResponseType,
};
use twilight_model::id::marker::GuildMarker;
use twilight_model::id::Id;

use crate::command::Command;
use crate::context::Context;

#[derive(Debug, Clone)]
pub struct CommandClientOptions<T> {
    pub amqp_url: String,
    pub amqp_exchange: String,
    pub amqp_queue: String,
    pub amqp_routing_keys_extra: Vec<String>,
    pub guild_ids: Vec<Id<GuildMarker>>,
    pub commands: Vec<Command<T>>,
}

#[derive(Debug)]
pub struct CommandClient<'a, T> {
    pub http: &'a Client,
    pub interaction: InteractionClient<'a>,
    pub gateway: GatewayClient,
    pub state: Arc<Mutex<T>>,

    guild_ids: Vec<Id<GuildMarker>>,
    commands: HashMap<String, Command<T>>,
}

impl<'a, T> CommandClient<'a, T> {
    pub async fn connect(
        http: &'a Client,
        state: T,
        options: CommandClientOptions<T>,
    ) -> Result<CommandClient<'a, T>> {
        let mut amqp_routing_keys = vec!["INTERACTION.APPLICATION_COMMAND.*".into()];
        amqp_routing_keys.extend_from_slice(&options.amqp_routing_keys_extra);

        let gateway_options = GatewayClientOptions {
            amqp_url: options.amqp_url.clone(),
            amqp_exchange: options.amqp_exchange.clone(),
            amqp_queue: options.amqp_queue.clone(),
            amqp_routing_keys,
        };

        let gateway = GatewayClient::connect(gateway_options).await?;
        let application = http.current_user_application().exec().await?.model().await?;
        let interaction = http.interaction(application.id);

        let mut commands = HashMap::new();

        for command in options.commands {
            let key = command.command.name.clone();
            match commands.get(&key) {
                Some(_) => bail!("Duplicate command {}", command.command.name),
                None => commands.insert(key, command),
            };
        }

        Ok(Self {
            http,
            interaction,
            gateway,
            commands,
            guild_ids: options.guild_ids,
            state: Arc::new(Mutex::new(state)),
        })
    }

    pub async fn register_commands(&self) -> Result<()> {
        info!("Registering commands");

        for command in self.commands.values() {
            for guild_id in &self.guild_ids {
                let mut action = self
                    .interaction
                    .create_guild_command(*guild_id)
                    .chat_input(&command.command.name, &command.command.description)?
                    .command_options(&command.command.options)?;

                if let Some(value) = command.command.default_permission {
                    action = action.default_permission(value);
                }

                action.exec().await?;
            }
        }

        Ok(())
    }

    pub async fn run(&mut self) -> Result<()> {
        while let Some(delivery) = self.gateway.consumer.next().await {
            if let Err(error) = self._handle_delivery(delivery?).await {
                error!("{:?}", error);
            }
        }

        Ok(())
    }

    pub async fn handle_delivery(&self, delivery: Delivery) -> Result<Result<()>, Delivery> {
        if delivery.routing_key.as_str().starts_with("INTERACTION.APPLICATION_COMMAND") {
            Ok(self._handle_delivery(delivery).await)
        } else {
            Err(delivery)
        }
    }

    async fn _handle_delivery(&self, delivery: Delivery) -> Result<()> {
        delivery.ack(BasicAckOptions::default()).await?;

        let event: InteractionCreate = serde_json::from_slice(&delivery.data)?;

        if let Interaction::ApplicationCommand(interaction) = event.0 {
            if let Some(command) = self.commands.get(&interaction.data.name) {
                let ctx = Context { client: self, interaction: &*interaction };

                if let Err(error) = (command.handler)(ctx.clone()).await {
                    self.handle_command_error(command, ctx, error).await?;
                };
            }
        }

        Ok(())
    }

    async fn handle_command_error(
        &self,
        command: &Command<T>,
        ctx: Context<'a, T>,
        error: Error,
    ) -> Result<()> {
        fn make_error_response(error: Error) -> InteractionResponse {
            InteractionResponse {
                kind: InteractionResponseType::ChannelMessageWithSource,
                data: Some(InteractionResponseData {
                    content: Some(error.to_string()),
                    flags: Some(MessageFlags::EPHEMERAL),
                    ..Default::default()
                }),
            }
        }

        let response = match command.error_handler {
            Some(error_handler) => match error_handler(ctx.clone(), error).await {
                Ok(()) => return Ok(()),
                Err(error) => make_error_response(error),
            },
            _ => make_error_response(error),
        };

        ctx.create_response(&response).exec().await?;

        Ok(())
    }
}
