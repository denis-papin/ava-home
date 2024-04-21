let socket = new WebSocket('ws://192.168.0.99:9002');

let roomStatus = {}
roomStatus['salon'] = ''
roomStatus['bureau'] = ''
roomStatus['couloir'] = ''
roomStatus['chambre'] = ''


document.addEventListener('visibilitychange', function() {
    if (document.visibilityState === 'visible') {
        location.reload()
    }
});


// Connection opened
socket.addEventListener('open', (event) => {
    console.log('WebSocket connection opened:', event);
});

// Listen for messages
socket.addEventListener('message', (event) => {
    const bridgeMsg = JSON.parse(event.data)
    const mqttMsg = JSON.parse(bridgeMsg.raw_message)

    switch (bridgeMsg.topic) {
        case 'regulator/regulate_radiator' : {
            regulateRadiatorAction(mqttMsg)
            break;
        }
        case 'zigbee2mqtt/ts_salon_1' : {
            tsSalonAction(mqttMsg)
            break;
        }
        case 'zigbee2mqtt/ts_bureau' : {
            tsBureauAction(mqttMsg)
            break;
        }
        case 'zigbee2mqtt/ts_chambre_1' : {
            tsChambreAction(mqttMsg)
            break;
        }
        case 'zigbee2mqtt/ts_couloir' : {
            tsCouloirAction(mqttMsg)
            break;
        }
        case 'external/rad_salon' : {
            externalRad('salon', mqttMsg)
            break;
        }
        case 'external/rad_bureau' : {
            externalRad('bureau', mqttMsg)
            break;
        }
        case 'external/rad_chambre' : {
            externalRad('chambre', mqttMsg)
            break;
        }
        case 'external/rad_couloir' : {
            externalRad('couloir', mqttMsg)
            break;
        }
    }

    console.log(bridgeMsg.topic, mqttMsg)
});

// Connection closed
socket.addEventListener('close', (event) => {
    console.log('WebSocket connection closed:', event);
});

// Connection error
socket.addEventListener('error', (event) => {
    console.error('WebSocket error:', event);
});

function spinStatus(room) {
    if (roomStatus[room] === 'STOP') {
        roomStatus[room] = 'CFT'
    } else if (roomStatus[room] === 'CFT') {
        roomStatus[room] = 'ECO'
    } else {
        roomStatus[room] = 'STOP'
    }
    turnRadiator(room, roomStatus[room])
}

function turnRadiator(room, status) {
    const mqttMsg = { mode : status}
    sendMessage(`external/rad_${room}`, mqttMsg)
}

function sendMessage(topic, mqttMsg) {
    const rawMqttMsg = JSON.stringify(mqttMsg)
    // const bridgeMessage = { topic : "external/rad_couloir", raw_message: "{\"mode\":\"CFT\"}"}
    const bridgeMessage = { topic : topic, raw_message: rawMqttMsg}
    const rawBridgeMessage = JSON.stringify(bridgeMessage)
    socket.send(rawBridgeMessage);
}
