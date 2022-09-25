-- Load game modules
local playfield = require("playfield")
local inspect = require("inspect")

-- Forward declare internal functions
local movePlayer

-- Local game variables
local playerOrder = {}
local playerDetails = {}
local pf = {}
local pfRows, pfCols -- Helper variables
local weapons = {}

-- Generates an object representing the global game state
local function memoizeGameState()
  return {
    playfield = pf,
    players = playerDetails,
    weapons = weapons,
  }
end

-- ====================================================
--  Initialization routine - Receives array of players
-- ====================================================
function Init(ctx, players)
  playerOrder = players
  pf, pfRows, pfCols = playfield.newPlayfield(10, 10)

  -- Insert players into random positions in the playfield
  for _, player in ipairs(players) do
    local row, col
    repeat
      row = math.random(1, pfRows)
      col = math.random(1, pfCols)
    until (pf[row][col] == playfield.BLANK)

    playerDetails[player] = { row = row, col = col, health = 3 }
    pf[row][col] = -1 -- Temporarily
  end

  -- Now generate the weapons
  weapons = {}
  for _ = 1, #players do
    local row, col
    repeat
      row = math.random(1, pfRows)
      col = math.random(1, pfCols)
    until (pf[row][col] == playfield.BLANK)

    table.insert(weapons, {
      type = "laserGun",
      row = row,
      col = col,
      ammo = 1,
    })
    pf[row][col] = -1 -- Temporarily
  end

  -- Reset all negative playfield spaces to 0
  for row = 1, #pf do
    for col = 1, #pf do
      if pf[row][col] == -1 then
        pf[row][col] = playfield.BLANK
      end
    end
  end

  return memoizeGameState()
end

-- ====================================================
--   Update routine - Receives map of player actions
-- ====================================================
function Update(ctx, actions)

  -- Perform the player actions in order
  --  Rust guarantees these follow the right input format
  for _, player in ipairs(playerOrder) do
    -- Make sure player specified an action
    local action = actions[player]
    if action == nil then goto continue end

    if action.type == "move" then
      movePlayer(player, action.direction)

    elseif action.type == "shoot" then
      -- TODO: Handle a shoot
    end

    ::continue::
  end

  return memoizeGameState()
end

-- Handle player movement
function movePlayer(player, direction)
  local deltaCol, deltaRow = 0, 0
  if direction == "up" then
    deltaRow = -1
  elseif direction == "down" then
    deltaRow = 1
  elseif direction == "left" then
    deltaCol = -1
  elseif direction == "right" then
    deltaCol = 1
  else
    return -- Unknown direction
  end

  local newRow = playerDetails[player].row + deltaRow
  local newCol = playerDetails[player].col + deltaCol

  -- 1. Make sure player is still inside the playfield
  if (newRow < 1) or (newRow > pfRows) or (newCol < 1) or (newCol > pfCols) then
    return
  end

  -- 2. Make sure there isn't a wall in the way
  if pf[newRow][newCol] ~= playfield.BLANK then
    return
  end

  -- 3. Make sure another player isn't in the way
  for otherPlayer, details in pairs(playerDetails) do
    if (otherPlayer ~= player) and (details.row == newRow) and (details.col == newCol) then
      return
    end
  end

  -- Move player to new position
  playerDetails[player].row = newRow
  playerDetails[player].col = newCol

  -- Handle weapon pickups
  for i, weapon in ipairs(weapons) do
    if (weapon.row == newRow) and (weapon.col == newCol) then
      if (player.weapon == nil) then -- Pickup the weapon
        playerDetails[player].weapoon = weapon
        table.remove(weapons, i)
      else -- Switch the weapons
        table.insert(weapons, player.weapon)
        playerDetails[player].weapon = weapon
        table.remove(weapons, i)
      end
      break
    end
  end
end
