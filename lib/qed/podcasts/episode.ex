defmodule Qed.Podcasts.Episode do
  @moduledoc "An episode of a podcast. Shownotes are Markdown."
  use Ash.Resource,
    otp_app: :qed,
    domain: Qed.Podcasts,
    data_layer: AshSqlite.DataLayer,
    authorizers: [Ash.Policy.Authorizer]

  sqlite do
    table "episodes"
    repo Qed.Repo

    references do
      reference :podcast, on_delete: :delete
    end
  end

  @editable [
    :title,
    :slug,
    :subtitle,
    :number,
    :published_at,
    :author,
    :explicit,
    :duration,
    :shownotes,
    :chapters
  ]

  actions do
    defaults [:read, :destroy]

    create :create do
      primary? true
      accept [:podcast_id | @editable]
    end

    update :update do
      # Check policies against the loaded record (clear "forbidden" errors).
      require_atomic? false
      primary? true
      accept @editable
    end

    update :publish do
      description "Marks the episode as published; sets the date to now unless one is set."
      accept []
      require_atomic? false

      change set_attribute(:state, :published)

      change fn changeset, _context ->
        if is_nil(changeset.data.published_at) do
          now = DateTime.utc_now() |> DateTime.truncate(:second)
          Ash.Changeset.force_change_attribute(changeset, :published_at, now)
        else
          changeset
        end
      end
    end

    update :unpublish do
      # Check policies against the loaded record (clear "forbidden" errors).
      require_atomic? false
      accept []
      change set_attribute(:state, :draft)
    end
  end

  policies do
    policy action_type(:create) do
      authorize_if Qed.Podcasts.Checks.PodcastRole
    end

    policy action_type([:read, :update, :destroy]) do
      authorize_if expr(exists(podcast.memberships, user_id == ^actor(:id)))
    end
  end

  attributes do
    uuid_primary_key :id

    attribute :title, :string, allow_nil?: false, public?: true

    attribute :slug, :string do
      allow_nil? false
      public? true
      constraints match: ~r/^[a-z0-9]+(?:-[a-z0-9]+)*$/
    end

    attribute :subtitle, :string, public?: true
    attribute :number, :integer, public?: true, constraints: [min: 0]

    attribute :state, :atom do
      allow_nil? false
      default :draft
      public? true
      constraints one_of: [:draft, :published]
    end

    attribute :published_at, :utc_datetime, public?: true
    attribute :author, :string, public?: true
    attribute :explicit, :boolean, allow_nil?: false, default: false, public?: true

    attribute :duration, :integer do
      public? true
      constraints min: 0
      description "Length in seconds"
    end

    attribute :shownotes, :string do
      public? true
      description "Markdown"
      constraints trim?: false, allow_empty?: true
    end

    attribute :chapters, {:array, Qed.Podcasts.Chapter},
      allow_nil?: false,
      default: [],
      public?: true

    timestamps()
  end

  relationships do
    belongs_to :podcast, Qed.Podcasts.Podcast, allow_nil?: false, public?: true
    has_many :audio_files, Qed.Podcasts.AudioFile, public?: true
  end

  identities do
    identity :unique_slug_per_podcast, [:podcast_id, :slug]
  end
end
