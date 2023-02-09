import {
  AttackAction,
  Direction,
  GameState,
  MoveAction,
  PlayfieldObject,
  Position,
  RequestType,
} from "./protocol";

/**
 * Pick a random element from a non-empty list
 */
export function pickRandom<T>(input: T[]): T {
  if (input.length === 0) {
    throw new Error("Cannot pick from an empty list!");
  }
  return input[Math.floor(Math.random() * input.length)];
}

export const randomDirection = (): Direction =>
  pickRandom([Direction.Up, Direction.Down, Direction.Left, Direction.Right]);

/**
 * Convert the playfield into something that can be used by the A* implementation
 */
export function buildAStarWeights(pf: number[][]): number[][] {
  const playfield: number[][] = [];
  for (let row = 0; row < pf.length; row += 1) {
    playfield.push([]);
    for (let col = 0; col < pf[row].length; col += 1) {
      playfield[row].push(1);
      if (pf[row][col] === PlayfieldObject.Wall) {
        playfield[row][col] = 0;
      }
    }
  }

  return playfield;
}

/**
 * Pick a random move or attack action
 */
export const randomMoveOrAttack = (): AttackAction | MoveAction => ({
  type: pickRandom([RequestType.Move, RequestType.Attack]),
  direction: randomDirection(),
});

/**
 * Convert the A* response to a movement direction in the grid
 */
export function getAStarMove(
  player: Position<unknown>,
  shortestPath: { x: number; y: number }[]
): MoveAction | null {
  // Note, the A* implementation uses (x,y) instead of (row, col), but it works the same
  const newRow = shortestPath[0].x + 1;
  const newCol = shortestPath[0].y + 1;
  if (newRow === player.row + 1 && newCol === player.col) {
    return { type: RequestType.Move, direction: Direction.Down };
  } else if (newRow === player.row - 1 && newCol === player.col) {
    return { type: RequestType.Move, direction: Direction.Up };
  } else if (newRow === player.row && newCol === player.col + 1) {
    return { type: RequestType.Move, direction: Direction.Right };
  } else if (newRow === player.row && newCol === player.col - 1) {
    return { type: RequestType.Move, direction: Direction.Left };
  } else {
    return null;
  }
}

/**
 * Test if the given row is blocked between the two columsn
 * All rows and columns should be indexed starting at 0
 */
export function isRowBlocked(
  state: GameState,
  row: number,
  p1Col: number,
  p2Col: number
): boolean {
  const minCol = Math.min(p1Col, p2Col);
  const maxCol = Math.max(p1Col, p2Col);

  // Test for walls
  for (let col = minCol; col < maxCol; col += 1) {
    if (state.playfield[row][col] === PlayfieldObject.Wall) {
      return true;
    }
  }

  // Test for other players
  for (const player of Object.values(state.players)) {
    if (
      player.row - 1 === row &&
      player.col - 1 > minCol &&
      player.col - 1 < maxCol
    ) {
      return true;
    }
  }

  return false;
}

/**
 * Test if the given column is blocked between the two rows
 * All rows and columns should be indexed starting at 0
 */
export function isColBlocked(
  state: GameState,
  col: number,
  p1Row: number,
  p2Row: number
): boolean {
  const minRow = Math.min(p1Row, p2Row);
  const maxRow = Math.max(p1Row, p2Row);

  // Test for walls
  for (let row = minRow; row < maxRow; row += 1) {
    if (state.playfield[row][col] === PlayfieldObject.Wall) {
      return true;
    }
  }

  // Test for other players
  for (const player of Object.values(state.players)) {
    if (
      player.col - 1 === col &&
      player.row - 1 > minRow &&
      player.row - 1 < maxRow
    ) {
      return true;
    }
  }

  return false;
}
