import { Direction, GameState, PlayerAction, RequestType } from "../protocol";
import { AlgorithmState } from "./state";
import { Graph, astar } from "../astar";
import {
  buildAStarWeights,
  getAStarMove,
  randomMoveOrAttack,
  isRowBlocked,
  isColBlocked,
} from "../utils";
import { PriorityQueue } from "../priorityQueue";

interface PriorityQueueEntry {
  direction: Direction;
  distanceAway: number;
  hasWeapon: boolean;
}

function compareQueueEntry(
  a: PriorityQueueEntry,
  b: PriorityQueueEntry
): boolean {
  // Objects with a laser gun trump all others
  if (a.hasWeapon && !b.hasWeapon) {
    return true;
  } else if (!a.hasWeapon && b.hasWeapon) {
    return false;
  }

  // Otherwise, find the closest distance
  return a.distanceAway < b.distanceAway;
}

/**
 * Find a player to attack using the laser gun
 */
export class AttackPlayerState implements AlgorithmState {
  private playerId: string;

  constructor(playerId: string) {
    this.playerId = playerId;
  }

  chooseNextAction(state: GameState): PlayerAction | null {
    const player = state.players[this.playerId];

    // Determine if any other players are in same row or column
    const queue = new PriorityQueue<PriorityQueueEntry>(compareQueueEntry);
    for (const p of Object.values(state.players)) {
      if (p === player) {
        continue;
      }

      if (p.row === player.row) {
        // Make sure nothing is blocking the player
        if (isRowBlocked(state, p.row - 1, p.col - 1, player.col - 1)) {
          continue;
        }

        const direction = p.col > player.col ? Direction.Right : Direction.Left;
        queue.push({
          direction,
          distanceAway: Math.abs(p.col - player.col),
          hasWeapon: p.weapon !== undefined,
        });
      } else if (p.col === player.col) {
        // Make sure nothing is blocking the player
        if (isColBlocked(state, p.col - 1, p.row - 1, player.row - 1)) {
          continue;
        }

        const direction = p.row > player.row ? Direction.Down : Direction.Up;
        queue.push({
          direction,
          distanceAway: Math.abs(p.row - player.row),
          hasWeapon: p.weapon !== undefined,
        });
      }
    }

    // Shoot at the robot with the highest priority
    if (queue.size() > 0) {
      return { type: RequestType.Attack, direction: queue.pop().direction };
    }

    // Okay, nobody in orthographic lines, so just find the closest player to attack
    let shortestPath: null | { x: number; y: number }[] = null;
    const graph = new Graph(buildAStarWeights(state.playfield));
    for (const p of Object.values(state.players)) {
      if (p === player) {
        continue;
      }

      const path = astar.search(
        graph,
        graph.grid[player.row - 1][player.col - 1],
        graph.grid[p.row - 1][p.col - 1]
      );

      if (path.length === 0) {
        continue; // Ignore if currently on the tile (should not happen, I hope)
      }

      if (shortestPath === null || shortestPath.length > path.length) {
        shortestPath = path;
      }
    }

    if (shortestPath === null) {
      // No shortest path (should not happen), randomly move or attack
      return randomMoveOrAttack();
    }

    // Figure out the direction to move
    return getAStarMove(player, shortestPath);
  }
}
