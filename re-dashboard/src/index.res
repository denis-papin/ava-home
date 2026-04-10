@val external document: Dom.document = "document"
@send external getElementById: (Dom.document, string) => Dom.element = "getElementById"

let rootElement = document->getElementById("root")

ReactDOM.Client.createRoot(rootElement)->ReactDOM.Client.Root.render(<App />)
