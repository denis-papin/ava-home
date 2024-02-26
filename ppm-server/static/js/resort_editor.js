console.log("Running resort_editor.js")

class ResortEditor {
    constructor(bus /** Bus */) {
        this.bus = bus
        this.channel = bus.getChannel("RESORT_INFO")
    }

    init() {
      console.log("init the resort_editor component")
      $("#resort_editor").show();
    }

    drop() {
        console.log("drop the resort_editor component")
        //$("#resort_editor").fadeOut(1);
        $("#resort_editor").remove();
    }

    on_key_up(event) {
        var inputValue = event.target.value;
        console.log("Nouvelle valeur de l'input :", inputValue);
        const address = {  'address' : inputValue }
        this.channel.sendData(address)
    }


}
