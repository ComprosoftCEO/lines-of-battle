export type Uuid = string;

export interface JWTPlayerData {
  name: string;
}

// ================================
//      Game Server Request
// ================================

export enum RequestType {
  Register = "register",
  Unregister = "unregister",
  Move = "move",
  Attack = "attack",
  DropWeapon = "dropWeapon",
  GetServerState = "getServerState",
  GetRegisteredPlayers = "getRegisteredPlayers",
}

export type GameServerRequest =
  | RegisterMessage
  | UnregisterMessage
  | PlayerAction
  | GetServerStateRequest
  | GetRegisteredPlayersRequest;

export interface RegisterMessage {
  type: RequestType.Register;
}

export interface UnregisterMessage {
  type: RequestType.Unregister;
}

export type PlayerAction = MoveAction | AttackAction | DropWeaponAction;

export interface MoveAction {
  type: RequestType.Move;
  direction: Direction;
  tag?: string;
}

export interface AttackAction {
  type: RequestType.Attack;
  direction: Direction;
  tag?: string;
}

export interface DropWeaponAction {
  type: RequestType.DropWeapon;
  tag?: string;
}

export enum Direction {
  Up = "up",
  Down = "down",
  Left = "left",
  Right = "right",
}

export interface GetServerStateRequest {
  type: RequestType.GetServerState;
}

export interface GetRegisteredPlayersRequest {
  type: RequestType.GetRegisteredPlayers;
}

// ================================
//      Game Server Response
// ================================

export enum ResponseType {
  Error = "error",
  WaitingOnPlayers = "waitingOnPlayers",
  GameStartingSoon = "gameStartingSoon",
  GameStarting = "gameStarting",
  GameInitialized = "init",
  NextState = "nextState",
  PlayerKilled = "playerKilled",
  GameEnded = "gameEnded",
  GetServerState = "serverState",
  GetRegisteredPlayers = "registeredPlayers",
}

export type GameServerResponse =
  | ErrorResponse
  | WaitingOnPlayers
  | GameStartingSoon
  | GameStarting
  | GameInitialized
  | NextState
  | PlayerKilled
  | GameEnded
  | GetServerStateResponse
  | GetRegisteredPlayersResponse;

export interface ErrorResponse {
  type: ResponseType.Error;
  description: string;
  errorCode: GlobalErrorCode;

  // Only included in a debug build of the game server
  //  Used by the game server developers for additional debugging
  developerNotes?: string;
}

export enum GlobalErrorCode {
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

export interface WaitingOnPlayers {
  type: ResponseType.WaitingOnPlayers;
  players: Record<Uuid, JWTPlayerData>;
  minPlayersNeeded: number;
  maxPlayersAllowed: number;
}

export interface GameStartingSoon {
  type: ResponseType.GameStartingSoon;
  players: Record<Uuid, JWTPlayerData>;
  minPlayersNeeded: number;
  maxPlayersAllowed: number;
  secondsLeft: number;
}

export interface GameStarting {
  type: ResponseType.GameStarting;
  players: Record<Uuid, JWTPlayerData>;
  playerOrder: Uuid[];
}

export interface GameInitialized {
  type: ResponseType.GameInitialized;
  gameState: GameState;
  ticksLeft: number;
  secondsPerTick: number;
}

export interface NextState {
  type: ResponseType.NextState;
  gameState: GameState;
  actionsTaken: Record<Uuid, PlayerAction>;
  ticksLeft: number;
  secondsPerTick: number;
}

export interface PlayerKilled {
  type: ResponseType.PlayerKilled;
  id: Uuid;
}

export interface GameEnded {
  type: ResponseType.GameEnded;
  winners: Uuid[];
  gameState: GameState;
  actionsTaken: Record<Uuid, PlayerAction>;
}

export interface GetServerStateResponse {
  type: ResponseType.GetServerState;
  state: ServerState;
}

export enum ServerState {
  Registration = "registration",
  Initializing = "initializing",
  Running = "running",
  FatalError = "fatalError",
}

export interface GetRegisteredPlayersResponse {
  type: ResponseType.GetRegisteredPlayers;
  players: Record<Uuid, JWTPlayerData>;
  playerOrder?: Uuid[];
}

// ================================
//      Game State Object
// ================================
export interface GameState {
  playfield: PlayfieldObject[][];
  players: Record<Uuid, Position<PlayerDetails>>;
  weapons: Position<Weapon>[];
  items: Position<Item>[]; // Unused right now
}

// Has a (row, col) position in the playfield
export type Position<T> = T & {
  row: number;
  col: number;
};

export enum PlayfieldObject {
  Empty = 0,
  Wall = 1,
}

export interface PlayerDetails {
  health: number;
  weapon?: Weapon;
}

// List of all weapons
export type Weapon = LaserGunWeapon;

export enum WeaponType {
  LaserGun = "laserGun",
}

export interface LaserGunWeapon {
  type: WeaponType.LaserGun;
  ammo: number;
  damage: number;
}

// List of all items
export type Item = never; // Unused right now
