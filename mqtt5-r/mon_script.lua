function process_map()
    for k,v in pairs(map_table) do
        print("clé:", k, "valeur:", v)
    end

    for i, loop in ipairs(all_loops) do
        print("=== HardLoop #"..i.." ===")
        print("Name:", loop.name)
        print("Device count:", loop.device_count)

        local devices = loop:list_devices()
        for j, dev in ipairs(devices) do
            print("  Device " .. j .. ": " .. dev)
        end
    end

end

