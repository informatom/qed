defmodule Qed.Podcasts.AudioFile do
  @moduledoc """
  One audio format of an episode (each format gets its own feed).

  The file is either uploaded to qed (`source: :upload`, copied to the target
  on publish) or hosted elsewhere (`source: :url`).
  """
  use Ash.Resource,
    otp_app: :qed,
    domain: Qed.Podcasts,
    data_layer: AshSqlite.DataLayer,
    authorizers: [Ash.Policy.Authorizer]

  @mime_types %{
    mp3: "audio/mpeg",
    m4a: "audio/mp4",
    ogg: "audio/ogg",
    opus: "audio/ogg; codecs=opus"
  }

  sqlite do
    table "audio_files"
    repo Qed.Repo

    references do
      reference :episode, on_delete: :delete
    end
  end

  @editable [:format, :source, :url, :path, :byte_size]

  actions do
    defaults [:read, :destroy]

    create :create do
      primary? true
      accept [:episode_id | @editable]
    end

    update :update do
      # Check policies against the loaded record (clear "forbidden" errors).
      require_atomic? false
      primary? true
      accept @editable
    end
  end

  policies do
    policy action_type(:create) do
      authorize_if {Qed.Podcasts.Checks.PodcastRole, via: :episode_id}
    end

    policy action_type([:read, :update, :destroy]) do
      authorize_if expr(exists(episode.podcast.memberships, user_id == ^actor(:id)))
    end
  end

  validations do
    validate present(:url), where: attribute_equals(:source, :url)
    validate present(:path), where: attribute_equals(:source, :upload)
  end

  attributes do
    uuid_primary_key :id

    attribute :format, :atom do
      allow_nil? false
      public? true
      constraints one_of: Map.keys(@mime_types)
    end

    attribute :source, :atom do
      allow_nil? false
      default :upload
      public? true
      constraints one_of: [:upload, :url]
    end

    attribute :url, :string, public?: true, description: "External URL (source: :url)"

    attribute :path, :string,
      public?: true,
      description: "Storage path of the uploaded file (source: :upload)"

    attribute :byte_size, :integer, public?: true, constraints: [min: 0]

    timestamps()
  end

  relationships do
    belongs_to :episode, Qed.Podcasts.Episode, allow_nil?: false, public?: true
  end

  calculations do
    calculate :mime_type,
              :string,
              fn records, _context -> Enum.map(records, &mime_type(&1.format)) end,
              public?: true
  end

  identities do
    identity :unique_format_per_episode, [:episode_id, :format]
  end

  @doc "MIME type for an audio format."
  def mime_type(format), do: Map.fetch!(@mime_types, format)
end
