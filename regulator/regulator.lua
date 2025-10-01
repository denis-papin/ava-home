---@diagnostic disable-next-line: undefined-global
regulation_schema = original_message

function regulate_radiator()
    for k, v in pairs(map_table) do
        print("clé:", k, "valeur:", v)
    end

    for k, v in pairs(actual_temperature) do
        print("heat clé:", k, "valeur:", v)
    end

    print("Bureau:", regulation_schema.tc_couloir)
end

