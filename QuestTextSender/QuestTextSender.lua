LibSerpix = LibStub("LibSerpix-0.1")

function init()
    UIParent:SetScript("OnUpdate", OnUpdate)
    f = CreateFrame("Frame")
    f:RegisterEvent("QUEST_DETAIL")
    f:RegisterEvent("QUEST_COMPLETE")
    f:RegisterEvent("QUEST_PROGRESS")
    f:RegisterEvent("GOSSIP_SHOW")
    f:SetScript("OnEvent", OnEvent)
end

function OnEvent(self, event, ...)
    local data = {}
    if event == "QUEST_DETAIL" then
        data["questText"] = GetQuestText()
        -- print(data["questText"])
    end
    if event == "GOSSIP_SHOW" then
        data["gossipText"] = GetGossipText()
        -- print(data["gossipText"])
    end
    if event == "QUEST_PROGRESS" then
        data["progressText"] = GetProgressText()
        -- print(data["progressText"])
    end
    if event == "QUEST_COMPLETE" then
        data["rewardText"] = GetRewardText()
        -- print(data["rewardText"])
    end
    LibSerpix.add_data("qtts", data)

end

init()
