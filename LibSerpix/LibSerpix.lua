local BOX_WIDTH = 1
local BOX_HEIGHT = 6
-- pixel boxes that numbers are stored in
-- stores rbg valuess that hold 0xFFFFFF each
local boxes = {}
local addons = {}
local _, ADDONSELF = ...
local cbor = get_cbor()

LibSerpix = {}
function init()
    LibSerpix.addons = {}
    serializer = {}
    serializer.vals = {}
    serializer.vals.p = {}
    LibSerpix.serializer = serializer

    create_boxes()
end

function LibSerpix.register_addon(addon_name)
    addons[#addons+1] = addon_name
end

function LibSerpix.unregister_addon(addon_name)
    for i = #addons, 1, -1 do
        if addons[i] == addon_name then
            table.remove(addons, i)
        end
    end
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

local clock = 0
function OnUpdate(self, elapsed)
    serializer.vals.p.clock = clock
    local t = cbor.encode(serializer.vals)
    local checksum = 0
    serializer.vals.u = {}
    for i = 1, #addons do
        serializer.vals.u[addons[i]] = {}
    end
    -- pad serialized message to multiple of 3 bytes to align with the three rgb channels in a pixel
    encode_size = #t
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
    header = checksum + bitshift_left(encode_size, 8)
    set_texture_from_arr(boxes[1], integerToColor(header))
    show_boxes(1+(#t/3))
    clock = Modulo(clock+1, 64)
end

init()
