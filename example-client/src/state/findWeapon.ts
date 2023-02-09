import { GameState, PlayerAction } from "../protocol";
import { AlgorithmState } from "./state";
import { Graph, astar } from "../astar";
import { buildAStarWeights, getAStarMove, randomMoveOrAttack } from "../utils";

/**
 * Move towards the nearest weapon in the grid
 */
export class FindWeaponState implements AlgorithmState {
  private playerId: string;

  constructor(playerId: string) {
    this.playerId = playerId;
  }

  chooseNextAction(state: GameState): PlayerAction | null {
    // Find the closest weapon
    let shortestPath: null | { x: number; y: number }[] = null;
    const player = state.players[this.playerId];
    const graph = new Graph(buildAStarWeights(state.playfield));
    for (const weapon of state.weapons) {
      const path = astar.search(
        graph,
        graph.grid[player.row - 1][player.col - 1],
        graph.grid[weapon.row - 1][weapon.col - 1]
      );

      if (path.length === 0) {
        continue; // Ignore if standing on the weapon
      }

      if (shortestPath === null || shortestPath.length > path.length) {
        shortestPath = path;
      }
    }

    if (shortestPath === null) {
      // No weapons left, randomly move or attack
      return randomMoveOrAttack();
    }

    // Figure out the direction to move
    return getAStarMove(player, shortestPath);
  }
}
