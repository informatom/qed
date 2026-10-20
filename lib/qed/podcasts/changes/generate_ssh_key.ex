defmodule Qed.Podcasts.Changes.GenerateSshKey do
  @moduledoc """
  Generates an Ed25519 key pair for SSH publish targets. The private key is
  stored encrypted; the user adds the public key to the server's
  `~/.ssh/authorized_keys`.

  With `force?: true` a new key is always generated (key rotation), otherwise
  only when the target is SSH and has no key yet.
  """
  use Ash.Resource.Change

  @impl true
  def change(changeset, opts, _context) do
    kind = Ash.Changeset.get_attribute(changeset, :kind)
    has_key? = not is_nil(Ash.Changeset.get_attribute(changeset, :ssh_public_key))

    if kind == :ssh and (opts[:force?] || not has_key?) do
      {private_key, public_key} = Qed.SshKey.generate("qed")

      changeset
      |> Ash.Changeset.force_change_attribute(:ssh_public_key, public_key)
      |> AshCloak.encrypt_and_set(:ssh_private_key, private_key)
    else
      changeset
    end
  end
end
