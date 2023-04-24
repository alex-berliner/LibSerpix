-- perform this before serializing
function LibSerpix.serializer.user_update()
    -- serializer.vals.a = "a"
end

function init()
    UIParent:SetScript("OnUpdate", OnUpdate)
    f = CreateFrame("Frame")
    -- Register the OnCombatLogEvent function to
    -- the COMBAT_LOG_EVENT_UNFILTERED event
    f:RegisterEvent("COMBAT_LOG_EVENT_UNFILTERED")
    f:SetScript("OnEvent", OnCombatLogEvent)
end

-- Define the OnCombatLogEvent function
function OnCombatLogEvent(event, ...)
    serializer.CombatEventHandler(event, ...)
end

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
            local questDescription, questObjectives = GetQuestLogQuestText()
            serializer.vals.questDescription = questObjectives
            serializer.vals.questObjectives = questObjectives
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

init()
