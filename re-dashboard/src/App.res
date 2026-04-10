@val external setInterval: (unit => unit, int) => int = "setInterval"
@val external clearInterval: int => unit = "clearInterval"
external asTimelineForm: {..} => TimelineHelpers.form = "%identity"

let readErrorMessage = () => "Impossible de contacter dashboard-api."

let pendingMatches = (pendingRoomId, roomId) =>
  switch pendingRoomId {
  | Some(currentRoomId) => currentRoomId == roomId
  | None => false
  }

let timelineLegendToneClass = mode =>
  switch mode {
  | "CFT" | "CRF" => "timelineLegend__swatch--heat"
  | "ECO" => "timelineLegend__swatch--eco"
  | "FRO" => "timelineLegend__swatch--frost"
  | "STOP" => "timelineLegend__swatch--stop"
  | _ => "timelineLegend__swatch--neutral"
  }

let defaultTimelineFilters = TimelineHelpers.makeDefaultTimelineFilters()

@react.component
let make = () => {
  let apiBaseUrl = DashboardApi.getApiBaseUrl()
  let (rooms, setRooms) = React.useState(() => DashboardApi.createPlaceholderRooms())
  let (isLoading, setIsLoading) = React.useState(() => true)
  let (errorMessage, setErrorMessage) = React.useState(() => None)
  let (pendingRoomId, setPendingRoomId) = React.useState(() => None)
  let (timelineLoading, setTimelineLoading) = React.useState(() => true)
  let (timelineSubmitPending, setTimelineSubmitPending) = React.useState(() => false)
  let (timelineError, setTimelineError) = React.useState(() => None)
  let (timelineViewModel, setTimelineViewModel) = React.useState(() => None)

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

  let loadTimeline = (~room, ~startDateTime, ~endDateTime, ~showButtonLoading) => {
    setTimelineLoading(_previous => true)
    if showButtonLoading {
      setTimelineSubmitPending(_previous => true)
    }

    TimelineHelpers.fetchRoomTemperatureByMode(apiBaseUrl, room, startDateTime, endDateTime)
    ->Promise.then(response => {
      setTimelineViewModel(_previous => Some(TimelineHelpers.buildTimelineViewModel(response)))
      setTimelineError(_previous => None)
      setTimelineLoading(_previous => false)
      setTimelineSubmitPending(_previous => false)
      Promise.resolve()
    })
    ->Promise.catch(_error => {
      setTimelineError(_previous => Some("Impossible de charger l'historique de temperature."))
      setTimelineLoading(_previous => false)
      setTimelineSubmitPending(_previous => false)
      Promise.resolve()
    })
    ->ignore
  }

  React.useEffect0(() => {
    loadDashboard()
    loadTimeline(
      ~room=TimelineHelpers.room(defaultTimelineFilters),
      ~startDateTime=TimelineHelpers.startDateTime(defaultTimelineFilters),
      ~endDateTime=TimelineHelpers.endDateTime(defaultTimelineFilters),
      ~showButtonLoading=false,
    )

    let intervalId = setInterval(() => {
      loadDashboard()
    }, 30000)

    Some(() => clearInterval(intervalId))
  })

  let applyTimelineFilters = filters => {
    loadTimeline(
      ~room=TimelineHelpers.room(filters),
      ~startDateTime=TimelineHelpers.startDateTime(filters),
      ~endDateTime=TimelineHelpers.endDateTime(filters),
      ~showButtonLoading=true,
    )
  }

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
      let actionLabel = isPending ? "Mise a jour..." : "Changer le mode"

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

  let timelineSection =
    switch timelineViewModel {
    | None =>
      <div className="timelineEmpty">
        {(
          timelineLoading ? "Chargement de l'historique..." : "Aucune donnee de timeline disponible."
        )->React.string}
      </div>
    | Some(viewModel) =>
      <section className="timelineCard">
        <div className="timelineCard__controls">
          <div className="timelineCard__copy">
            <p className="timelineCard__eyebrow">{"ClairDeLune"->React.string}</p>
            <h2>{"Temperature par mode radiateur"->React.string}</h2>
            <p>
              {"Explore l'evolution de la temperature dans une piece, coloree selon le mode actif du radiateur."
              ->React.string}
            </p>
          </div>

          <form
            className="timelineFilters"
            onSubmit={event => {
              event->ReactEvent.Form.preventDefault
              applyTimelineFilters(
                TimelineHelpers.readTimelineFilters(event->ReactEvent.Form.currentTarget->asTimelineForm),
              )
            }}>
            <label className="field">
              <span>{"Piece"->React.string}</span>
              <select
                name="room"
                defaultValue={TimelineHelpers.room(defaultTimelineFilters)}>
                <option value="bureau">{"Bureau"->React.string}</option>
                <option value="chambre">{"Chambre"->React.string}</option>
                <option value="salon">{"Salon"->React.string}</option>
                <option value="couloir">{"Couloir"->React.string}</option>
              </select>
            </label>

            <label className="field">
              <span>{"Debut"->React.string}</span>
              <input
                name="startDateTime"
                type_="datetime-local"
                step=60.
                defaultValue={TimelineHelpers.startDateTime(defaultTimelineFilters)}
              />
            </label>

            <label className="field">
              <span>{"Fin"->React.string}</span>
              <input
                name="endDateTime"
                type_="datetime-local"
                step=60.
                defaultValue={TimelineHelpers.endDateTime(defaultTimelineFilters)}
              />
            </label>

            <button className="primaryButton" type_="submit" disabled=timelineSubmitPending>
              {(
                timelineSubmitPending ? "Chargement..." : "Tracer la timeline"
              )->React.string}
            </button>
          </form>
        </div>

        <div className="timelineCard__panel">
          <div className="timelineMeta">
            <div>
              <h3>{viewModel->TimelineHelpers.title->React.string}</h3>
              <p>{viewModel->TimelineHelpers.subtitle->React.string}</p>
            </div>
            <div className="timelineLegend">
              {viewModel
               ->TimelineHelpers.legend
               ->Array.map(item =>
                 <span key={item->TimelineHelpers.mode} className="timelineLegend__item">
                    <span
                     className={
                       "timelineLegend__swatch "
                       ++ timelineLegendToneClass(item->TimelineHelpers.mode)
                     }
                   />
                   <span>{item->TimelineHelpers.mode->React.string}</span>
                 </span>
               )
               ->React.array}
            </div>
          </div>

          <p className="timelineStatus">
            {switch timelineError {
            | Some(message) => message->React.string
            | None => viewModel->TimelineHelpers.status->React.string
            }}
          </p>

          <div className="timelineHost">
            {switch viewModel->TimelineHelpers.chart->Nullable.toOption {
            | None =>
              <div className="timelineEmpty">
                {switch viewModel->TimelineHelpers.emptyMessage->Nullable.toOption {
                | Some(message) => message->React.string
                | None => "Aucune mesure disponible."->React.string
                }}
              </div>
            | Some(chartModel) =>
              <svg
                className="chartSvg"
                viewBox={chartModel->TimelineHelpers.viewBox}
                role="img"
                ariaLabel="Temperature chart">
                <defs>
                  <linearGradient id="lineGlow" x1="0%" y1="0%" x2="100%" y2="0%">
                    <stop offset="0%" stopColor="#ffd7b0" />
                    <stop offset="100%" stopColor="#d36744" />
                  </linearGradient>
                </defs>

                <rect
                  x="0"
                  y="0"
                  width="1100"
                  height="520"
                  rx="18"
                  fill="rgba(10, 14, 18, 0.18)"
                />

                {chartModel
                 ->TimelineHelpers.rects
                 ->Array.mapWithIndex((item, index) =>
                   <rect
                     key={"rect-" ++ Int.toString(index)}
                     x={item->TimelineHelpers.rectX->Float.toString}
                     y={item->TimelineHelpers.rectY->Float.toString}
                     width={item->TimelineHelpers.rectWidth->Float.toString}
                     height={item->TimelineHelpers.rectHeight->Float.toString}
                     fill={item->TimelineHelpers.fill}
                   />
                 )
                 ->React.array}

                {chartModel
                 ->TimelineHelpers.yGuides
                 ->Array.mapWithIndex((guide, index) =>
                   <React.Fragment key={"yg-" ++ Int.toString(index)}>
                     <line
                       x1={guide->TimelineHelpers.lineX1->Float.toString}
                       y1={guide->TimelineHelpers.lineY1->Float.toString}
                       x2={guide->TimelineHelpers.lineX2->Float.toString}
                       y2={guide->TimelineHelpers.lineY2->Float.toString}
                       stroke="rgba(245, 237, 226, 0.14)"
                       strokeWidth="1"
                     />
                     <text
                       x={guide->TimelineHelpers.labelX->Float.toString}
                       y={guide->TimelineHelpers.labelY->Float.toString}
                       className="gridLabel"
                       textAnchor={guide->TimelineHelpers.textAnchor}>
                       {guide->TimelineHelpers.labelText->React.string}
                     </text>
                   </React.Fragment>
                 )
                 ->React.array}

                {chartModel
                 ->TimelineHelpers.xGuides
                 ->Array.mapWithIndex((guide, index) =>
                   <React.Fragment key={"xg-" ++ Int.toString(index)}>
                     <line
                       x1={guide->TimelineHelpers.lineX1->Float.toString}
                       y1={guide->TimelineHelpers.lineY1->Float.toString}
                       x2={guide->TimelineHelpers.lineX2->Float.toString}
                       y2={guide->TimelineHelpers.lineY2->Float.toString}
                       stroke="rgba(245, 237, 226, 0.14)"
                       strokeWidth="1"
                     />
                     <text
                       x={guide->TimelineHelpers.labelX->Float.toString}
                       y={guide->TimelineHelpers.labelY->Float.toString}
                       className="tickLabel"
                       textAnchor={guide->TimelineHelpers.textAnchor}>
                       {guide->TimelineHelpers.labelText->React.string}
                     </text>
                   </React.Fragment>
                 )
                 ->React.array}

                <line
                  x1={chartModel->TimelineHelpers.axisLeftX->Float.toString}
                  y1={chartModel->TimelineHelpers.axisBottomY->Float.toString}
                  x2={chartModel->TimelineHelpers.axisRightX->Float.toString}
                  y2={chartModel->TimelineHelpers.axisBottomY->Float.toString}
                  stroke="rgba(247, 241, 232, 0.36)"
                />
                <line
                  x1={chartModel->TimelineHelpers.axisLeftX->Float.toString}
                  y1={chartModel->TimelineHelpers.axisTopY->Float.toString}
                  x2={chartModel->TimelineHelpers.axisLeftX->Float.toString}
                  y2={chartModel->TimelineHelpers.axisBottomY->Float.toString}
                  stroke="rgba(247, 241, 232, 0.36)"
                />

                <path
                  d={chartModel->TimelineHelpers.linePath}
                  fill="none"
                  stroke="url(#lineGlow)"
                  strokeWidth="4"
                  strokeLinejoin="round"
                  strokeLinecap="round"
                />

                {chartModel
                 ->TimelineHelpers.points
                 ->Array.mapWithIndex((point, index) =>
                   <circle
                     key={"pt-" ++ Int.toString(index)}
                     cx={point->TimelineHelpers.cx->Float.toString}
                     cy={point->TimelineHelpers.cy->Float.toString}
                     r="4"
                     fill="#fff6ec"
                     stroke="#d36744"
                     strokeWidth="2"
                   />
                 )
                 ->React.array}

                <text
                  x={chartModel->TimelineHelpers.axisLabelXPosX->Float.toString}
                  y={chartModel->TimelineHelpers.axisLabelXPosY->Float.toString}
                  className="axisLabel"
                  textAnchor="middle">
                  {chartModel->TimelineHelpers.axisLabelX->React.string}
                </text>
                <text
                  x={chartModel->TimelineHelpers.axisLabelYPosX->Float.toString}
                  y={chartModel->TimelineHelpers.axisLabelYPosY->Float.toString}
                  className="axisLabel"
                  transform={chartModel->TimelineHelpers.axisLabelYRotate}
                  textAnchor="middle">
                  {chartModel->TimelineHelpers.axisLabelY->React.string}
                </text>
              </svg>
            }}
          </div>

          <div className="timelineStats">
            {viewModel
             ->TimelineHelpers.stats
             ->Array.map(item =>
               <div key={item->TimelineHelpers.statLabel} className="timelineStatCard">
                 <span>{item->TimelineHelpers.statLabel->React.string}</span>
                 <strong>{item->TimelineHelpers.statValue->React.string}</strong>
               </div>
             )
             ->React.array}
          </div>
        </div>
      </section>
    }

  <main className="dashboardPage">
    <section className="hero">
      <div>
        <p className="eyebrow">{"re-dashboard"->React.string}</p>
        <h1>{"Dashboard chauffage"->React.string}</h1>
        <p className="lead">
          {"Le dashboard principal conserve la vue temps reel et integre maintenant l'ecran d'analyse type ClairDeLune."
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
    {timelineSection}
  </main>
}
