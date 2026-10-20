defmodule Qed.Vault do
  @moduledoc "Encrypts secrets at rest (SSH private keys, API tokens)."
  use Cloak.Vault, otp_app: :qed
end
