local MAX_PIXEL_SIZE = 256 * 256 * 256 - 1
local FLOAT_MAX = 9.99999

-- max pixel boxes, most will usually be turned off.
-- no cost to increasing this besides increased screen real estate
local BYTES_PER_BOX = 3
local HEADER_BYTES = 3
local PAYLOAD_BYTES = 1533
LIBSPX_NUM_BOXES = (HEADER_BYTES + PAYLOAD_BYTES)/BYTES_PER_BOX

-- https://gist.github.com/Elemecca/6361899
function hex_dump(str, len)
    local dump = ""
    local hex = ""
    local asc = ""

    for i = 1, len do
        if 1 == i % 8 then
            dump = dump .. hex .. asc .. "\n"
            hex = string.format( "%04x: ", i - 1 )
            asc = ""
        end

        local ord = string.byte( str, i )
        hex = hex .. string.format( "%02x ", ord )
        if ord >= 32 and ord <= 126 then
            asc = asc .. string.char( ord )
        else
            asc = asc .. "."
        end
    end

    return dump .. hex
            .. string.rep( "   ", 8 - len % 8 ) .. asc
end

function bitshift_left(n, k)
    local result = n
    for i = 1, k do
      result = result + result
    end
    return result
  end

function set_texture_from_arr(t, c)
    t.Texture:SetColorTexture(c[1], c[2], c[3])
end

function fixedDecimalToColor(f)
    if f > FLOAT_MAX then
        -- print("Number too big to be passed as a fixed-point decimal")
        return {0}
    elseif f < 0 then
        return {0}
    end
    -- "%f" denotes formatting a string as floating point decimal
    -- The number (.5 in this case) is used to denote the number of decimal places
    local f6 = tonumber(string.format("%.5f", 1))
    -- Makes number an integer so it can be encoded
    local i = math.floor(f * 100000)
    return integerToColor(i)
end

function Modulo(val, by)
    return val - math.floor(val / by) * by
end

function integerToColor(i)
    if i ~= math.floor(i) then
        print("The number passed to 'integerToColor' must be an integer")
    end

    if i > (256 * 256 * 256 - 1) then -- the biggest value to represent with 3 bytes of colour
        print("Integer too big to encode as color", string.format("%08x", i))
    end

    -- r,g,b are integers in range 0-255
    local b = Modulo(i, 256)
    i = math.floor(i / 256)
    local g = Modulo(i, 256)
    i = math.floor(i / 256)
    local r = Modulo(i, 256)

    -- then we turn them into 0-1 range
    return {r / 255, g / 255, b / 255}
end

function rbgToColor(r,g,b)
    return {r / 255, g / 255, b / 255}
end

local cbor = get_cbor()
function getBytesRemaining(serializer)
    local t = cbor.encode(serializer.vals)
    return PAYLOAD_BYTES-#t
end
