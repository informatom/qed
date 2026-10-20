defmodule Qed.Accounts do
  use Ash.Domain,
    otp_app: :qed

  resources do
    resource Qed.Accounts.Token
    resource Qed.Accounts.User
  end
end
