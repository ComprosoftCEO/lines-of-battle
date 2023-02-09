import { Direction, GameState, PlayerAction, RequestType } from "../protocol";
import { pickRandom } from "../utils";
import { AlgorithmState } from "./state";

/**
 * Try to attack an adjacent player
 */
export class AttackAdjacentPlayerState implements AlgorithmState {
  private playerId: string;
  private nextAlgorithm: AlgorithmState;

  constructor(playerId: string, nextAlgorithm: AlgorithmState) {
    this.playerId = playerId;
    this.nextAlgorithm = nextAlgorithm;
  }

  chooseNextAction(state: GameState): PlayerAction | null {
    const player = state.players[this.playerId];

    // Find direction of all adjacent players
    const adjacent: Direction[] = [];
    for (const p of Object.values(state.players)) {
      if (p === player) {
        continue;
      }

      if (p.row === player.row - 1 && p.col === player.col) {
        adjacent.push(Direction.Up);
      } else if (p.row === player.row + 1 && p.col === player.col) {
        adjacent.push(Direction.Down);
      } else if (p.row === player.row && p.col === player.col - 1) {
        adjacent.push(Direction.Left);
      } else if (p.row === player.row && p.col === player.col + 1) {
        adjacent.push(Direction.Right);
      }
    }

    // Always randomly attack an adjacent player
    if (adjacent.length > 0) {
      return { type: RequestType.Attack, direction: pickRandom(adjacent) };
    }

    // Otherwise, defer to the next algorithm
    //  (Chain of Command design pattern)
    return this.nextAlgorithm.chooseNextAction(state);
  }
}
