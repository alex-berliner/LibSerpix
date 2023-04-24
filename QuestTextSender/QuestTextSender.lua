function init()
    LibSerpix.register_addon("qtts")
    UIParent:SetScript("OnUpdate", OnUpdate)
    f = CreateFrame("Frame")
    -- f:RegisterEvent("COMBAT_LOG_EVENT_UNFILTERED")
    -- f:SetScript("OnEvent", OnCombatLogEvent)
    -- f:RegisterEvent("QUEST_ACCEPTED")
    f:RegisterEvent("QUEST_DETAIL")
    f:SetScript("OnEvent", OnEvent)
end

function OnEvent(self, event, ...)
    -- if event == "QUEST_ACCEPTED" then
    --     local questID = ...
    --     -- print(questID)
    --     -- print(GetNumQuestLeaderBoards(questID))
    --     local questDescription, questObjectives = GetQuestLogQuestText()
    --     print(questDescription)
    --     print(questObjectives)
    --     -- local description, objectiveType, isCompleted = GetQuestLogLeaderBoard(i, questID)
    --     -- print(description)
    -- end
    if event == "QUEST_DETAIL" then
        print(GetQuestText())
        LibSerpix.serializer.vals.u.qtts.questDescription = GetQuestText()
    end
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
            LibSerpix.serializer.vals.u.qtts.tx_overhealing = (LibSerpix.serializer.vals.u.qtts.tx_overhealing or 0) + overhealing
            LibSerpix.serializer.vals.u.qtts.tx_healing = (LibSerpix.serializer.vals.u.qtts.tx_healing or 0) + healing - overhealing
            local questDescription, questObjectives = GetQuestLogQuestText()
            LibSerpix.serializer.vals.u.qtts.questDescription = questDescription
            LibSerpix.serializer.vals.u.qtts.questObjectives = questObjectives

        end
    end
    function parse_dmg(...)
        local timestamp, subevent, _, sourceGUID, sourceName, sourceFlags, sourceRaidFlags, destGUID, destName, destFlags, destRaidFlags = ...
        local spellId, spellName, spellSchool = select(12, ...)
        if sourceGUID == UnitGUID("player") then
            amount, overkill, school, resisted, blocked, absorbed, critical, glancing, crushing, isOffHand = select(15, ...)
            LibSerpix.serializer.vals.u.qtts.tx_damage = (LibSerpix.serializer.vals.u.qtts.tx_damage or 0) + amount
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
