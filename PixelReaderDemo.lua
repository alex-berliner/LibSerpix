-- Notes: max table size is 300
local BOX_WIDTH = 3
local BOX_HEIGHT = 3
-- max pixel boxes, most will usually be turned off.
-- no cost to increasing this besides increased screen real estate
local NUM_BOXES = 100
-- pixel boxes that numbers are stored in
-- stores rbg valuess that hold 0xFFFFFF each
-- can sometimes (~once per 1000 frames) be inaccurate, so should be accounted for
local boxes = {}
local _, ADDONSELF = ...

local cbor = get_cbor()

function init()
    print("init")
    init_my_serialized_data()
    create_boxes()
    UIParent:SetScript("OnUpdate", OnUpdate)
end

function init_my_serialized_data()
    h = {sz = 0, cs=0, px=BOX_WIDTH}
    b = {a=1}
    d = {h=h, b=b}
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
        e = create_box(BOX_WIDTH, BOX_HEIGHT, (i-1)*BOX_WIDTH, 0)
        e.Texture:SetColorTexture(1,1,1)
        boxes[i] = e
    end
    boxes["active_boxes"] = NUM_BOXES
    boxes["max_boxes"] = NUM_BOXES
    print("num boxes "..tostring(#boxes))
    print('boxes["active_boxes"]: '..tostring(boxes["active_boxes"]))
    print('boxes["max_boxes"]: '..tostring(boxes["max_boxes"]))
end

function show_boxes(n)
    for i = 1, n do
        -- print("show "..tostring(i))
        boxes[i]:Show()
    end
    for i = n+1, NUM_BOXES do
        -- print("hide "..tostring(i))
        boxes[i]:Hide()
    end
    boxes["active_boxes"] = n
end

local clock = 0

function OnUpdate(self, elapsed)
    local t = cbor.encode(d)
    -- data.st = data.st.."a"
    -- pad serialized message to multiple of 3 bytes to align with the three rgb channels in a pixel
    while (Modulo(#t, 3) ~= 0) do
        t = t .. "\0"
    end
    -- cbor isn't ordered so keep header as first pixel
    header = bitshift_left(#t, 16) + bitshift_left(BOX_WIDTH, 8) + clock
    -- print(header)
    set_texture_from_arr(boxes[1], integerToColor(header))
    print(#t)
    for i = 1, #t, 3 do
        local r = string.byte(t, i)
        local g = string.byte(t, i+1)
        local b = string.byte(t, i+2)
        box_index = math.floor(i/3)+1
        box_index = box_index + 1 -- added for header
        set_texture_from_arr(boxes[box_index], rbgToColor(r,g,b))
    end
    show_boxes(1+(#t/3))
    clock = Modulo(clock+1, 256)
end

init()
