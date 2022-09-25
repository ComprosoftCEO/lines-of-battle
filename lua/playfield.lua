--
-- Generate the obstacles in the playfield
--
local Module = {}

-- Define constants
Module.BLANK = 0
Module.WALL = 1

local DEFAULT_PLAYFIELD = [[
x........x
..x....x..
.xx.xx.xx.
..........
..x.xx.x..
..x.xx.x..
..........
.xx.xx.xx.
..x....x..
x........x
]]

local PLAYFIELD_LOOKUP = {
  ["."] = Module.BLANK,
  ["x"] = Module.WALL,
}

local REVERSE_LOOKUP = {
  [Module.BLANK] = ".",
  [Module.WALL] = "x",
}

-- Convert a string playfield into a list of numbers
local function convertStringToPlayfield(playfield)
  local pf = {}
  local row = 1
  for line in playfield:gmatch("[^\r\n]+") do
    pf[row] = {}

    local col = 1
    for c in line:gmatch(".") do
      pf[row][col] = PLAYFIELD_LOOKUP[c] or Module.BLANK
      col = col + 1
    end

    row = row + 1
  end

  return pf
end

-- Insert a template into the middle of the playfield
--  Modifies the "playfield" input, but NOT the template
local function fillTemplate(playfield, template)
  local pfRowStart = math.max(math.floor((#playfield - #template) / 2), 0)
  local tmplRowStart = math.max(math.floor((#template - #playfield) / 2), 0)
  for row = 1, math.min(#playfield, #template) do
    local pfColStart = math.max(math.floor((#playfield[row] - #template[row]) / 2), 0)
    local tmplColStart = math.max(math.floor((#template[row] - #playfield[row]) / 2), 0)
    for col = 1, math.min(#playfield[row], #template[row]) do
      playfield[pfRowStart + row][pfColStart + col] = template[tmplRowStart + row][tmplColStart + col]
    end
  end
end

-- Create a new playfield of the given size
function Module.newPlayfield(rows, cols)
  -- Create a new playfield
  local pf = {}
  for row = 1, rows do
    pf[row] = {}
    for col = 1, cols do
      pf[row][col] = Module.BLANK
    end
  end

  -- Center the default template in the grid
  fillTemplate(pf, convertStringToPlayfield(DEFAULT_PLAYFIELD))

  return pf, rows, cols
end

-- Print out a playfield array to standard output as a string
function Module.playfieldToString(playfield)
  local output = ""
  for row = 1, #playfield do
    for col = 1, #playfield[row] do
      output = output .. (REVERSE_LOOKUP[playfield[row][col]] or "?")
    end

    if row < #playfield then -- Last line does not get a newline
      output = output .. "\n"
    end
  end

  return output
end

return Module -- Return the module
