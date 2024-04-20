
function regulateRadiatorAction(mqttMsg) {

    const t = currentDateTime();

    document.getElementById("tc_salon_1").textContent = mqttMsg.tc_salon_1;
    document.getElementById("tc_salon_1_ts").textContent = t;

    document.getElementById("tc_bureau").textContent = mqttMsg.tc_bureau;
    document.getElementById("tc_bureau_ts").textContent = t;

    document.getElementById("tc_chambre").textContent = mqttMsg.tc_chambre_1;
    document.getElementById("tc_chambre_ts").textContent = t;

    document.getElementById("tc_couloir").textContent = mqttMsg.tc_couloir;
    document.getElementById("tc_couloir_ts").textContent = t;

    // console.log("tc_salon_1 à ", mqttMsg.tc_salon_1, currentDateTime())
}
function tsSalon1Action(mqttMsg) {

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