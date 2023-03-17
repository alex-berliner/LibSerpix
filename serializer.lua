-- User modifiable file to change serialized data

serializer = {}

function serializer.CombatEventHandler(event, ...)
    function parse_heal(...)
        local timestamp, subevent, _, sourceGUID, sourceName, sourceFlags, sourceRaidFlags, destGUID, destName, destFlags, destRaidFlags = ...
        local spellId, spellName, spellSchool = select(12, ...)
        -- Check if the sourceGUID is your character's GUID
        if sourceGUID == UnitGUID("player") then
            healing, overhealing, absorbed, critical = select(15, ...)
            -- Add values to serializer
            serializer.vals.tx_overhealing = (serializer.vals.tx_overhealing or 0) + overhealing
            serializer.vals.tx_healing = (serializer.vals.tx_healing or 0) + healing - overhealing
        end
    end
    function parse_dmg(...)
        local timestamp, subevent, _, sourceGUID, sourceName, sourceFlags, sourceRaidFlags, destGUID, destName, destFlags, destRaidFlags = ...
        local spellId, spellName, spellSchool = select(12, ...)
        if sourceGUID == UnitGUID("player") then
            amount, overkill, school, resisted, blocked, absorbed, critical, glancing, crushing, isOffHand = select(15, ...)
            -- Add values to serializer
            serializer.vals.tx_damage = (serializer.vals.tx_damage or 0) + amount
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

local function init_user_serialized_data()
    serializer.vals.tx_healing = "sss"
    serializer.vals.tx_overhealing = 0
    serializer.vals.tx_damage = 0
end

function get_serializer()
    return serializer
end

local function serializer_init()
    serializer.vals = {}
    init_user_serialized_data()
end

serializer_init()
