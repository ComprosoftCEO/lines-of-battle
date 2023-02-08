# Game Server Protocol

The following document details the protocol for interacting with the game server.

## Connecting to the Server

Communication with the game server is implemented using the [WebSocket Protocol](https://www.rfc-editor.org/rfc/rfc6455).
When connecting a WebSocket client to the server, they need to specify the following subprotocols:

1. `game-server`
2. JSON Web Token (JWT) received from the server admin

### Player Clients

Player clients are allowed to register for the game and control their player inside the game.
This connection must use a player JSON Web Token (not a viewer token).
**Important Note: each player JWT can only have one connection to the server at a time.**
If a player becomes disconnected, they can reconnect using the same JWT.
However, trying to initialize a second connection with the same JWT will result in the connection being terminated.

The websocket connection route for a player client is given by:

```text
/api/v1/play
```

_For example, a server running locally without HTTPS would use `ws://localhost:53700/api/v1/play`_

### Viewer Clients

Viewer clients still get game state updates, but are not allowed to register for the game nor control a player.
This connection must use a viewer JSON Web Token (not a player token).
Unlike a player client, a viewer client has no connection limit per JWT.

The websocket connection route for a viewer client is given by:

```text
/api/v1/view
```

_For example, a server running locally without HTTPS would use `ws://localhost:53700/api/v1/view`_

### JWT TypeScript Types

Each JWT (player and viewer) is associated with a unique [UUID](https://en.wikipedia.org/wiki/Universally_unique_identifier).
However, UUIDs in JSON are just string types, so the type alias is used:

```typescript
type Uuid = string;
```

Each player JWT is also given a name alias that can be shown in-game instead of the UUID:

```typescript
interface JWTPlayerData {
  name: string;
}
```

Finally, we use the `Map<K, V>` to represent a JavaScript object that maps string `K` to a given `V` value type.
In TypeScript, this is more accurately represented as `Record<K, V>`, but we use Map for readability.

<br />

## Communication

All communication between the server and client is handled using JSON text messages.
The general format for JSON messages is given by:

```typescript
interface Message {
  type: string;

  // Other payload fields here ...
}
```

If an error occurs when communicating with the server, it will return the following JSON message:

```typescript
interface Error {
  type: "error";
  description: string;
  errorCode: GlobalErrorCode;

  // Only included in a debug build of the game server
  //  Used by the game server developers for additional debugging
  developerNotes?: string;
}

enum GlobalErrorCode {
  UnknownError = 0,
  MissingAppData,
  JSONPayloadError,
  FormPayloadError,
  URLPathError,
  QueryStringError,
  StructValidationError,
  InvalidJWTToken,
  GameEngineError,
  GameEngineCrash,
  WebsocketError,
  NotRegistered,
  FailedToRegister,
  FailedToUnregister,
  AlreadyConnected,
  CannotSendAction,
}
```

The error description is a nice "printable" string explaining the error that ocurred.
The global error code can be used by clients to perform additional logic checks.
**Expect the error code list to be updated throughout the server's development.**

<br />

## Server States

The game server has four main states:

- [Registration](#registration) - Player clients can connect and register for the game
- [Initializing](#game-initialization) - Registration is closed, but the game has not started yet
- [Running](#game-running) - Player clients can control their player in the game
- [Fatal Error](#fatal-error) - Game server has crashed and needs to be restarted

```typescript
enum ServerState {
  Registration = "registration",
  Initializing = "initializing",
  Running = "running",
  FatalError = "fatalError",
}
```

Unregistered player clients can only connect to the server during the `Registration` state.
Attempting to connect unregistered clients during `Initializing` or `Running` will terminate the connection.
If the server gets into a `FatalError` state (albeit unlikely), all existing connections (and any attempts at new connections) will be terminated, and the server must be manually restarted.

<br />

## Registration

When first run, the game server starts in the `Registration` state.
After a player client connects to the server, they need to send the `Registration` message to indicate their desire to participate in the next game round.
Alternatively, an already registered player client can send a message to unregister from the game:

```typescript
interface RegisterMessage {
  type: "register";
}

interface UnregisterMessage {
  type: "unregister";
}
```

Once the server receives enough registrations, it begins a countdown process before starting the game.
New player clients can still register during this time, but when the timer reaches 0 the server enters the `Initializing` state.
If enough player clients unregister during this time, the clock will stop and reset back to its default value.
The server administrator can configure the minimum number of players required and countdown time before starting the game.

See [Server Events](#server-events) for more details about messages that can be broadcasted from the server.

<br />

## Game Initialization

During this state, new player clients are not allowed to register for the game; the current list of players is now the official list playing in the game.
The game server generates a new game world and spawns players into the world.
The game server also generates a random movement order for players in the game.
Once the game is ready to begin, the server broadcasts out a message to all player clients with the initial state of the game world.

See [Server Events](#server-events) for more details about messages that can be broadcasted from the server.

<br />

## Game Running

When the game is running, alive player clients need to send one action to the server every game "tick" to move their player in the game.
Only the first action is stored; subsequent actions sent during the same game tick will return an error.
If a client does not send an action before the time-window expires, then their player doesn't move that round.
After the tick time window time window has passed, the game server will grab the current list of actions and update the game state in the specified player order.
By default, a game tick occurs every real-world second, but this can be changed by the server administrator.

Each player action has an optional string `tag` field that can be used by the client.
When the server returns the [Next State Message](#next-state), it includes the list of actions taken during the current game tick.
The `tag` field is passed through transparently to provide a mechanism for the client to uniquely track each action sent to the server.

When a player is eliminated from the game ([Player Killed](#player-killed) message), they are no longer allowed to take any actions.
However, they can stay connected to the server and receive broadcast update messages.
When the game ends (with the [Game Ended](#game-ended) message), the player can register for the next round.

See [Server Events](#server-events) for more details about messages that can be broadcasted from the server.

### Allowed Actions

```typescript
type PlayerAction = MoveAction | AttackAction | DropWeaponAction;

interface MoveAction {
  type: "move";
  direction: Direction;
  tag?: string;
}

interface AttackAction {
  type: "attack";
  direction: Direction;
  tag?: string;
}

interface DropWeaponAction {
  type: "dropWeapon";
  tag?: string;
}

enum Direction {
  Up = "up",
  Down = "down",
  Left = "left",
  Right = "right",
}
```

### Game State Type

The `GameState` type defines all details about the current game state in the world.
**Important Note:** The `players` field only lists the players remaining in the game, not any killed players.

```typescript
interface GameState {
  playfield: PlayfieldObject[][];
  players: Map<Uuid, Position<PlayerDetails>>;
  weapons: Position<Weapon>[];
  items: Position<Item>[]; // Unused right now
}

// Has a (row, col) position in the playfield
interface Position<T> extends T {
  row: number;
  col: number;
}

enum PlayfieldObject {
  Empty = 0,
  Wall = 1,
}

interface PlayerDetails {
  health: number;
  weapon?: Weapon;
}

// List of all weapons
type Weapon = LaserGunWeapon;

interface LaserGunWeapon {
  type: "laserGun";
  ammo: number;
  damage: number;
}

// List of all items
type Item = {}; // Unused right now
```

Note the use of the TypeScript `Position<T>` generic type, which is used to give (row, col) properties to a type.
For example, `Position<PlayerDetails>` is the object:

```typescript
type Position<PlayerDetails> = {
  row: number;
  col: number;
  health: number;
  weapon?: Weapon;
};
```

### Gameplay Details

Players in the game are allowed to move to orthogonal tiles, assuming there isn't a wall in the way or another player on the same tile.
If a player steps on a tile that also has a weapon, they pick up the weapon.
If a player already has a weapon when they pick up another weapon, the old weapon is dropped onto the tile.
Using the drop weapon action allows a player to drop their held weapon and pick up any existing weapon on the tile (swap).

Players can attack other players in two ways:
First, they can be on an orthogonal tile and attack the player directly (for 1 health damage).
Otherwise, if the player is holding a weapon, the attack action will fire the held weapon.
At a minimum, each weapon has an `ammo` and `damage` field. If a weapon runs out of ammo, it is removed from the game.

Laser gun bullets cannot move through walls or other players, but have an infinite range.
They deal 2 damage to a player but only have 1 ammo.

<br />

## Fatal Error

A game can enter a fatal error state if the game engine code crashes.
All existing connections are terminated and any new connections will also be terminated.
The game server will need to be manually restarted if it enters this error state.

<br />

## Queries

Player clients and viewer clients can send messages to the server to query details about the current game state.

### Get Current State

**Allowed by:** player, viewer

This request returns the current state in the server. It can be sent at any time.

```typescript
interface GetServerStateRequest {
  type: "getServerState";
}

interface GetServerStateResponse {
  type: "serverState";
  state: ServerState;
}
```

### Get Registered Players

**Allowed by:** player, viewer

This request returns the list of players registered in the server.
If the game is running, it also sends the order of players.
This query can be sent at any time.

```typescript
interface GetRegisteredPlayersRequest {
  type: "getRegisteredPlayers";
}

interface GetRegisteredPlayersResponse {
  type: "registeredPlayers";
  players: Map<Uuid, JWTPlayerData>;
  playerOrder?: Uuid[];
}
```

_More queries may be added in the future..._

<br />

## Server Events

The game server has a variety of messages that it can broadcast in response to different real-time events.

### Waiting on Players

**Sent to:** All players and all viewers

This message is sent anytime a player registers (or unregisters) from the game **and** the number of registered users is less than the minumum required to start.
It returns the current list of registered players and the minimum number of players required to actually start the game.
If the number of registered players is more than the minimum number required, it will periodically send the [Game Starting Soon](#game-starting-soon) message instead.

```typescript
interface WaitingOnPlayers {
  type: "waitingOnPlayers";
  players: Map<Uuid, JWTPlayerData>;
  minPlayersNeeded: number;
  maxPlayersAllowed: number;
}
```

### Game Starting Soon

**Sent to:** All players and all viewers

The server will send out this message every second once there are enough players registered to start the game.
Additionally, this message is sent anytime a player registers (or unregisters) from the game **and** there are enough players to start the game.
It returns the current list of registered players and the minimum number of players required to actually start the game.
If the number of registered players becomes less than the minimum number required, it will send the [Waiting On Players](#waiting-on-players) message instead.

```typescript
interface GameStartingSoon {
  type: "gameStartingSoon";
  players: Map<Uuid, JWTPlayerData>;
  minPlayersNeeded: number;
  maxPlayersAllowed: number;
  secondsLeft: number;
}
```

### Game Starting

**Sent to:** All players and all viewers

This message is sent after the `GameStartingSoon` seconds left counts down to 0.
It indicates that the server is generating the game world.
This message returns the official list of players registered in the game and the player turn order.

```typescript
interface GameStarting {
  type: "gameStarting";
  players: Map<Uuid, JWTPlayerData>;
  playerOrder: Uuid[];
}
```

### Game Initialized

**Sent to:** All players and all viewers

This message indicates that the game has now officially started.
It returns the initial game world state and the number of "ticks" left in the game.

```typescript
interface GameInitialized {
  type: "init";
  gameState: GameState;
  ticksLeft: number;
  secondsPerTick: number;
}
```

### Next State

**Sent to:** All players and all viewers

This message returns the next state of the game world and the number of "ticks" left in the game.
It also returns a map of the actions that were performed by the players during the last tick.
Note that the map may not contain an entry for every player if a player didn't take an action during the last game tick.

```typescript
interface NextState {
  type: "nextState";
  gameState: GameState;
  actionsTaken: Map<Uuid, PlayerAction>;
  ticksLeft: number;
  secondsPerTick: number;
}
```

### Player Killed

**Sent to:** All players and all viewers

Sent whenever a player's health drops below 0.

```typescript
interface PlayerKilled {
  type: "playerKilled";
  id: Uuid;
}
```

The player is eliminated from the game but they can stay connected to the server.
When the game ends (with the [Game Ended](#game-ended) message), the player can register for the next round.

### Game Ended

**Sent to:** All players and all viewers

There are three conditions when the game round ends:

1. There is only one player remaining in the arena (winners list will have one player listed)
2. The game timer runs out (winners list will have two or more players listed, they all tie this round)
3. All players are killed (winners list will have zero players listed)

After this message is sent, the game server goes back into the `Registration` state and player clients can register for the next round.

```typescript
interface GameEnded {
  type: "gameEnded";
  winners: Uuid[];
  gameState: GameState;
  actionsTaken: Map<Uuid, PlayerAction>;
}
```
