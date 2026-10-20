import Config
config :qed, token_signing_secret: "T1C2v1ywydSbptzJuresDd0HcHUE2gdm"
config :bcrypt_elixir, log_rounds: 1
config :ash, policies: [show_policy_breakdowns?: true], disable_async?: true

# Configure your database
#
# The MIX_TEST_PARTITION environment variable can be used
# to provide built-in test partitioning in CI environment.
# Run `mix help test` for more information.
config :qed, Qed.Repo,
  database: Path.expand("../qed_test.db", __DIR__),
  pool_size: 5,
  pool: Ecto.Adapters.SQL.Sandbox

# We don't run a server during test. If one is required,
# you can enable the server option below.
config :qed, QedWeb.Endpoint,
  http: [ip: {127, 0, 0, 1}, port: 4002],
  secret_key_base: "Kb28iy79vtGyg/phKTI8EPwgBjHNR5VtaRzUqUPMlMo8XeBgUQM0d5wMh/kgBx/o",
  server: false

# In test we don't send emails
config :qed, Qed.Mailer, adapter: Swoosh.Adapters.Test

# Disable swoosh api client as it is only required for production adapters
config :swoosh, :api_client, false

# Print only warnings and errors during test
config :logger, level: :warning

# Initialize plugs at runtime for faster test compilation
config :phoenix, :plug_init_mode, :runtime

# Enable helpful, but potentially expensive runtime checks
config :phoenix_live_view,
  enable_expensive_runtime_checks: true

# Sort query params output of verified routes for robust url comparisons
config :phoenix,
  sort_verified_routes_query_params: true

# Encryption key for secrets at rest (test only; prod reads CLOAK_KEY)
config :qed, Qed.Vault,
  ciphers: [
    default:
      {Cloak.Ciphers.AES.GCM,
       tag: "AES.GCM.V1",
       key: Base.decode64!("uqgW3+QRbnTYw0F78G8RgE3CTauPKR8ala0ECIwcxxA="),
       iv_length: 12}
  ]
