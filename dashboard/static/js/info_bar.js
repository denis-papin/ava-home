
console.log("Running info_bar.js")

class InfoBar {
    constructor(bus /** Bus */) {
        this.bus = bus
        this.channel = this.bus.getChannel("SHOW_FORM")
        this.channel2 = this.bus.getChannel("SHOW_FORM2")
        this.form_is_visible = false;
        this.form2_is_visible = false;
    }

    init() {
      console.log("init the component info_bar.js")
      $("#info_bar").show();
    }

    on_click(event) {
        console.log("Send click event")

        this.form_is_visible = ! this.form_is_visible

        this.channel.sendData({'show' : this.form_is_visible}) // Send data on the SHOW_FORM channel
//      const addressEl = document.querySelector('#address');
//      addressEl.innerHTML = data.address;
    }


    on_click_direct(event) {
        console.log("Send click event 2")
        this.form2_is_visible = ! this.form2_is_visible
        //this.channel2.sendData({'show' : this.form2_is_visible}) // Send data on the SHOW_FORM channel
        let  componentName = "resort_editor2"
        let html = "<div id='resort_editor2' class=\"visual_resort_editor position_resort_editor\"> "+
                   "   <div class=\"visual_input_label\">Nom de la rue</div> "+
                   "   <input class=\"visual_input_field position_input_field\" value=\"\" onkeyup=\"CLoader.findObject('resort_editor').on_key_up(event)\" /> "+

                   "   <div class=\"visual_input_label\">Code postal</div> "+
                   "   <input class=\"visual_input_field position_input_field\" value=\"\" /> "+

                   "   <div class=\"visual_input_label\">Ville</div> "+
                   "   <input class=\"visual_input_field position_input_field\" value=\"\"  /> "+
                   " </div>"

        if (this.form2_is_visible) {
            let holderId = "holder_resort_editor2"
            $(`#${holderId}`).hide()
            $(`#${holderId}`).html(html)
            // $(`#${componentName}`).hide()
            $(`#${holderId}`).show()
        } else {
            $(`#${componentName}`).remove()
        }
    }

}

