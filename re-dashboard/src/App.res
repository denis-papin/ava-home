@val external setInterval: (unit => unit, int) => int = "setInterval"
@val external clearInterval: int => unit = "clearInterval"

let readErrorMessage = () => "Impossible de contacter dashboard-api."

let pendingMatches = (pendingRoomId, roomId) =>
  switch pendingRoomId {
  | Some(currentRoomId) => currentRoomId == roomId
  | None => false
  }

@react.component
let make = () => {
  let apiBaseUrl = DashboardApi.getApiBaseUrl()
  let (rooms, setRooms) = React.useState(() => DashboardApi.createPlaceholderRooms())
  let (_tick, setTick) = React.useState(() => 0)
  let (isLoading, setIsLoading) = React.useState(() => true)
  let (errorMessage, setErrorMessage) = React.useState(() => None)
  let (pendingRoomId, setPendingRoomId) = React.useState(() => None)

  let loadDashboard = () => {
    DashboardApi.fetchDashboardRooms(apiBaseUrl)
    ->Promise.then(nextRooms => {
      setRooms(_previous => nextRooms)
      setErrorMessage(_previous => None)
      setIsLoading(_previous => false)
      Promise.resolve()
    })
    ->Promise.catch(_error => {
      setErrorMessage(_previous => Some(readErrorMessage()))
      setIsLoading(_previous => false)
      Promise.resolve()
    })
    ->ignore
  }

  React.useEffect(() => {
    loadDashboard()

    let intervalId = setInterval(() => {
      setTick(previous => previous + 1)
      loadDashboard()
    }, 30000)

    Some(() => clearInterval(intervalId))
  }, ())

  let spinRoomStatus = (roomId, currentStatus) => {
    setPendingRoomId(_previous => Some(roomId))

    DashboardApi.spinRadiatorStatus(apiBaseUrl, roomId, currentStatus)
    ->Promise.then(nextStatus => {
      setRooms(previousRooms => DashboardApi.replaceRoomStatus(previousRooms, roomId, nextStatus))
      setPendingRoomId(_previous => None)
      setErrorMessage(_previous => None)
      Promise.resolve()
    })
    ->Promise.catch(_error => {
      setPendingRoomId(_previous => None)
      setErrorMessage(_previous => Some(readErrorMessage()))
      Promise.resolve()
    })
    ->ignore
  }

  let roomCards =
    rooms
    ->Array.map(room => {
      let roomId = DashboardApi.id(room)
      let status = DashboardApi.status(room)
      let statusTone = DashboardApi.statusTone(status)
      let displayedElapsed =
        switch DashboardApi.tsCreate(room)->Nullable.toOption {
        | Some(tsCreate) => DashboardApi.formatElapsed(tsCreate)
        | None => DashboardApi.elapsedLabel(room)
        }
      let isPending = pendingMatches(pendingRoomId, roomId)
      let actionLabel =
        isPending ? "Mise a jour..." : "Changer le mode"

      <article key=roomId className="roomCard">
        <div className="roomCard__body">
          <div>
            <p className="roomCard__label">{room->DashboardApi.label->React.string}</p>
            <div className="roomCard__temperatureLine">
              <span className="roomCard__temperature">
                {room->DashboardApi.temperature->React.string}
              </span>
              <span className="roomCard__unit">{"°C"->React.string}</span>
            </div>
            <p className="roomCard__elapsed">
              {"Mise a jour "->React.string}
              <strong>{displayedElapsed->React.string}</strong>
            </p>
          </div>
          <div className="thermoIcon" ariaHidden=true>
            {"°"->React.string}
          </div>
        </div>

        <button
          className="roomCard__footer"
          disabled=isPending
          onClick={_event => spinRoomStatus(roomId, status)}>
          <div>
            <p className="roomCard__targetLabel">{"Temperature cible"->React.string}</p>
            <p className="roomCard__targetValue">
              {room->DashboardApi.targetTemperature->React.string}
              <span className="roomCard__unit roomCard__unit--small">{"°C"->React.string}</span>
            </p>
          </div>
          <div className="roomCard__statusGroup">
            <span className={"statusDot statusDot--" ++ statusTone} />
            <span className={"statusText statusText--" ++ statusTone}>
              {status->DashboardApi.statusLabel->React.string}
            </span>
            <span className="roomCard__action">{actionLabel->React.string}</span>
          </div>
        </button>
      </article>
    })

  <main className="dashboardPage">
    <section className="hero">
      <div>
        <p className="eyebrow">{"re-dashboard"->React.string}</p>
        <h1>{"Dashboard chauffage"->React.string}</h1>
        <p className="lead">
          {"Le frontend React / ReScript consomme maintenant les endpoints exposes par dashboard-api."
          ->React.string}
        </p>
      </div>
      <div className="hero__meta">
        <span className="badge">
          {(isLoading ? "Chargement..." : "Synchro toutes les 30 s")->React.string}
        </span>
        <span className="badge badgeAlt">{apiBaseUrl->React.string}</span>
      </div>
    </section>

    {switch errorMessage {
    | Some(message) =>
      <section className="alert">
        <strong>{"Connexion API"->React.string}</strong>
        <span>{message->React.string}</span>
      </section>
    | None => React.null
    }}

    <section className="roomGrid">{roomCards->React.array}</section>
  </main>
}
