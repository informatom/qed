# qed

An open-source static site generator for podcasts that is itself a web app,
built with Elixir, Phoenix, Ash, Tailwind and daisyUI. It runs on a server
(yours, via Docker Compose, or a hosted demo) and can be installed as a PWA.

## Run with Docker Compose

```sh
cp .env.example .env   # then set SECRET_KEY_BASE, TOKEN_SIGNING_SECRET and CLOAK_KEY
docker compose up -d --build
```

Open http://localhost:4000. Data (SQLite) lives in the `qed_data` volume.

## Run from source

Requires the Erlang/Elixir versions in `.tool-versions`.

```sh
mix setup
mix phx.server
```

Open http://localhost:4000.
