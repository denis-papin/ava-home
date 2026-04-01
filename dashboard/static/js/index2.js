const rooms = ["salon", "bureau", "chambre", "couloir"];
const roomStatus = {};
const nextStatus = {
    STOP: "CFT",
    CFT: "ECO",
    ECO: "STOP"
};

document.addEventListener("DOMContentLoaded", () => {
    rooms.forEach((room) => {
        const status = document
            .getElementById(`${room}_rad_icon`)
            .getAttribute("data-status");
        roomStatus[room] = status || "STOP";
        renderStatus(room, roomStatus[room]);
        refreshElapsedLabel(room);
    });

    refreshData();
    setInterval(refreshData, 30000);
});

async function refreshData() {
    const dashboardDataUrl = document.body.getAttribute("data-dashboard-data-url");

    try {
        const response = await fetch(dashboardDataUrl, { cache: "no-store" });
        if (!response.ok) {
            throw new Error(`HTTP ${response.status}`);
        }

        const data = await response.json();
        rooms.forEach((room) => updateRoom(room, data));
        updateTargets(data);
    } catch (error) {
        console.error("Cannot refresh dashboard/index2 data", error);
        rooms.forEach(refreshElapsedLabel);
    }
}

function updateRoom(room, data) {
    const temperature = data[`${room}_temperature`];
    const tsCreate = data[`${room}_ts_create`];
    const status = data[`${room}_status`];

    if (temperature !== undefined) {
        document.getElementById(`${room}_temperature`).textContent = temperature;
    }

    if (tsCreate) {
        const elapsedEl = document.getElementById(`${room}_elapse`);
        elapsedEl.setAttribute("data-ts-create", tsCreate);
    }

    refreshElapsedLabel(room);

    if (status) {
        roomStatus[room] = status;
        renderStatus(room, status);
    }
}

function updateTargets(data) {
    const targetIds = {
        salon: "tc_salon_1",
        bureau: "tc_bureau",
        chambre: "tc_chambre",
        couloir: "tc_couloir"
    };

    Object.entries(targetIds).forEach(([room, elementId]) => {
        const value = room === "salon" ? data.tc_salon : data[elementId];
        if (value !== undefined) {
            document.getElementById(elementId).textContent = value;
        }
    });
}

function refreshElapsedLabel(room) {
    const elapsedEl = document.getElementById(`${room}_elapse`);
    const tsCreate = elapsedEl.getAttribute("data-ts-create");
    elapsedEl.textContent = formatElapsed(tsCreate);
}

function formatElapsed(tsCreate) {
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

function renderStatus(room, status) {
    const presentation = getStatusPresentation(status);
    const iconEl = document.getElementById(`${room}_rad_icon`);
    const messageEl = document.getElementById(`${room}_rad_message`);

    removeStatusClasses(iconEl, "bg");
    removeStatusClasses(messageEl, "text");

    iconEl.classList.add(presentation.bgClass);
    iconEl.setAttribute("data-status", status);
    messageEl.classList.add(presentation.textClass);
    messageEl.textContent = presentation.label;
}

function getStatusPresentation(status) {
    switch ((status || "").toUpperCase()) {
        case "CFT":
            return { bgClass: "bg-green-500", textClass: "text-green-500", label: "En chauffe" };
        case "ECO":
            return { bgClass: "bg-yellow-500", textClass: "text-yellow-500", label: "Desactive" };
        case "STOP":
        default:
            return { bgClass: "bg-red-500", textClass: "text-red-500", label: "Arret" };
    }
}

function removeStatusClasses(element, prefix) {
    element.classList.remove(`${prefix}-green-500`);
    element.classList.remove(`${prefix}-yellow-500`);
    element.classList.remove(`${prefix}-red-500`);
}

async function spinStatus(room) {
    const currentStatus = roomStatus[room] || "STOP";
    const targetStatus = nextStatus[currentStatus] || "STOP";
    const url = `/dashboard/index2/radiator/${room}`;

    try {
        const response = await fetch(url, {
            method: "POST",
            headers: {
                "Content-Type": "application/json"
            },
            body: JSON.stringify({ mode: targetStatus })
        });

        if (!response.ok) {
            throw new Error(`HTTP ${response.status}`);
        }

        const body = await response.json();
        roomStatus[room] = body.status || targetStatus;
        renderStatus(room, roomStatus[room]);
    } catch (error) {
        console.error(`Cannot update radiator mode for ${room}`, error);
    }
}

window.spinStatus = spinStatus;
