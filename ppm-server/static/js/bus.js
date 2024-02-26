class CLoader {
    constructor(cdn /** string */, bus /** Bus */) {
        this.cdn = cdn;
        this.bus = bus;
    }

    loadComponent(holderEl /** Element */, componentName /** string */ ) {
        let self = this
        const CDN = this.cdn
        const BUS = this.bus
        const COMP_URL = `/${this.cdn}/${componentName}`
        fetch(COMP_URL).then(function (response) {
          // The API call was successful!
          return response.text();
        }).then(function (html) {
          // This is the HTML from our response as a text string
          // holderEl.innerHTML = html;
          let holderId = holderEl.id
          $(`#${holderId}`).hide()
          $(`#${holderId}`).html(html)
          $(`#${componentName}`).hide()
          $(`#${holderId}`).show()

           //.fadeIn(150);
          // $(`#${holderId}`).show()

          var compClassName = holderEl.getAttribute("data-class");

          console.log('Loading component classname:', compClassName)

//          try {
//              let compClass = eval(compClassName)
//              console.log('Checked successful the component class already exists:', compClassName)
//              self.instanceBuilder(compClass, componentName)
//          } catch(e) {
            console.log('Component class not found :', compClassName)
            // We need to create the componentClass
            var script = document.createElement("script");
            script.src = `/${CDN}/static/js/${componentName}.js`
            script.addEventListener("load", function() {
                // when the loading is complete
                console.log("Script loading done !");

                let compClass = eval(compClassName)
                self.instanceBuilder(compClass, componentName)
            });
            holderEl.appendChild(script);
//          }

        }).catch(function (err) {
          // There was an error
          console.error(`Cannot load the component: ${COMP_URL}`, err);
        });
    }

    /** private */ instanceBuilder(compClass /** class */, componentName /** string */) {
      let args = [this.bus]
      var instance = new (Function.prototype.bind.apply(compClass, [null, ...args]))();
      instance.init();
      const componentEl = document.querySelector(`#${componentName}`);
      componentEl['instance'] = instance;
    }

    /** Retrieve the object associated to the component
    meaning the instance we stored the instance attribute */
    static findObject(componentName /** string */) /** object */ {
        const componentEl = document.querySelector(`#${componentName}`);
        console.log("Holder instance found: ", componentEl.instance)
        return componentEl.instance
    }

}


class Channel {
    constructor(channelName /** string */) {
        this.channelName = channelName
        this.callbacks /** (data)[] */ = []
    }

    subscribe(callback /** (data) */) {
        this.callbacks.push(callback)
    }

    sendData(data) {
        console.log("Data ready to be sent :", data)
        for (var call of this.callbacks) {
            console.log("Trigger callback")
            call(data)
        }
    }
}

class Bus {
    constructor() {
        this.channels /** {[key:string, <Channel>] */ = {}
    }

    addChannel(channel /** Channel */) {
        this.channels[channel.channelName] = channel
    }

    dropChannel(channel /** Channel */) {
        delete this.channels[channel.channelName]
    }

    getChannel(channelName) {
        return this.channels[channelName]
    }
}



