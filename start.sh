gnome-terminal -- /home/denis/Projects/wks-ava-home/ava-home/target/debug/regulator-heart-beat  &
: "${AVA_REGULATOR_PASSWORD:?Missing AVA_REGULATOR_PASSWORD}"
: "${AVA_REGULATOR_TOKEN:?Missing AVA_REGULATOR_TOKEN}"
gnome-terminal -- /home/denis/Projects/wks-ava-home/ava-home/target/debug/regulator "$AVA_REGULATOR_PASSWORD" "$AVA_REGULATOR_TOKEN" &
gnome-terminal -- /home/denis/Projects/wks-ava-home/ava-home/target/debug/event-storage &
