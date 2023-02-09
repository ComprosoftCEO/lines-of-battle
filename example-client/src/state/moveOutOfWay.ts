import { Direction, GameState, PlayerAction, RequestType } from "../protocol";
import { isColBlocked, isRowBlocked, pickRandom } from "../utils";
import { AlgorithmState } from "./state";

/**
 * Move out of the way if you don't have a laser gun weapon
 */
export class MoveOutOfWayState implements AlgorithmState {
  private playerId: string;
  private nextAlgorithm: AlgorithmState;

  constructor(playerId: string, nextAlgorithm: AlgorithmState) {
    this.playerId = playerId;
    this.nextAlgorithm = nextAlgorithm;
  }

  chooseNextAction(state: GameState): PlayerAction | null {
    const player = state.players[this.playerId];

    // Skip this step if the player has a weapon
    if (player.weapon !== undefined) {
      return this.nextAlgorithm.chooseNextAction(state);
    }

    // Determine if any other players are in same row or column has a weapon
    const directions = new Set<Direction>();
    for (const p of Object.values(state.players)) {
      if (p === player) {
        continue;
      }
      if (p.weapon === undefined) {
        // We only care about players with weapons
        continue;
      }

      if (p.row === player.row) {
        // Make sure nothing blocking the player, or else we don't care
        if (isRowBlocked(state, p.row - 1, p.col - 1, player.col - 1)) {
          continue;
        }

        const direction = p.col > player.col ? Direction.Right : Direction.Left;
        directions.add(direction);
      } else if (p.col === player.col) {
        // Make sure nothing is blocking the player
        if (isColBlocked(state, p.col - 1, p.row - 1, player.row - 1)) {
          continue;
        }

        const direction = p.row > player.row ? Direction.Down : Direction.Up;
        directions.add(direction);
      }
    }

    // ===========================
    //  Handle a left-right dodge
    // ===========================
    if (directions.has(Direction.Up) || directions.has(Direction.Down)) {
      // Make sure it actually makes sense to dodge
      //   We can't dodge two axes, so ignore if not worth trying
      if (
        !(directions.has(Direction.Left) || directions.has(Direction.Right))
      ) {
        // Only pick from directions where there is no wall or player
        const directions: Direction[] = [];
        if (
          !isRowBlocked(
            state,
            player.row - 1,
            player.col - 1 - 1,
            player.col - 1 - 2
          )
        ) {
          directions.push(Direction.Left);
        }
        if (
          !isRowBlocked(
            state,
            player.row - 1,
            player.col - 1 + 1,
            player.col - 1 + 2
          )
        ) {
          directions.push(Direction.Right);
        }

        if (directions.length > 0) {
          return {
            type: RequestType.Move,
            direction: pickRandom(directions),
          };
        }
      }
    }

    // ===========================
    //   Handle an up-down dodge
    // ===========================
    if (directions.has(Direction.Left) || directions.has(Direction.Right)) {
      // Make sure it actually makes sense to dodge
      //   We can't dodge two axes, so ignore if not worth trying
      if (!(directions.has(Direction.Up) || directions.has(Direction.Down))) {
        // Only pick from directions where there is no wall or player
        const directions: Direction[] = [];
        if (
          !isColBlocked(
            state,
            player.col - 1,
            player.row - 1 - 1,
            player.row - 1 - 2
          )
        ) {
          directions.push(Direction.Up);
        }
        if (
          !isColBlocked(
            state,
            player.col - 1,
            player.row - 1 + 1,
            player.row - 1 + 2
          )
        ) {
          directions.push(Direction.Down);
        }

        if (directions.length > 0) {
          return {
            type: RequestType.Move,
            direction: pickRandom(directions),
          };
        }
      }
    }

    return this.nextAlgorithm.chooseNextAction(state);
  }
}
