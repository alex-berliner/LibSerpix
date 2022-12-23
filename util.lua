MAX_PIXEL_SIZE = 256 * 256 * 256 - 1
FLOAT_MAX = 9.99999
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
        print("Integer too big to encode as color")
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
