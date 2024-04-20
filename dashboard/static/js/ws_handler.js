const socket = new WebSocket('ws://127.0.0.1:9002');

// Connection opened
socket.addEventListener('open', (event) => {
    console.log('WebSocket connection opened:', event);
});

// Listen for messages
socket.addEventListener('message', (event) => {
    //const messagesDiv = document.getElementById('messages');
    //messagesDiv.innerHTML = `<p>Received: ${event.data}</p>`;
    const bridgeMsg = JSON.parse(event.data)
    const mqttMsg = JSON.parse(bridgeMsg.raw_message)

    switch (bridgeMsg.topic) {
        case 'regulator/regulate_radiator' : {
            regulateRadiatorAction(mqttMsg)
            break;
        }
        case 'zigbee2mdtt/ts_salon_1' : {
            tsSalon1Action(mqttMsg)
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

function sendMessage() {
    const messageInput = document.getElementById('messageInput');
    const message = messageInput.value;

    if (message.trim() !== '') {
        // Send a message to the WebSocket server
        socket.send(message);
        messageInput.value = '';
    }
}
