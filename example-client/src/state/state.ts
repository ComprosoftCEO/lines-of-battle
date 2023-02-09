import { GameState, PlayerAction } from "../protocol";

export interface AlgorithmState {
  chooseNextAction(state: GameState): PlayerAction | null;
}
