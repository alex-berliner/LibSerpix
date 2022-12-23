local BOX_WIDTH = 10
local BOX_HEIGHT = 10
-- max pixel boxes, most will usually be turned off.
-- no cost to increasing this besides increased screen real estate
local NUM_BOXES = 100
-- pixel boxes that numbers are stored in
-- stores rbg valuess that hold 0xFFFFFF each
-- can sometimes (~once per 1000 frames) be inaccurate, so should be accounted for
local boxes = {}
local _, ADDONSELF = ...

function init()
    init_my_protobuf()
    create_boxes()
end

function init_my_protobuf()
    local luapb = ADDONSELF.luapb
    local person = luapb.load_proto_ast(ADDONSELF.pbperson).Person

    msg0 = person()

    msg0.name = "hello my name is frank i am 21 years old"
    msg0.id = 1
    msg0.ctr1 = 0
    msg0.ctr2 = 1
    msg0.ctr3 = 2

    print("serialize: name " .. msg0.name .. " id " .. msg0.id)

    local t = msg0:Serialize()

    assert(#t > 0, "size of t > 0")

    local msg1 = person()
    msg1:Parse(t)

    assert(msg1.name == msg0.name, "name not equal")
    assert(msg1.id == msg0.id, "id not equal")

    print("deserialize: name " .. msg1.name .. " id " .. msg1.id)

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
        print("show "..tostring(i))
        boxes[i]:Show()
    end
    for i = n+1, NUM_BOXES do
        -- print("hide "..tostring(i))
        boxes[i]:Hide()
    end
    boxes["active_boxes"] = n
end

function OnUpdate(self, elapsed)
    local t = msg0:Serialize()
    -- pad serialized message to multiple of 3 bytes to align with the three rgb channels in a pixel
    while (Modulo(#t, 3) ~= 0) do
        t = t .. "\0"
    end
    for i = 1, #t, 3 do
        local r = string.byte(t, i)
        local g = string.byte(t, i+1)
        local b = string.byte(t, i+2)
        box_index = math.floor(i/3)+1
        set_texture_from_arr(boxes[box_index], rbgToColor(r,g,b))
    end
    show_boxes(#t/3)
    msg0.name = msg0.name .. string.char(string.byte('a') + Modulo(msg0.ctr1, 26))
    msg0.ctr1=Modulo(msg0.ctr1+1, 100)
    msg0.ctr2=Modulo(msg0.ctr2+1, 100)
    msg0.ctr3=Modulo(msg0.ctr3+1, 100)
    print(#t)
end

UIParent:SetScript("OnUpdate", OnUpdate)
init()
