# re-dashboard

Projet frontend autonome en React + ReScript pour consommer `dashboard-api`.

## Demarrage

```bash
npm install
npm run dev
```

Le serveur de developpement tourne sur `http://localhost:2091`.

Par defaut, le frontend appelle `http://127.0.0.1:2090/dashboard-api`.

## Configuration API

Vous pouvez surcharger l'origine de `dashboard-api` avec une variable Vite:

```bash
VITE_DASHBOARD_API_ORIGIN=http://127.0.0.1:2090 npm run dev
```

## Fonctionnalites

- affichage des 4 pieces: salon, bureau, chambre, couloir
- chargement des temperatures et statuts via `GET /dashboard-api/index/data`
- rotation du mode radiateur via `POST /dashboard-api/index/radiator/:room`
- rafraichissement automatique toutes les 30 secondes

## Scripts

- `npm run dev` lance ReScript en watch et Vite sur le port `2091`
- `npm run build` compile ReScript puis construit le bundle de production
- `npm run preview` sert le build sur le port `2091`
