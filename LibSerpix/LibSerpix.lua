local BOX_WIDTH = 1
local BOX_HEIGHT = 6
-- pixel boxes that numbers are stored in
-- stores rbg valuess that hold 0xFFFFFF each
local boxes = {}
local firstbox = {}
local lastbox = {}
local addons = {}
local _, ADDONSELF = ...
local cbor = get_cbor()

LibSerpix = {}
function init()
    LibSerpix.data_queue = {}
    serializer = {}
    serializer.vals = {}
    serializer.vals.p = {}
    LibSerpix.serializer = serializer

    create_boxes()
end

function LibSerpix.add_data(namespace, data_table)
    LibSerpix.data_queue[#LibSerpix.data_queue + 1] = {namespace, data_table}
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
    for i = 1, LIBSPX_NUM_BOXES do
        local space_bw_boxes = 1
        local x = (i-1) * (BOX_WIDTH + space_bw_boxes)
        e = create_box(BOX_WIDTH, BOX_HEIGHT, x, 1)
        e.Texture:SetColorTexture(1,1,1)
        boxes[i] = e
    end
    boxes["active_boxes"] = LIBSPX_NUM_BOXES
    boxes["max_boxes"] = LIBSPX_NUM_BOXES

    for i = 1, LIBSPX_NUM_BOXES do
        local space_bw_boxes = 1
        local x = (i-1) * (BOX_WIDTH + space_bw_boxes)
        e = create_box(BOX_WIDTH, 1, x, 0)
        e.Texture:SetColorTexture(1,1,1)
        firstbox[i] = e
        set_texture_from_arr(firstbox[i], rbgToColor(42,0,69))
    end
    firstbox["active_boxes"] = LIBSPX_NUM_BOXES
    firstbox["max_boxes"] = LIBSPX_NUM_BOXES

    for i = 1, LIBSPX_NUM_BOXES do
        local space_bw_boxes = 1
        local x = (i-1) * (BOX_WIDTH + space_bw_boxes)
        e = create_box(BOX_WIDTH, 1, x, BOX_HEIGHT+1)
        e.Texture:SetColorTexture(1,1,1)
        lastbox[i] = e
        set_texture_from_arr(lastbox[i], rbgToColor(42,0,69))
    end
    lastbox["active_boxes"] = LIBSPX_NUM_BOXES
    lastbox["max_boxes"] = LIBSPX_NUM_BOXES
end

function show_boxes(n)
    for i = 1, n do
        firstbox[i]:Show()
    end
    for i = n+1, LIBSPX_NUM_BOXES do
        firstbox[i]:Hide()
    end
    firstbox["active_boxes"] = n

    for i = 1, n do
        lastbox[i]:Show()
    end
    for i = n+1, LIBSPX_NUM_BOXES do
        lastbox[i]:Hide()
    end
    lastbox["active_boxes"] = n

    for i = 1, n do
        boxes[i]:Show()
    end
    for i = n+1, LIBSPX_NUM_BOXES do
        boxes[i]:Hide()
    end
    boxes["active_boxes"] = n
end

function consume_message_queue()
    local elem_cnt = #LibSerpix.data_queue
    for i = 1, elem_cnt do
        local data = LibSerpix.data_queue[i]
        local namespace, data_table = unpack(data)
        LibSerpix.serializer.vals.u[namespace] = data_table
    end
    for i = 1, elem_cnt do
        table.remove(LibSerpix.data_queue, 1)
    end
end

local clock = 0
function OnUpdate(self, elapsed)
    serializer.vals.p.clock = clock
    consume_message_queue()
    local t = cbor.encode(serializer.vals)
    serializer.vals.u = {}
    local checksum = 0
    -- pad serialized message to multiple of 3 bytes to align with the three rgb channels in a pixel
    encode_size = #t
    while (Modulo(#t, 3) ~= 0) do
        t = t .. "\0"
    end
    -- pack string bytes into pixels
    -- cbor table isn't ordered so keep header as first pixel
    local j = 0
    for i = 1, #t, 3 do
        local r = string.byte(t, i)
        checksum = Modulo(checksum+r, 256)
        local g = string.byte(t, i+1)
        checksum = Modulo(checksum+g, 256)
        local b = string.byte(t, i+2)
        checksum = Modulo(checksum+b, 256)
        box_index = math.floor(i/3)+1
        j = j + 1
        set_texture_from_arr(boxes[box_index+2], rbgToColor(r,g,b))
    end
    header = checksum + bitshift_left(encode_size, 8)
    set_texture_from_arr(boxes[1], rbgToColor(42,0,69))
    set_texture_from_arr(boxes[2], integerToColor(header))
    show_boxes(2+(#t/3))
    clock = 0 -- Modulo(clock+1, 64)
end

init()
