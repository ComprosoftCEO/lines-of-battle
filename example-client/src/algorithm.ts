import { WebSocket } from "ws";
import {
  GameServerRequest,
  GameServerResponse,
  ResponseType,
  RequestType,
  GameState,
  PlayerAction,
} from "./protocol";
import { AttackAdjacentPlayerState } from "./state/attackAdjacentPlayer";
import { AttackPlayerState } from "./state/attackPlayer";
import { FindWeaponState } from "./state/findWeapon";
import { MoveOutOfWayState } from "./state/moveOutOfWay";
import { AlgorithmState } from "./state/state";

/**
 * Encapsulates the logic of our algorithm
 */
export class Algorithm {
  private ws: WebSocket;

  private playerId: string;
  private isKilled = false;

  constructor(playerId: string, ws: WebSocket) {
    this.ws = ws;
    this.playerId = playerId;

    ws.onopen = (event) => {
      this.onInit();
    };

    ws.onmessage = (event) => {
      try {
        const data = JSON.parse(event?.data as string);
        this.onMessage(data);
      } catch (e) {
        console.error("Failed to parse JSON message: ", e);
      }
    };

    ws.onerror = (event) => {
      console.error("WebSocket error: ", event);
    };
  }

  /**
   * Sent a JSON message to the game server
   */
  private sendMessage(msg: GameServerRequest): void {
    this.ws.send(JSON.stringify(msg));
  }

  /**
   * Called when the connection is first established
   */
  private onInit(): void {
    // Try to register for the next game
    this.sendMessage({ type: RequestType.Register });
  }

  /**
   * Called when a message is received
   * @param msg Message received
   */
  private onMessage(msg: GameServerResponse): void {
    switch (msg.type) {
      // Register for the next game once the game ends
      case ResponseType.GameEnded: {
        this.isKilled = false;
        this.sendMessage({ type: RequestType.Register });
        break;
      }

      // Indicate if the player is killed
      case ResponseType.PlayerKilled:
        if (msg.id === this.playerId) {
          this.isKilled = true;
        }
        break;

      // Pick the player action
      case ResponseType.GameInitialized:
      case ResponseType.NextState: {
        if (this.isKilled) {
          return; // No need to perform an action if killed
        }

        const action = this.chooseNextAction(msg.gameState);
        if (action !== null) {
          this.sendMessage(action);
        }
        break;
      }
    }
  }

  /**
   * Algorithm to pick the next player action
   * Returns "null" to not choose an action
   */
  private chooseNextAction(state: GameState): PlayerAction | null {
    let algorithm: AlgorithmState = { chooseNextAction: () => null };
    const weapon = state.players[this.playerId].weapon;
    if (weapon !== undefined) {
      algorithm = new AttackPlayerState(this.playerId);
    } else {
      algorithm = new FindWeaponState(this.playerId);
    }

    // Always try to attack adjacent players first
    algorithm = new AttackAdjacentPlayerState(
      this.playerId,
      new MoveOutOfWayState(this.playerId, algorithm)
    );

    return algorithm.chooseNextAction(state);
  }
}
