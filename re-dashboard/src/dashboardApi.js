const ROOM_DEFINITIONS = [
  { id: "salon", label: "Salon", targetKey: "tc_salon" },
  { id: "bureau", label: "Bureau", targetKey: "tc_bureau" },
  { id: "chambre", label: "Chambre", targetKey: "tc_chambre" },
  { id: "couloir", label: "Couloir", targetKey: "tc_couloir" },
];

const NEXT_STATUS = {
  STOP: "CFT",
  CFT: "ECO",
  ECO: "STOP",
};

const DEFAULT_API_ORIGIN = "http://127.0.0.1:2090";

export function getApiBaseUrl() {
  const configuredOrigin =
    typeof import.meta !== "undefined" && import.meta.env?.VITE_DASHBOARD_API_ORIGIN
      ? import.meta.env.VITE_DASHBOARD_API_ORIGIN
      : DEFAULT_API_ORIGIN;

  return `${configuredOrigin.replace(/\/$/, "")}/dashboard-api`;
}

export function createPlaceholderRooms() {
  return ROOM_DEFINITIONS.map((room) => ({
    id: room.id,
    label: room.label,
    temperature: "--",
    targetTemperature: "--",
    elapsedLabel: "--",
    tsCreate: null,
    status: "STOP",
  }));
}

export async function fetchDashboardRooms(apiBaseUrl) {
  const response = await fetch(`${apiBaseUrl}/index/data`, {
    cache: "no-store",
  });

  if (!response.ok) {
    throw new Error(`dashboard-api /index/data a repondu ${response.status}`);
  }

  const payload = await response.json();

  return ROOM_DEFINITIONS.map((room) => {
    const tsCreate = payload[`${room.id}_ts_create`] ?? null;

    return {
      id: room.id,
      label: room.label,
      temperature: payload[`${room.id}_temperature`] ?? "--",
      targetTemperature: payload[room.targetKey] ?? "--",
      elapsedLabel: payload[`${room.id}_elapse`] ?? formatElapsed(tsCreate),
      tsCreate,
      status: normalizeStatus(payload[`${room.id}_status`]),
    };
  });
}

export async function spinRadiatorStatus(apiBaseUrl, roomId, currentStatus) {
  const targetStatus = NEXT_STATUS[normalizeStatus(currentStatus)] ?? "STOP";
  const response = await fetch(`${apiBaseUrl}/index/radiator/${roomId}`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({ mode: targetStatus }),
  });

  if (!response.ok) {
    throw new Error(`dashboard-api /index/radiator/${roomId} a repondu ${response.status}`);
  }

  const body = await response.json().catch(() => ({}));
  return normalizeStatus(body.status ?? body.mode ?? targetStatus);
}

export function replaceRoomStatus(rooms, roomId, nextStatus) {
  return rooms.map((room) =>
    room.id === roomId ? { ...room, status: normalizeStatus(nextStatus) } : room,
  );
}

export function formatElapsed(tsCreate) {
  if (!tsCreate) {
    return "--";
  }

  const createdAt = new Date(tsCreate);
  if (Number.isNaN(createdAt.getTime())) {
    return "--";
  }

  const diffMs = Math.max(0, Date.now() - createdAt.getTime());
  const totalMinutes = Math.floor(diffMs / 60000);
  const hours = Math.floor(totalMinutes / 60);
  const minutes = totalMinutes % 60;

  if (hours > 0) {
    return `${hours} h ${minutes} min`;
  }

  return `${minutes} min`;
}

export function statusLabel(status) {
  switch (normalizeStatus(status)) {
    case "CFT":
      return "En chauffe";
    case "ECO":
      return "Desactive";
    case "STOP":
    default:
      return "Arret";
  }
}

export function statusTone(status) {
  switch (normalizeStatus(status)) {
    case "CFT":
      return "green";
    case "ECO":
      return "yellow";
    case "STOP":
    default:
      return "red";
  }
}

function normalizeStatus(status) {
  const normalized = `${status ?? "STOP"}`.toUpperCase();
  return NEXT_STATUS[normalized] ? normalized : normalized === "STOP" ? "STOP" : "STOP";
}
