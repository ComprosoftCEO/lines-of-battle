-- Load game modules
local libPlayfield = require("playfield")
local libWeapon = require("weapon")
Inspect = require("inspect") -- For debugging

-- Forward declare internal functions
local movePlayer, attack, tryHitPlayer, dropWeapon

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
  pf, pfRows, pfCols = libPlayfield.newPlayfield(10, 10)

  -- Insert players into random positions in the playfield
  playerDetails = {}
  for _, player in ipairs(players) do
    local row, col
    repeat
      row = math.random(1, pfRows)
      col = math.random(1, pfCols)
    until (pf[row][col] == libPlayfield.BLANK)

    playerDetails[player] = { row = row, col = col, health = 3 }
    pf[row][col] = -1 -- Temporarily
  end

  -- Now generate the weapons (3 per player)
  weapons = {}
  for _ = 1, (3 * #players) do
    local row, col
    repeat
      row = math.random(1, pfRows)
      col = math.random(1, pfCols)
    until (pf[row][col] == libPlayfield.BLANK)

    table.insert(weapons, libWeapon.NewLaserGun(row, col))
    pf[row][col] = -1 -- Temporarily
  end

  -- Reset all negative playfield spaces to 0
  for row = 1, #pf do
    for col = 1, #pf do
      if pf[row][col] == -1 then
        pf[row][col] = libPlayfield.BLANK
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
  for _, playerID in ipairs(playerOrder) do
    -- Make sure player specified an action and is still alive
    local action = actions[playerID]
    if (action == nil) or (playerDetails[playerID] == nil) then goto continue end

    if action.type == "move" then
      movePlayer(playerID, action.direction)

    elseif action.type == "attack" then
      attack(ctx, playerID, action.direction)

    elseif action.type == "dropWeapon" then
      dropWeapon(playerID)

    end

    ::continue::
  end

  return memoizeGameState()
end

-- Convert a direction string ("up", "down", "left", "right") to (Δrow, Δcol)
local function directionToDeltas(direction)
  local deltaRow, deltaCol = 0, 0
  if direction == "up" then
    deltaRow = -1
  elseif direction == "down" then
    deltaRow = 1
  elseif direction == "left" then
    deltaCol = -1
  elseif direction == "right" then
    deltaCol = 1
  end

  return deltaRow, deltaCol
end

-- Test if a given (row,col) is outside the playfield
--  Uses pfRows and pfCols to detect the height and width of the playfield
local function isOutsidePlayfield(row, col)
  return (row < 1) or (row > pfRows) or (col < 1) or (col > pfCols)
end

-- Handle player movement
function movePlayer(playerID, direction)
  local player = playerDetails[playerID]

  local deltaRow, deltaCol = directionToDeltas(direction)
  local newRow = player.row + deltaRow
  local newCol = player.col + deltaCol

  -- 1. Make sure player is still inside the playfield
  if isOutsidePlayfield(newRow, newCol) then
    return
  end

  -- 2. Make sure there isn't a wall in the way
  if pf[newRow][newCol] ~= libPlayfield.BLANK then
    return
  end

  -- 3. Make sure another player isn't in the way
  for otherPlayerID, otherPlayer in pairs(playerDetails) do
    if (otherPlayerID ~= playerID) and (otherPlayer.row == newRow) and (otherPlayer.col == newCol) then
      return
    end
  end

  -- Move player to new position
  player.row = newRow
  player.col = newCol

  -- Handle weapon pickups
  for i, weapon in ipairs(weapons) do
    if (weapon.row == newRow) and (weapon.col == newCol) then
      if (player.weapon == nil) then -- Pickup the weapon
        weapon.row = nil
        weapon.col = nil
        player.weapon = weapon
        table.remove(weapons, i)

      else -- Switch the weapons
        player.weapon.row = newRow
        player.weapon.col = newCol
        table.insert(weapons, player.weapon)

        weapon.row = nil
        weapon.col = nil
        player.weapon = weapon
        table.remove(weapons, i)
      end
      break
    end
  end
end

-- Handle a player attack
function attack(ctx, playerID, direction)
  local player = playerDetails[playerID]
  local weapon = player.weapon

  -- Remove ammo first (always costs ammo to shoot)
  --   ammo of "nil" indiates infinite ammo
  if player.weapon ~= nil and player.weapon.ammo ~= nil then
    player.weapon.ammo = player.weapon.ammo - 1
    if player.weapon.ammo <= 0 then
      player.weapon = nil -- Weapon is empty
    end
  end

  -- Find direction to attack
  local deltaRow, deltaCol = directionToDeltas(direction)
  local attackRow = player.row + deltaRow
  local attackCol = player.col + deltaCol

  -- Move the attack in a straight direction
  while (true) do
    -- 1. Make sure player is still inside the playfield
    if isOutsidePlayfield(attackRow, attackCol) then
      return
    end

    -- 2. Make sure there isn't a wall in the way
    if pf[attackRow][attackCol] ~= libPlayfield.BLANK then
      return
    end

    -- 3. Try to hit a player, or return on failure
    if tryHitPlayer(ctx, attackRow, attackCol, weapon) then
      return
    end

    -- 4. Attack only immediate space if player doesn't have a weapon
    if weapon == nil then
      return
    end

    attackRow = attackRow + deltaRow
    attackCol = attackCol + deltaCol
  end
end

-- Try to hit a player at a given (row, col)
--   Returns true if player was hit, or false otherwise
function tryHitPlayer(ctx, row, col, weapon)
  -- See if there is a player at the location
  for playerID, player in pairs(playerDetails) do
    if (player.row == row) and (player.col == col) then
      local damage = 1
      if weapon ~= nil then
        damage = weapon.damage
      end

      -- Attack the player
      player.health = math.max(player.health - damage, 0)
      if player.health <= 0 then
        ctx:notifyPlayerKilled(playerID)
        playerDetails[playerID] = nil
      end

      return true
    end
  end

  return false
end

-- Handle dropping a weapon
function dropWeapon(playerID)
  local player = playerDetails[playerID]

  -- See if there is a weapon to switch
  for i, weapon in ipairs(weapons) do
    if (weapon.row == player.row) and (weapon.col == player.col) then
      if (player.weapon == nil) then
        -- Calling "drop" with no weapon picks up any weapon on the space
        weapon.row = nil
        weapon.col = nil
        player.weapon = weapon
        table.remove(weapons, i)

      else -- Switch the weapons
        player.weapon.row = player.row
        player.weapon.col = player.col
        table.insert(weapons, player.weapon)

        weapon.row = nil
        weapon.col = nil
        player.weapon = weapon
        table.remove(weapons, i)
      end
      return
    end
  end

  -- No weapon on space, so drop it
  if player.weapon ~= nil then
    player.weapon.row = player.row
    player.weapon.col = player.col
    table.insert(weapons, player.weapon)
    player.weapon = nil
  end
end
