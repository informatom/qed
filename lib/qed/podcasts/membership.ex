defmodule Qed.Podcasts.Membership do
  @moduledoc """
  Connects a user to a podcast with a role:

    * `:owner`  - everything, including members, publish targets and deletion
    * `:editor` - edit the podcast and its episodes, publish
  """
  use Ash.Resource,
    otp_app: :qed,
    domain: Qed.Podcasts,
    data_layer: AshSqlite.DataLayer,
    authorizers: [Ash.Policy.Authorizer]

  sqlite do
    table "memberships"
    repo Qed.Repo

    references do
      reference :podcast, on_delete: :delete
      reference :user, on_delete: :delete
    end
  end

  actions do
    defaults [:read, :destroy]

    create :create do
      primary? true
      accept [:podcast_id, :user_id, :role]
    end

    update :change_role do
      # Check policies against the loaded record (clear "forbidden" errors).
      require_atomic? false
      accept [:role]
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

  attributes do
    uuid_primary_key :id

    attribute :role, :atom do
      allow_nil? false
      default :editor
      public? true
      constraints one_of: [:owner, :editor]
    end

    timestamps()
  end

  relationships do
    belongs_to :podcast, Qed.Podcasts.Podcast, allow_nil?: false, public?: true
    belongs_to :user, Qed.Accounts.User, allow_nil?: false, public?: true
  end

  identities do
    identity :unique_member, [:podcast_id, :user_id]
  end
end
