# better-poketwo
A Better Discord bot.

## Components
Better-Pokétwo is a collection of smaller components:

### Supporting Services
These services handle important tasks and are relied on by other modules.

- `database` — handles database operations, accepts GRPC requests
- `gateway` — handles connections to Discord Gateway, sends events to RabbitMQ
- `imgen` — handles image generation, accepts GRPC requests

### Command Modules
These services consume Discord events from RabbitMQ, including commands and messages.

- `module-pokedex` — Pokédex and Pokémon inventory-related commands
- `module-catching` — spawning and catching of Pokémon
- `module-general` — miscellaneous commands not belonging to any other category
- `module-market` — the global Pokémon marketplace
- `module-auctions` — (not yet implemented) Pokémon auction functionality
- `module-battling` — battles between trainers
- `module-shop` — the in-game item shop
- `module-quests` — daily and weekly quests

### Libraries
These components are shared code used by other services.

- `command-framework` — primary slash command handler framework
- `command-framework-macros` — macros used for the command framework
- `emojis` — Pokétwo's custom emoji library
- `gateway-client` — client that consumes events from RabbitMQ
- `i18n-rust` — enables localization of messages
- `protobuf` — Protobuf declarations shared across all components
- `protobuf-elixir` — generated Protobuf code for Elixir
- `protobuf-rust` — generated Protobuf code for Rust
- `resources` — folder for translations and other static files

## Deploying
Deploying Better-Pokétwo is not simple, and is not meant to be. If persistent then DM for setup and other features to buy.

If you would still like to run your own instance, perhaps for development purposes, and you know what you are doing, a sample docker-compose.yml is provided. As of now, however, there is no other documentation or instructions for building, and no support whatsoever will be provided.

