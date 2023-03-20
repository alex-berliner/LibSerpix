local BOX_WIDTH = 1
local BOX_HEIGHT = 6
-- max pixel boxes, most will usually be turned off.
-- no cost to increasing this besides increased screen real estate
local NUM_BOXES = 512
-- pixel boxes that numbers are stored in
-- stores rbg valuess that hold 0xFFFFFF each
-- can sometimes (~once per 1000 frames) be inaccurate, so should be accounted for
local boxes = {}
local _, ADDONSELF = ...

local cbor = get_cbor()
local serializer = get_serializer()

function init()
    create_boxes()
    UIParent:SetScript("OnUpdate", OnUpdate)
    f = CreateFrame("Frame")
    -- Register the OnCombatLogEvent function to the COMBAT_LOG_EVENT_UNFILTERED event
    f:RegisterEvent("COMBAT_LOG_EVENT_UNFILTERED")
    f:SetScript("OnEvent", OnCombatLogEvent)
end

-- Define the OnCombatLogEvent function
function OnCombatLogEvent(event, ...)
    serializer.CombatEventHandler(event, ...)
end

function create_boxes()
    local function create_box(w,h,x,y)
        p = CreateFrame("Frame", nil, UIParent)
        p:SetWidth(w)
        p:SetHeight(h)
        p:SetPoint("TOPLEFT", x, -y)
        p.Texture = p:CreateTexture()
        p.Texture:SetColorTexture(0,1,0)
        p.Texture:SetAllPoints()
        p:Show()
        return p
    end
    for i = 1, NUM_BOXES do
        local space_bw_boxes = 1
        local x = (i-1) * (BOX_WIDTH + space_bw_boxes)
        e = create_box(BOX_WIDTH, BOX_HEIGHT, x, 0)
        e.Texture:SetColorTexture(1,1,1)
        boxes[i] = e
    end
    boxes["active_boxes"] = NUM_BOXES
    boxes["max_boxes"] = NUM_BOXES
end

function show_boxes(n)
    for i = 1, n do
        boxes[i]:Show()
    end
    for i = n+1, NUM_BOXES do
        boxes[i]:Hide()
    end
    boxes["active_boxes"] = n
end
function count_bits(num)
    local count = 0
    while num > 0 do
      count = count + 1
      num = math.floor(num / 2)
    end
    return count
  end

local clock = 0
function OnUpdate(self, elapsed)
    local t = cbor.encode(serializer.vals)
    local checksum = 0
    -- serializer.vals = {}
    -- pad serialized message to multiple of 3 bytes to align with the three rgb channels in a pixel
    encode_size = #t
    while (Modulo(#t, 3) ~= 0) do
        t = t .. "\0"
    end
    -- pack string bytes into pixels
    -- cbor table isn't ordered so keep header as first pixel
    -- print(hex_dump(t, #t))
    for i = 1, #t, 3 do
        local r = string.byte(t, i)
        checksum = Modulo(checksum+r, 128)
        local g = string.byte(t, i+1)
        checksum = Modulo(checksum+g, 128)
        local b = string.byte(t, i+2)
        checksum = Modulo(checksum+b, 128)
        box_index = math.floor(i/3)+1
        box_index = box_index + 1 -- added for header
        set_texture_from_arr(boxes[box_index], rbgToColor(r,g,b))
    end
    -- encode_size : 10 bits
    -- checksum: 8 bits
    -- clock : 6 bits
    -- print("getBytesRemaining", getBytesRemaining(serializer))
    header = bitshift_left(encode_size, 14) + bitshift_left(checksum, 6) + clock
    -- print(string.format("0x%02x", checksum))
    print(hex_dump(t, #t))
    -- print(header)
    set_texture_from_arr(boxes[1], integerToColor(header))
    show_boxes(1+(#t/3))
    clock = Modulo(clock+1, 64)
end

init()
