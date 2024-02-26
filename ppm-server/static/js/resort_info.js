
console.log("Running resort_info.js")

class ResortInfo {
    constructor(bus /** Bus */) {
        this.bus = bus
        this.channel = bus.getChannel("RESORT_INFO")
    }

    on_resort_info(data /** Address */) {
      const addressEl = document.querySelector('#address');
      addressEl.innerHTML = data.address;
    }

    init() {
      console.log("init the component")
      const fn = (data) => { this.on_resort_info(data) }
      this.channel.subscribe(  fn /** (data) */ )
    }
}

