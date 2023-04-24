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
    LibSerpix.register_addon("Demo")
end

-- Define the OnCombatLogEvent function
function OnCombatLogEvent(event, ...)
    CombatEventHandler(event, ...)
end

function CombatEventHandler(event, ...)
    function parse_heal(...)
        local timestamp, subevent, _, sourceGUID, sourceName, sourceFlags, sourceRaidFlags, destGUID, destName, destFlags, destRaidFlags = ...
        local spellId, spellName, spellSchool = select(12, ...)
        -- Check if the sourceGUID is your character's GUID
        if sourceGUID == UnitGUID("player") then
            healing, overhealing, absorbed, critical = select(15, ...)
            -- Add values to serializer
            LibSerpix.serializer.vals.u.Demo.tx_overhealing = (LibSerpix.serializer.vals.u.Demo.tx_overhealing or 0) + overhealing
            LibSerpix.serializer.vals.u.Demo.tx_healing = (LibSerpix.serializer.vals.u.Demo.tx_healing or 0) + healing - overhealing
            local questDescription, questObjectives = GetQuestLogQuestText()
            LibSerpix.serializer.vals.u.Demo.questDescription = questObjectives
            LibSerpix.serializer.vals.u.Demo.questObjectives = questObjectives

        end
    end
    function parse_dmg(...)
        local timestamp, subevent, _, sourceGUID, sourceName, sourceFlags, sourceRaidFlags, destGUID, destName, destFlags, destRaidFlags = ...
        local spellId, spellName, spellSchool = select(12, ...)
        if sourceGUID == UnitGUID("player") then
            amount, overkill, school, resisted, blocked, absorbed, critical, glancing, crushing, isOffHand = select(15, ...)
            -- Add values to LibSerpix.serializer
            LibSerpix.serializer.vals.u.Demo.tx_damage = (LibSerpix.serializer.vals.u.Demo.tx_damage or 0) + amount
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
