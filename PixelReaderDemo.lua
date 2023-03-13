-- Notes: max table size is 300
local BOX_WIDTH = 1
local BOX_HEIGHT = 6
-- max pixel boxes, most will usually be turned off.
-- no cost to increasing this besides increased screen real estate
-- 100 is the max because lua strings appear not to be able to hold more than 300 bytes
local NUM_BOXES = 100
-- pixel boxes that numbers are stored in
-- stores rbg valuess that hold 0xFFFFFF each
-- can sometimes (~once per 1000 frames) be inaccurate, so should be accounted for
local boxes = {}
local _, ADDONSELF = ...

local cbor = get_cbor()
tx_healing = 0
tx_overhealing = 0
tx_damage = 0

function init()
    init_my_serialized_data()
    create_boxes()
    UIParent:SetScript("OnUpdate", OnUpdate)
    f = CreateFrame("Frame")
    -- Register the OnCombatLogEvent function to the COMBAT_LOG_EVENT_UNFILTERED event
    f:RegisterEvent("COMBAT_LOG_EVENT_UNFILTERED")
    f:SetScript("OnEvent", OnCombatLogEvent)
end

-- Define the OnCombatLogEvent function
function OnCombatLogEvent(event, ...)
    function parse_heal(...)
        local timestamp, subevent, _, sourceGUID, sourceName, sourceFlags, sourceRaidFlags, destGUID, destName, destFlags, destRaidFlags = ...
        local spellId, spellName, spellSchool = select(12, ...)
        -- Check if the sourceGUID is your character's GUID
        if sourceGUID == UnitGUID("player") then
            healing, overhealing, absorbed, critical = select(15, ...)
            tx_overhealing = tx_overhealing + overhealing
            tx_healing = tx_healing + healing - overhealing
        end
    end
    function parse_dmg(...)
        local timestamp, subevent, _, sourceGUID, sourceName, sourceFlags, sourceRaidFlags, destGUID, destName, destFlags, destRaidFlags = ...
        local spellId, spellName, spellSchool = select(12, ...)
        if sourceGUID == UnitGUID("player") then
            amount, overkill, school, resisted, blocked, absorbed, critical, glancing, crushing, isOffHand = select(15, ...)
            -- print("amount: " .. tostring(amount))
            tx_damage = tx_damage + amount
        end
    end
    function parse_event(...)
        local timestamp, subevent, _, sourceGUID, sourceName, sourceFlags, sourceRaidFlags, destGUID, destName, destFlags, destRaidFlags = ...
        if subevent == "SPELL_HEAL" or subevent == "SPELL_PERIODIC_HEAL" then
            parse_heal(...)
        end
        if subevent == "SPELL_DAMAGE" or subevent == "SPELL_PERIODIC_DAMAGE" then
            parse_dmg(...)
        end
    end
    parse_event(CombatLogGetCurrentEventInfo())
end

function init_my_serialized_data()
    d = {
        empty=0,
    }
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

-- https://gist.github.com/Elemecca/6361899
function hex_dump (str, len)
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

local clock = 0
function OnUpdate(self, elapsed)
    d = {
        bonders=tx_healing,
        overhealing=tx_overhealing,
        damage=tx_damage,
    }
    local t = cbor.encode(d)
    local checksum = 0
    -- if tx_healing > 0 then
    --     print(tx_healing)
    -- end
    tx_healing = 0
    tx_overhealing = 0
    tx_damage = 0
    -- pad serialized message to multiple of 3 bytes to align with the three rgb channels in a pixel
    orig_size = #t
    while (Modulo(#t, 3) ~= 0) do
        t = t .. "\0"
    end
    -- pack string bytes into pixels
    -- cbor table isn't ordered so keep header as first pixel
    for i = 1, #t, 3 do
        local r = string.byte(t, i)
        checksum = Modulo(checksum+r, 256)
        local g = string.byte(t, i+1)
        checksum = Modulo(checksum+g, 256)
        local b = string.byte(t, i+2)
        checksum = Modulo(checksum+b, 256)
        box_index = math.floor(i/3)+1
        box_index = box_index + 1 -- added for header
        set_texture_from_arr(boxes[box_index], rbgToColor(r,g,b))
    end
    header = bitshift_left(orig_size, 16) + bitshift_left(checksum, 8) + clock
    set_texture_from_arr(boxes[1], integerToColor(header))
    show_boxes(1+(#t/3))
    clock = Modulo(clock+1, 180)
end

init()
