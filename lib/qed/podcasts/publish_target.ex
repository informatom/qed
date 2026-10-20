defmodule Qed.Podcasts.PublishTarget do
  @moduledoc """
  Where a podcast's generated site goes:

    * `:ssh`          - rsync over SSH to your own VM or webspace. qed generates
                        a key pair; add the public key to the server's
                        `~/.ssh/authorized_keys`.
    * `:zip`          - a ZIP archive to download.
    * `:github_pages` - push to a branch of a GitHub repository.

  Secrets (SSH private key, GitHub token) are stored encrypted and never loaded
  unless explicitly requested.
  """
  use Ash.Resource,
    otp_app: :qed,
    domain: Qed.Podcasts,
    data_layer: AshSqlite.DataLayer,
    authorizers: [Ash.Policy.Authorizer],
    extensions: [AshCloak]

  sqlite do
    table "publish_targets"
    repo Qed.Repo

    references do
      reference :podcast, on_delete: :delete
    end
  end

  cloak do
    vault(Qed.Vault)
    attributes([:ssh_private_key, :github_token])
    encrypt_nil?(false)
  end

  @editable [
    :name,
    :ssh_host,
    :ssh_port,
    :ssh_user,
    :ssh_path,
    :rsync_delete,
    :github_repo,
    :github_branch,
    :github_token
  ]

  actions do
    defaults [:read, :destroy]

    create :create do
      primary? true
      accept [:podcast_id, :kind | @editable]
      change Qed.Podcasts.Changes.GenerateSshKey
    end

    update :update do
      primary? true
      require_atomic? false
      accept @editable
    end

    update :regenerate_ssh_key do
      description "Replaces the SSH key pair; the new public key must be added to the server."
      accept []
      require_atomic? false
      validate attribute_equals(:kind, :ssh)
      change {Qed.Podcasts.Changes.GenerateSshKey, force?: true}
    end
  end

  policies do
    policy action_type(:read) do
      authorize_if expr(exists(podcast.memberships, user_id == ^actor(:id)))
    end

    policy action_type(:create) do
      authorize_if {Qed.Podcasts.Checks.PodcastRole, roles: [:owner]}
    end

    policy action_type([:update, :destroy]) do
      authorize_if expr(exists(podcast.memberships, user_id == ^actor(:id) and role == :owner))
    end
  end

  validations do
    validate present([:ssh_host, :ssh_user, :ssh_path]),
      where: attribute_equals(:kind, :ssh)

    validate present([:github_repo, :github_branch]),
      where: attribute_equals(:kind, :github_pages)

    validate match(:github_repo, ~r/^[\w.-]+\/[\w.-]+$/) do
      where [attribute_equals(:kind, :github_pages), present(:github_repo)]
      message "must look like owner/repository"
    end
  end

  attributes do
    uuid_primary_key :id

    attribute :name, :string, allow_nil?: false, public?: true

    attribute :kind, :atom do
      allow_nil? false
      public? true
      constraints one_of: [:ssh, :zip, :github_pages]
    end

    # SSH
    attribute :ssh_host, :string, public?: true
    attribute :ssh_port, :integer, default: 22, public?: true, constraints: [min: 1, max: 65535]
    attribute :ssh_user, :string, public?: true
    attribute :ssh_path, :string, public?: true, description: "Document root on the server"

    attribute :rsync_delete, :boolean,
      allow_nil?: false,
      default: false,
      public?: true,
      description: "Delete files on the server that are not part of the site"

    attribute :ssh_public_key, :string, public?: true
    attribute :ssh_private_key, :string, sensitive?: true

    # GitHub Pages
    attribute :github_repo, :string, public?: true, description: "owner/repository"
    attribute :github_branch, :string, default: "gh-pages", public?: true
    attribute :github_token, :string, sensitive?: true

    timestamps()
  end

  relationships do
    belongs_to :podcast, Qed.Podcasts.Podcast, allow_nil?: false, public?: true
  end
end
