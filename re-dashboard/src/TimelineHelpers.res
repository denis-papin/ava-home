type filters
type response
type viewModel
type legendItem
type statItem
type chartModel
type chartRect
type chartGuide
type chartPoint
type form

@module("./timelineHelpers.js")
external makeDefaultTimelineFilters: unit => filters = "makeDefaultTimelineFilters"

@module("./timelineHelpers.js")
external readTimelineFilters: form => filters = "readTimelineFilters"

@module("./timelineHelpers.js")
external fetchRoomTemperatureByMode: (string, string, string, string) => promise<response> =
  "fetchRoomTemperatureByMode"

@module("./timelineHelpers.js")
external buildTimelineViewModel: response => viewModel = "buildTimelineViewModel"

@get external room: filters => string = "room"
@get external startDateTime: filters => string = "startDateTime"
@get external endDateTime: filters => string = "endDateTime"

@get external title: viewModel => string = "title"
@get external subtitle: viewModel => string = "subtitle"
@get external status: viewModel => string = "status"
@get external legend: viewModel => array<legendItem> = "legend"
@get external stats: viewModel => array<statItem> = "stats"
@get external chart: viewModel => Nullable.t<chartModel> = "chart"
@get external emptyMessage: viewModel => Nullable.t<string> = "emptyMessage"

@get external mode: legendItem => string = "mode"
@get external color: legendItem => string = "color"

@get external statLabel: statItem => string = "label"
@get external statValue: statItem => string = "value"

@get external viewBox: chartModel => string = "viewBox"
@get external linePath: chartModel => string = "linePath"
@get external axisBottomY: chartModel => float = "axisBottomY"
@get external axisLeftX: chartModel => float = "axisLeftX"
@get external axisRightX: chartModel => float = "axisRightX"
@get external axisTopY: chartModel => float = "axisTopY"
@get external axisLabelX: chartModel => string = "axisLabelX"
@get external axisLabelY: chartModel => string = "axisLabelY"
@get external axisLabelXPosX: chartModel => float = "axisLabelXPosX"
@get external axisLabelXPosY: chartModel => float = "axisLabelXPosY"
@get external axisLabelYPosX: chartModel => float = "axisLabelYPosX"
@get external axisLabelYPosY: chartModel => float = "axisLabelYPosY"
@get external axisLabelYRotate: chartModel => string = "axisLabelYRotate"
@get external rects: chartModel => array<chartRect> = "rects"
@get external yGuides: chartModel => array<chartGuide> = "yGuides"
@get external xGuides: chartModel => array<chartGuide> = "xGuides"
@get external points: chartModel => array<chartPoint> = "points"

@get external rectX: chartRect => float = "x"
@get external rectY: chartRect => float = "y"
@get external rectWidth: chartRect => float = "width"
@get external rectHeight: chartRect => float = "height"
@get external fill: chartRect => string = "fill"

@get external lineX1: chartGuide => float = "lineX1"
@get external lineY1: chartGuide => float = "lineY1"
@get external lineX2: chartGuide => float = "lineX2"
@get external lineY2: chartGuide => float = "lineY2"
@get external labelX: chartGuide => float = "labelX"
@get external labelY: chartGuide => float = "labelY"
@get external labelText: chartGuide => string = "label"
@get external textAnchor: chartGuide => string = "textAnchor"

@get external cx: chartPoint => float = "cx"
@get external cy: chartPoint => float = "cy"
