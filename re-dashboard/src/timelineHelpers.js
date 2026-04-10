function normalizeApiTimestamp(value) {
  return value && value.endsWith("Z") ? value : `${value}Z`;
}

function parseApiDate(value) {
  return new Date(normalizeApiTimestamp(value));
}

function pad2(value) {
  return value < 10 ? `0${value}` : `${value}`;
}

function toLocalInputValue(date) {
  return `${date.getFullYear()}-${pad2(date.getMonth() + 1)}-${pad2(date.getDate())}T${pad2(
    date.getHours(),
  )}:${pad2(date.getMinutes())}`;
}

function toApiDateTime(value) {
  return value.length === 16 ? `${value}:00` : value;
}

function formatDateLabel(value) {
  const date = parseApiDate(value);
  const monthNames = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
  return `${monthNames[date.getMonth()]} ${date.getDate()}, ${pad2(date.getHours())}:${pad2(
    date.getMinutes(),
  )}`;
}

function formatTemp(value) {
  return `${value.toFixed(1)}°C`;
}

function capitalize(value) {
  return value.length === 0 ? value : `${value.slice(0, 1).toUpperCase()}${value.slice(1)}`;
}

function colorForMode(mode) {
  switch ((mode || "").toUpperCase()) {
    case "CFT":
    case "CRF":
      return "rgba(220, 95, 69, 0.22)";
    case "ECO":
      return "rgba(98, 153, 120, 0.22)";
    case "FRO":
      return "rgba(130, 176, 228, 0.22)";
    case "STOP":
      return "rgba(92, 135, 198, 0.22)";
    default:
      return "rgba(188, 168, 130, 0.22)";
  }
}

function timezoneLabel() {
  const now = new Date();
  const offsetMinutes = -now.getTimezoneOffset();
  const sign = offsetMinutes >= 0 ? "+" : "-";
  const absoluteMinutes = Math.abs(offsetMinutes);
  const offsetHours = Math.floor(absoluteMinutes / 60);
  const offsetRemainderMinutes = absoluteMinutes % 60;
  const zoneName = Intl.DateTimeFormat().resolvedOptions().timeZone || "Local time";
  return `${zoneName}, UTC${sign}${pad2(offsetHours)}:${pad2(offsetRemainderMinutes)}`;
}

function allReadings(sections) {
  return sections.flatMap((section) =>
    section.temperatures.map((reading) => ({
      ...reading,
      mode: section.mode,
    })),
  );
}

function uniqueModes(sections) {
  return [...new Set(sections.map((section) => section.mode))];
}

function computeStats(readings) {
  if (readings.length === 0) {
    return [];
  }

  let min = readings[0].temperature;
  let max = readings[0].temperature;
  let total = 0;

  for (const reading of readings) {
    if (reading.temperature < min) min = reading.temperature;
    if (reading.temperature > max) max = reading.temperature;
    total += reading.temperature;
  }

  return [
    { label: "Samples", value: `${readings.length}` },
    { label: "Minimum", value: formatTemp(min) },
    { label: "Maximum", value: formatTemp(max) },
    { label: "Average", value: formatTemp(total / readings.length) },
  ];
}

export function buildTimelineViewModel(data) {
  const sections = data.sections || [];
  const readings = allReadings(sections);

  if (readings.length === 0) {
    return {
      chart: null,
      emptyMessage: "Aucune mesure de temperature n'a ete retournee pour cette periode.",
      legend: uniqueModes(sections).map((mode) => ({ mode, color: colorForMode(mode) })),
      stats: [],
      status: "0 point de mesure charge.",
      title: `${capitalize(data.room)} temperature timeline`,
      subtitle: `${formatDateLabel(data.startDateTime)} to ${formatDateLabel(data.endDateTime)}`,
    };
  }

  const width = 1100;
  const height = 520;
  const marginLeft = 62;
  const marginRight = 24;
  const marginTop = 28;
  const marginBottom = 52;
  const plotWidth = width - marginLeft - marginRight;
  const plotHeight = height - marginTop - marginBottom;
  const startMs = parseApiDate(data.startDateTime).getTime();
  const endMs = parseApiDate(data.endDateTime).getTime();
  const temperatures = readings.map((reading) => reading.temperature);
  const minTemp = Math.min(...temperatures);
  const maxTemp = Math.max(...temperatures);
  const padding = maxTemp - minTemp > 0.5 ? (maxTemp - minTemp) * 0.14 : 0.5;
  const yMin = minTemp - padding;
  const yMax = maxTemp + padding;
  const xRange = endMs - startMs || 1;
  const yRange = yMax - yMin || 1;
  const xFor = (value) => marginLeft + ((value - startMs) / xRange) * plotWidth;
  const yFor = (value) => marginTop + plotHeight - ((value - yMin) / yRange) * plotHeight;

  const linePath = readings
    .map((reading, index) => {
      const x = xFor(parseApiDate(reading.measuredAt).getTime()).toFixed(2);
      const y = yFor(reading.temperature).toFixed(2);
      return `${index === 0 ? "M" : "L"} ${x} ${y}`;
    })
    .join(" ");

  const rects = sections.map((section) => {
    const startX = xFor(parseApiDate(section.startDateTime).getTime());
    const endX = xFor(parseApiDate(section.endDateTime).getTime());

    return {
      x: startX,
      y: marginTop,
      width: Math.max(0, endX - startX),
      height: plotHeight,
      fill: colorForMode(section.mode),
    };
  });

  const yGuides = Array.from({ length: 6 }, (_, index) => {
    const value = yMin + (index / 5) * (yMax - yMin);
    const y = yFor(value);
    return {
      lineX1: marginLeft,
      lineY1: y,
      lineX2: width - marginRight,
      lineY2: y,
      labelX: marginLeft - 12,
      labelY: y + 4,
      label: formatTemp(value),
      textAnchor: "end",
    };
  });

  const xGuides = Array.from({ length: 9 }, (_, index) => {
    const ms = startMs + (index / 8) * (endMs - startMs);
    const x = xFor(ms);
    const labelDate = new Date(ms);
    return {
      lineX1: x,
      lineY1: marginTop,
      lineX2: x,
      lineY2: height - marginBottom,
      labelX: x,
      labelY: height - marginBottom + 24,
      label: `${pad2(labelDate.getHours())}:${pad2(labelDate.getMinutes())}`,
      textAnchor: "middle",
    };
  });

  const points = readings.map((reading) => ({
    cx: xFor(parseApiDate(reading.measuredAt).getTime()),
    cy: yFor(reading.temperature),
  }));

  const sectionCount = data.sectionCount || sections.length;

  return {
    chart: {
      viewBox: `0 0 ${width} ${height}`,
      linePath,
      axisBottomY: height - marginBottom,
      axisLeftX: marginLeft,
      axisRightX: width - marginRight,
      axisTopY: marginTop,
      axisLabelX: `Hours (${timezoneLabel()})`,
      axisLabelY: "Temperature",
      axisLabelXPosX: width / 2,
      axisLabelXPosY: height - 10,
      axisLabelYPosX: 16,
      axisLabelYPosY: height / 2,
      axisLabelYRotate: `rotate(-90 16 ${height / 2})`,
      rects,
      yGuides,
      xGuides,
      points,
    },
    emptyMessage: null,
    legend: uniqueModes(sections).map((mode) => ({ mode, color: colorForMode(mode) })),
    stats: computeStats(readings),
    status: `${sectionCount} mode section${sectionCount > 1 ? "s" : ""} loaded.`,
    title: `${capitalize(data.room)} temperature timeline`,
    subtitle: `${formatDateLabel(data.startDateTime)} to ${formatDateLabel(data.endDateTime)}`,
  };
}

export function makeDefaultTimelineFilters() {
  const now = new Date();
  const startDate = new Date(now.getTime() - 24 * 60 * 60 * 1000);

  return {
    room: "bureau",
    startDateTime: toLocalInputValue(startDate),
    endDateTime: toLocalInputValue(now),
  };
}

export function readTimelineFilters(form) {
  const formData = new FormData(form);

  return {
    room: `${formData.get("room") || "bureau"}`,
    startDateTime: `${formData.get("startDateTime") || ""}`,
    endDateTime: `${formData.get("endDateTime") || ""}`,
  };
}

export async function fetchRoomTemperatureByMode(apiBaseUrl, room, startDateTime, endDateTime) {
  const params = new URLSearchParams();
  params.set("room", room);
  params.set("startDateTime", toApiDateTime(startDateTime));
  params.set("endDateTime", toApiDateTime(endDateTime));

  const response = await fetch(`${apiBaseUrl}/room_temperature_by_mode?${params.toString()}`, {
    cache: "no-store",
  });

  const body = await response.json();
  if (!response.ok) {
    throw new Error(body.error || `dashboard-api a repondu ${response.status}`);
  }

  return body;
}
