# Game Server

API Game Server for the Semester Project

See [Protocol.md](Protocol.md) for the communication protocol details.

<br />

## Main Frameworks Used

- **[Rust](https://www.rust-lang.org/)** - Programming language
- **[Actix](https://github.com/actix/actix)** - Active object framework
- **[Actix Web](https://actix.rs/)** - Rust web framework
- **[Serde](https://serde.rs/)** - Serialization framework
- **[Rlua](https://github.com/amethyst/rlua)** - Run Lua code from Rust
- **[Log](https://docs.rs/log/0.4.14/log/)** - Log server information to the terminal

<br/>

## Compiling

You will need to install Rust by following the directions on the [main website](https://www.rust-lang.org/tools/install).
If you want to add the Rust utilities to your path, you will need to manually run `~/.cargo/env`,
or you can edit your `.bashrc` file to run this script automatically.

Finally, run `cargo build` from the root directory to compile the source code.
All of the additional frameworks listed will be installed automatically when you first build the project.
Be sure to compile the code using at least `Rust 1.65`. The code can be compiled using the `stable` channel.
If you are compiling for a production build, you should compile the code using `cargo build --release` instead.

Once the code is built, you can run the server using `cargo run` (development server) or `cargo run --release` (production server).
You can also optionally specify command-line arguments (Like `--port` or `--host`), which override any environment values specified in the `.env` files.
Use the `--help` flag to list all command-line options

<br/>

## Environment Variables

For running the game server, you will need to specify certain environment variables.
This can be done using the following files:

- `.env` - Environment variables shared by both development and production systems
- `.env.development` - Environment variables only on development system
- `.env.production` - Environment variables only on production system

Alternatively, these values can be passed in using command-line parameters when running the API game server.
The command-line parameters override any values set in the `.env` files.

|      Variable       |    Command-line Flag    |      Required       | Default Value  | Description                                                                                                                                                                                                         |
| :-----------------: | :---------------------: | :-----------------: | :------------: | :------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
|        HOST         |     `--host`, `-h`      |         No          |   127.0.0.1    | IP address to use for running the API game server. If you use the `localhost` IP address, then you cannot connect to the API server from an external location. This must be an IP address and not a domain name.    |
|        PORT         |     `--port`, `-p`      |         No          |      3000      | Port number for the API game server.                                                                                                                                                                                |
|      USE_HTTPS      |      `--use-https`      |         No          |     false      | If true, then use HTTPS instead of HTTP for API requests. HTTPS encryption is performed using the [Rustls library](https://github.com/rustls/rustls).                                                               |
|      KEY_FILE       |      `--key-file`       | Only If `USE_HTTPS` |                | Private key file for Rustls. This should be an unencrypted `.pem` file.                                                                                                                                             |
|      CERT_FILE      |      `--cert-file`      | Only If `USE_HTTPS` |                | Certificate file for Rustls. This should be the unencrypted `.pem` file generated using the private key. For compatibility with some applications, this should be the full chain file and not just the certificate. |
|     JWT_SECRET      |  `--jwt-secret`, `-s`   |         No          |  _Hidden..._   | Secret value for signing the JSON Web Token                                                                                                                                                                         |
|      LUA_FILE       |      `--lua-file`       |         No          | `lua/game.lua` | Lua code file that contains the game engine code                                                                                                                                                                    |
| MIN_PLAYERS_NEEDED  | `--min-players-needed`  |         No          |       2        | Minimum number of players that must be registered to play the game. Must be >= 2 players.                                                                                                                           |
| MAX_PLAYERS_ALLOWED | `--max-players-allowed` |         No          |       8        | Maximum number of players that are allowed to compete in a single match. Must be >= MIN_PLAYERS_NEEDED.                                                                                                             |
| LOBBY_WAIT_SECONDS  | `--lobby-wait-seconds`  |         No          |       10       | Amount of time to wait before starting the game after the minimum number of players is reached. Cannot be less than 1 second.                                                                                       |
|   TICKS_PER_GAME    |   `--ticks-per-game`    |         No          |      180       | Number of total game engine "ticks" for a complete round in the game. Cannot be less than 30.                                                                                                                       |
|  SECONDS_PER_TICK   |  `--seconds-per-tick`   |         No          |       1        | Number of seconds between each game engine "tick". Must be at least 1 second.                                                                                                                                       |

<br />

## Code Structure

- [`/src`](/src) - All source code files for the API game server
- [`/lua`](/lua) - All source code files for the game engine logic written in Lua

Main files in the `/src` directory:

- [`main.rs`](/src/main.rs) - Entry point for the server application
- [`lib.rs`](/src/lib.rs) - Entry point for the shared library
- [`config.rs`](/src/config.rs) - Handle environment variables

Main folders in the `/src` directory:

- [`/actors`](/src/actors/) - Active objects used by Actix for handling the WebSocket registration and broadcasting logic
- [`/bin`](/src/bin) - Other utility programs defined by the game server
- [`/errors`](/src/errors) - Structures and functions for error handling across the application
- [`/game`](/src/game) - Structures and functions for interacting with the Lua game code for running the game engine
- [`/handlers`](/src/handlers) - All REST API handlers
- [`/jwt`](/src/jwt) - Structures and functions for parsing JSON Web Tokens for user authentication
- [`/protocol`](/src/protocol) - Types specific to the WebSocket communication protocol
- [`/utils`](/src/utils) - Miscellaneous helper functions

Main files in the `/lua` directory:

- [`game.lua`](/lua/game.lua) - Contains the main game engine logic
- [`playfield.lua`](/lua/playfield.lua) - Functions for generating the game playfield (walls and obstacles)
- [`weapon.lua`](/lua/weapon.lua) - Functions for building the different weapons in the game
- [`inspect.lua`](/lua/inspect.lua) - Copy of the [inspect.lua](https://github.com/kikito/inspect.lua) library to help with debugging.

**Note:** The API game server compiles both a shared library and a main executable.
Using this structure enables other [binary utilities](https://doc.rust-lang.org/cargo/guide/project-layout.html) (`/src/bin` directory) to access the data types and API handlers.

### Linting and Formatting

Rust provides a custom code formatter named `rustfmt`, which is configured in the `rustfmt.toml` file.
When working with Rust, try to install a rustfmt plugin to automatically format your code when saving to ensure a consistent style in the codebase.
For example, [VSCode](https://code.visualstudio.com/) provides good Rust integration through the following plugins:

- [Rust](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust)
- [Rust Grammar](https://marketplace.visualstudio.com/items?itemName=siberianmh.rust-grammar)
- [Rust Analyzer](https://marketplace.visualstudio.com/items?itemName=matklad.rust-analyzer)
- [vscode-rust](https://github.com/editor-rs/vscode-rust)

### FromRequest Trait

FromRequest is special trait used by [Actix Web](https://docs.rs/actix-web/3.3.2/actix_web/trait.FromRequest.html) that allows types to be referenced directly in the API handler.
Consider the API handler, which refers to the custom type `PlayerWebsocketToken`:

```rust
pub async fn connect_player(
  token: PlayerWebsocketToken,
  mediator: web::Data<Addr<GameMediatorActor>>,
  send_player_actions: web::Data<Sender<(Uuid, PlayerAction)>>,
  req: HttpRequest,
  payload: web::Payload,
) -> Result<HttpResponse, ServiceError> {
  // Code omitted
}
```

The following structures implement this trait in the server:

- `JWTToken`
- `JWTWebsocketToken`

### Error Handling

The `ServiceError` structure is a Rust enum that stores all errors in the system.
The `ErrorResponse` structure defines the error JSON format returned to the user.
ServiceError implements the [Actix `ResponseError`](https://docs.rs/actix-web/3.3.2/actix_web/trait.ResponseError.html) trait so it can be returned directly from API handlers.
Anytime an error is returned from an API handler, it is logged to the terminal.
On the production server, the `ErrorResponse` object does **NOT** return the `developer_notes` field, as it may contain sentivie information about the API server.
However, this field is still printed to the log file on the production server.

The error handling system also defines a few more structures used by the API server:

- `GameEngineError` - More detailed information about an error with the game engine logic
- `WebsocketError` - More detailed information about an error with the websocket communication protocol
- `GlobalErrorCodes` - Integer error codes used by the frontend for more finely-grained error handling

### API Handlers

General guidelines:

- Each API handler is a single [Rust async function](https://rust-lang.github.io/async-book/), and is defined in its own file
- All handler parameters must implement the `FromRequest` trait in [Actix Web](https://docs.rs/actix-web/3.3.2/actix_web/trait.FromRequest.html)
- Usually, API handlers return `HttpResponse` or empty data.
- If an error can occur, use the `Result<HttpResponse, ServiceError>`
- API handlers use the Permissions object to check for user permissions
- If JSON data is passed to the API server, use the Validator library to ensure the data is correct

[Actix Web](https://docs.rs/actix-web/3.3.2/actix_web/index.html) defines special types of `FromRequest` objects to assist with writing API handlers:

- `web::Json<>` - Parse the body of the request as JSON data
- `web::Path<>` - Read parameters from the path, such as string or integer identifiers
- `web::Query<>` - Parse parameters in the URL query string

General guidelines for JSON structures:

- Structures that parse data from the user should implement the Deserialize trait
- Structures that return data to the user should implement the Serialize trait
- All structure fields should be renamed to camelCase using #[serde(rename_all = "camelCase")]

All API routes are defined in `src/main.rs`.
This is handled by the Actix Web framework, which provides the following types of objects for defining routes:

- `.route()` - Specify a route using a string and a HTTP method
- `web::scope()` - Define a new subpath in the route
- `web::resource()` - Define a single path which supports multiple HTTP methods

Routes in the API server define parameters using brackets `{}`, such as `/api/users/{userId}/roles`

Most API handlers follow CRUD rules for naming and function (Create, Read, Update, Delete)
In general, HTTP methods work as follows:

- `GET` - Fetch a resource
- `POST` - Create a new resource
- `PATCH` - Modify various properties of a resource
- `PUT` - Replace a resource (Such as with file upload)
- `DELETE` - Delete a resource

### Actors

The game server uses a framework called [Actix](https://github.com/actix/actix) to handle the asynchronous communication between the clients and game engine.
Actix is an active object framework, where objects (named "actors" in the framework) run asynchronously and can send messages to one another.
_Incidentally, the web framework, [Actix Web](https://actix.rs/), is also built on the Actix library._

In Actix, "actors" must implement the [`Actor`](https://docs.rs/actix/latest/actix/trait.Actor.html) trait, which has overloadable methods for when an actor is started, stopping, and stopped.
Any message objects must implement the [`Message`](https://docs.rs/actix/latest/actix/trait.Message.html) trait, which defines a return value for the message.
The Actix library provides macro to automatically implement the message trait without requiring a bunch of boiler
To handle a message, actors must implement the [`Handler`](https://docs.rs/actix/latest/actix/trait.Handler.html) trait, which defines the logic for when a message is sent to an actor.
For example, we can define a message `PlayerKilled` which notifies an actor that a player has died:

```rust
struct MyActor;

impl Actor for MyActor {
  type Context = Context<Self>;

  fn started(&mut self, ctx: &mut Self::Context) {
    // Run when actor is started...
  }
}

#[derive(Debug, Clone, Message)]
#[rtype(result = "()")]
pub struct PlayerKilled {
  player_id: Uuid,
}

impl Handler<PlayerKilled> for MyActor {
  type Result = ();

  fn handle(&mut self, player_killed: PlayerKilled, ctx: &mut Self::Context) -> Self::Result {
    // Implement the logic ...
  }
}
```

The game server defines the following actors:

- `WebsocketActor` - Handles a single player client WebSocket connection. It parses JSON and sends any state updates to the `GameMediatorActor`
- `ViewerActor` - Same function as the WebsocketActor, but is only for viewer connections. Any attempt to send a game action (like registering or movement) returns an error.
- `GameMediatorActor` - Handles registration and gameplay logic. This includes things like which players are still alive, ticking the game engine, handling a game over, etc.

### Lua Code

the [`GamePlayer`](/src/game/game_player.rs) struct defines all of the logic for running the Lua code.
When the server is first started, it loads the Lua file defined by the `LUA_FILE` configuration variable.
It adds the parent folder of the `LUA_FILE` to the path, then executes the Lua code to define the game engine functions.
The GamePlayer struct encapsulates all logic of interacting with the GameMediatorActor and handling game ticks.

The Lua game engine code needs to define two functions:

```lua
-- Called once to initialize the game engine
--   ctx is the Context variable (explained below)
--   players is a string UUID array of the player order
function Init(ctx, players)

end

-- Called each "tick" to update the game engine
--   ctx is the Context variable (explained below)
--   actions is a map<UUID, action object> (see the protocol document for all action objects)
function Update(ctx, actions)

end
```

The `ctx` variable provides several useful Lua functions for communicating with the game server:

- `notifyPlayerKilled(playerID)` - Notify the server that the given Player UUID has been killed
- `getPlayerOrder()` - Returns a string UUID array with the order that player actions should be executed
- `getPlayersRemaining()` - Returns a `map<UUID, true>` of the alive players in the game (Lua equivalent of a set)
- `getTicksLeft()` - Returns (ticks left, total ticks in game) as numeric values

Both the Init and Update functions need to return the next game state.
See the [Protocol Document](Protocol.md) for details on the game state data type.

### Miscellaneous Objects and Functions

- `new_safe_uuid_v4()` - Since UUIDs are represented as a base-64 string, it may be possible for a UUID to contain a curse word. This method filters the most common types of curse words and curse variants.

<br />

## Included Binaries

The game server includes two utilities for the server administrator:

- `generate_token` - Generate a JWT for players to connect to the game server
- `test_game_code` - Sends random actions to the Lua game code to test for any crashes

These can be run using the command:

```bash
cargo run --bin generate_token -- <<<Parameters>>>
```

```bash
cargo run --bin test_game_code -- <<<Parameters>>>
```

Replace the `<<<Parameters>>>` with any command-line parameters you wish to pass into the program.
_Notice the double minus `--` and space before the list of parameters. This is required so Cargo doesn't mistake Cargo parameters with executable parameters._

### Generate Token Executable

The `generate_token` executable is used by the server admin to generate new JSON Web Tokens for players trying to play on the server.
To show the list of options, run:

```bash
cargo run --bin generate_token -- -h
```

Which will print:

```text
Generate a JSON web token for the game server

USAGE:
    generate_token <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    help      Prints this message or the help of the given subcommand(s)
    player    Generate a player JWT
    viewer    Generate a viewer JWT
```

The program has subcommands `player` and `viewer` to generate player JWTs and viewer JWTs respectively.
Once again, you can use the `-h` flag to show options for the subcommands:

```bash
cargo run --bin generate_token -- player -h
cargo run --bin generate_token -- viewer -h
```

Which has the options:

```text
Generate a player JWT

USAGE:
    generate_token player [OPTIONS] --jwt-secret <jwt-secret> --name <name>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -d, --duration <duration>        Duration for the JWT as an English string [default: 1 year]
    -i, --id <id>                    Player UUID (Picks a random one if omitted))
    -s, --jwt-secret <jwt-secret>    JSON Web Token secret [env: JWT_SECRET]
    -n, --name <name>                Player name or alias
```

```text
Generate a viewer JWT

USAGE:
    generate_token viewer [OPTIONS] --jwt-secret <jwt-secret>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -d, --duration <duration>        Duration for the JWT as an English string [default: 1 year]
    -i, --id <id>                    Viewer UUID (Picks a random one if omitted))
    -s, --jwt-secret <jwt-secret>    JSON Web Token secret [env: JWT_SECRET]
```

To generate either token, the server admin needs to know the `JWT_SECRET` environment variable.
Like the server, this can be read from the `.env` file or passed in using the command-line.
_Note: `generate_token` ignores the `.env.development` and `.env.production` files, it only recognizes `.env`._

Both tokens also require a duration parameter, which can be passed as an English string.
By default, it is set to `1 year`, but some other possible values could include:

- `5 months`
- `2 weeks`
- `10 days`

The time is relative to the current date.

Finally, both the player and viewer tokens require a unique UUID.
If this parameter is omitted, then the `generate_token` will pick a random UUID for the token.
Otherwise, if you are renewing an existing token, be sure to use the same UUID.
The player token also requires a name, which is used for the player alias for people watching the game.

### Test Game Code

This is a simple executable used to check your Lua code.
Since Lua is an interpreted language, common coding errors are hard to detect without running the code first.
So, this program is designed to "play" the Lua code as though it was a real game match.
The tester spawns players and simulates random player actions to check for any bugs, using the same game playing logic as the real server.
This is **not** a comprehensive test suite but rather serves as a simple way to verify the Lua code works for the most common cases.

To see the list of command-line options, run:

```bash
cargo run --bin test_game_code -- -h
```

```text
Simple test of the game engine code to detect runtime bugs

USAGE:
    test_game_code [FLAGS] [OPTIONS]

FLAGS:
    -h, --help          Prints help information
    -d, --show-debug    If set, also shows the debug output
    -V, --version       Prints version information

OPTIONS:
        --lua-file <lua-file>                Lua file containing the game engine code [env: LUA_FILE=]  [default:
                                             lua/game.lua]
        --num-players <num-players>          Number of players in the game [default: 4]
        --ticks-per-game <ticks-per-game>    Number of total "ticks" for a complete round in the game [env:
                                             TICKS_PER_GAME=]  [default: 180]
```

Like the `generate_token` executable, some of the parameters can be read from the `.env` file.
_Note: it ignores the `.env.development` and `.env.production` files and only recognizes `.env`._

The `--lua-file` is required but by default it uses the provided Lua game file in `lua/game.lua`.
The `--num-players` parameter can also be varied to simulate different sized games.
Ticks are run as fast as possible since we don't need to wait for WebSocket messages to pick the next action (it is done by the simulator).
When running the Lua code, the executable prints out helpful logging messages for debugging any code problems.
