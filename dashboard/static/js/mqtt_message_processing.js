
function regulateRadiatorAction(mqttMsg) {
    const t = currentDateTime();
    document.getElementById("tc_salon_1").textContent = mqttMsg.tc_salon_1;
    document.getElementById("tc_salon_1_ts").textContent = t;
    localStorage.setItem('tc_salon_1', mqttMsg.tc_salon_1);
    localStorage.setItem('tc_ts', t);

    document.getElementById("tc_bureau").textContent = mqttMsg.tc_bureau;
    document.getElementById("tc_bureau_ts").textContent = t;
    localStorage.setItem('tc_bureau', mqttMsg.tc_bureau);

    document.getElementById("tc_chambre").textContent = mqttMsg.tc_chambre_1;
    document.getElementById("tc_chambre_ts").textContent = t;
    localStorage.setItem('tc_chambre_1', mqttMsg.tc_chambre_1);

    document.getElementById("tc_couloir").textContent = mqttMsg.tc_couloir;
    document.getElementById("tc_couloir_ts").textContent = t;
    localStorage.setItem('tc_couloir', mqttMsg.tc_couloir);
}

function tsSalonAction(mqttMsg) {
    tsGenericAction("salon_temperature", "salon_elapse", mqttMsg)
}

function tsCouloirAction(mqttMsg) {
    tsGenericAction("couloir_temperature", "couloir_elapse", mqttMsg)
}

function tsBureauAction(mqttMsg) {
    tsGenericAction("bureau_temperature", "bureau_elapse", mqttMsg)
}

function tsChambreAction(mqttMsg) {
    tsGenericAction("chambre_temperature", "chambre_elapse", mqttMsg)
}

function tsGenericAction(tempId, elapseId, mqttMsg) {
    document.getElementById(tempId).textContent = mqttMsg.temperature;
    document.getElementById(elapseId).textContent = '--'; // find a way to implement a counter h:m
}

//// RAD
function initRadStatus() {
    for (var room of ['salon', 'bureau', 'chambre', 'couloir']) {
        let iconEl = document.getElementById(`${room}_rad_icon`);
        let status = iconEl.getAttribute('data-status');
        roomStatus[room] = status
        externalRad(room, {mode: status})
    }
}

function externalRad(room, mqttMsg) {
//			  <div id="salon_rad_icon" class="w-3 h-3 bg-green-500 rounded-full"></div>
// 			  <span id="salon_rad_message" class="ml-1 text-green-500">En chauffe</span>

    let color = '';
    let text = '';
    switch (mqttMsg.mode.toUpperCase()) {
        case "CFT" : {
            color = "green"
            text = "En chauffe"
            break;
        }
        case "STOP" : {
            color = "red"
            text = "Arrêt"
            break;
        }
        case "ECO" : {
            color = "yellow"
            text = "Désactivé"
            break;
        }
    }
    const divElement = document.getElementById(`${room}_rad_icon`);
    removeBg(divElement)
    divElement.classList.add(`bg-${color}-500`);
    const divElementMessage = document.getElementById(`${room}_rad_message`);
    removeBgText(divElementMessage)
    divElementMessage.classList.add(`text-${color}-500`);
    divElementMessage.innerText = text
}

function removeBg(divElement) {
    divElement.classList.remove("bg-green-500");
    divElement.classList.remove("bg-yellow-500");
    divElement.classList.remove("bg-red-500");
}

function removeBgText(divElement) {
    divElement.classList.remove("text-green-500");
    divElement.classList.remove("text-yellow-500");
    divElement.classList.remove("text-red-500");
}

function currentDateTime() /*: string*/ {
    // Obtenir la date actuelle
    var date = new Date();

    // Tableaux de correspondance pour les noms des mois et des jours de la semaine
    var mois = [
        "jan.", "fév.", "mar.", "avr.", "mai", "juin",
        "juil.", "août", "sept.", "oct.", "nov.", "déc."
    ];

// Récupérer le jour, le mois et l'année
    var jour = date.getDate();
    var moisActuel = mois[date.getMonth()];
    var annee = date.getFullYear();

// Récupérer l'heure et les minutes
    var heures = date.getHours();
    var minutes = date.getMinutes();

// Formatage des minutes pour ajouter un zéro si elles sont inférieures à 10
    if (minutes < 10) {
        minutes = "0" + minutes;
    }

// Création du texte final
    var texteDateHeure/*: string*/ = "le " + jour + " " + moisActuel + " à " + heures + "h" + minutes;

    // console.log(texteDateHeure); // Affiche le texte au format demandé
    return texteDateHeure
}

document.addEventListener("DOMContentLoaded", function() {
    // Votre code JavaScript à exécuter une fois que le DOM est prêt
    console.log("Le DOM est prêt !");
    initRadStatus()

    // Mettre les dernières valeurs cibles connues
    let tc_salon_1 = localStorage.getItem('tc_salon_1');
    let tc_bureau = localStorage.getItem('tc_bureau');
    let tc_chambre_1 = localStorage.getItem('tc_chambre_1');
    let tc_couloir = localStorage.getItem('tc_couloir');

    if (tc_salon_1 != undefined) {
        const mqttMsg = {
            "tc_bureau": tc_bureau,
            "tc_salon_1": tc_salon_1,
            "tc_salon_2": 0.0,
            "tc_chambre_1": tc_chambre_1,
            "tc_couloir": tc_couloir,
            "mode": "J"
        }
        regulateRadiatorAction(mqttMsg)
    }
});


