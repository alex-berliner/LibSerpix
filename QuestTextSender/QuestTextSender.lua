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
    if event == "QUEST_DETAIL" then
        local questText = {}
        questText["questText"] = GetQuestText()
        -- print(questText["questText"])
        LibSerpix.add_data("qtts", questText)
    end
    if event == "GOSSIP_SHOW"
        local gossipText = {}
        gossipText["gossipText"] = GetGossipText()
        -- print(gossipText["gossipText"])
        LibSerpix.add_data("qtts", gossipText)
    end
    if event == "QUEST_PROGRESS"
        local progressText = {}
        progressText["progressText"] = GetProgressText()
        -- print(progressText["progressText"])
        LibSerpix.add_data("qtts", progressText)
    end
    if event == "QUEST_COMPLETE" then
        local rewardText = {}
        rewardText["rewardText"] = GetRewardText()
        -- print(rewardText["rewardText"])
        LibSerpix.add_data("qtts", rewardText)
    end
end

init()
