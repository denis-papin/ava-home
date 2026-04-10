type room

@module("./dashboardApi.js")
external getApiBaseUrl: unit => string = "getApiBaseUrl"

@module("./dashboardApi.js")
external createPlaceholderRooms: unit => array<room> = "createPlaceholderRooms"

@module("./dashboardApi.js")
external fetchDashboardRooms: string => promise<array<room>> = "fetchDashboardRooms"

@module("./dashboardApi.js")
external spinRadiatorStatus: (string, string, string) => promise<string> = "spinRadiatorStatus"

@module("./dashboardApi.js")
external replaceRoomStatus: (array<room>, string, string) => array<room> = "replaceRoomStatus"

@module("./dashboardApi.js")
external formatElapsed: string => string = "formatElapsed"

@module("./dashboardApi.js")
external statusLabel: string => string = "statusLabel"

@module("./dashboardApi.js")
external statusTone: string => string = "statusTone"

@get external id: room => string = "id"
@get external label: room => string = "label"
@get external temperature: room => string = "temperature"
@get external targetTemperature: room => string = "targetTemperature"
@get external elapsedLabel: room => string = "elapsedLabel"
@get external tsCreate: room => Nullable.t<string> = "tsCreate"
@get external status: room => string = "status"
