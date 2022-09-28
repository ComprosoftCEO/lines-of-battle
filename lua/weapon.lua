--
-- Generate the weapons
--
local Module = {}

-- Make a new laser gun weapon
function Module.NewLaserGun(row, col)
  return {
    type = "laserGun",
    row = row,
    col = col,
    ammo = 1,
    damage = 2,
  }
end

return Module
