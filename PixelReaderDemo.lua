-- Notes: max table size is 300
local BOX_WIDTH = 1
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
    d = {
        a="a",b="2",c="3",d="4",e="5",f="6",g="7",h="8",i="9",
        a1="a",b1="2",c1="3",d1="4",e1="5",f1="6",g1="7",h1="8",i1="9",
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
    -- print("num boxes "..tostring(#boxes))
    -- print('boxes["active_boxes"]: '..tostring(boxes["active_boxes"]))
    -- print('boxes["max_boxes"]: '..tostring(boxes["max_boxes"]))
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
-- header
-- num_boxes:8; 24-8
-- width
function OnUpdate(self, elapsed)
    local t = cbor.encode(d)
    local checksum = 0
    -- data.st = data.st.."a"
    -- pad serialized message to multiple of 3 bytes to align with the three rgb channels in a pixel
    while (Modulo(#t, 3) ~= 0) do
        t = t .. "\0"
    end
    -- cbor isn't ordered so keep header as first pixel
    -- print(#t)
    for i = 1, #t, 3 do
        local r = string.byte(t, i)
        checksum = Modulo(checksum+r, 256)
        local g = string.byte(t, i+1)
        checksum = Modulo(checksum+g, 256)
        local b = string.byte(t, i+2)
        checksum = Modulo(checksum+b, 256)
        box_index = math.floor(i/3)+1
        box_index = box_index + 1 -- added for header
        -- if i == 1 then
        --     print(tostring(r).." ".. tostring(g).." ".. tostring(b))
        -- end
        set_texture_from_arr(boxes[box_index], rbgToColor(r,g,b))
    end
    -- print("checksum "..tostring(checksum))
    -- print("clock "..tostring(clock))
    header = bitshift_left(#t, 16) + bitshift_left(checksum, 8) + clock
    set_texture_from_arr(boxes[1], integerToColor(header))
    show_boxes(1+(#t/3))
    if clock == 0 then
        for i = 1, #t do
            local byte = string.byte(t, i)
            local hex_char = string.format("%x", string.byte(t, i))
            print(tostring(i)..": 0x"..hex_char)
        end
        print("\n")
    end
    -- print(header)
    clock = Modulo(clock+1, 256)
end

init()
