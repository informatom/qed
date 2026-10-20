defmodule Qed.Secrets do
  use AshAuthentication.Secret

  def secret_for([:authentication, :tokens, :signing_secret], Qed.Accounts.User, _opts, _context) do
    Application.fetch_env(:qed, :token_signing_secret)
  end
end
