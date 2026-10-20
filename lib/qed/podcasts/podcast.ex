defmodule Qed.Podcasts.Podcast do
  @moduledoc "A podcast: channel-level metadata used for the site and its feeds."
  use Ash.Resource,
    otp_app: :qed,
    domain: Qed.Podcasts,
    data_layer: AshSqlite.DataLayer,
    authorizers: [Ash.Policy.Authorizer]

  sqlite do
    table "podcasts"
    repo Qed.Repo
  end

  @editable [
    :title,
    :slug,
    :subtitle,
    :description,
    :author,
    :email,
    :language,
    :keywords,
    :categories,
    :explicit,
    :site_url,
    :license,
    :license_url,
    :fediverse_url,
    :apple_podcasts_url,
    :episodes_per_feed_page
  ]

  actions do
    defaults [:read, :destroy]

    create :create do
      primary? true
      accept @editable
      change Qed.Podcasts.Changes.AddCreatorAsOwner
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
      authorize_if actor_present()
    end

    policy action_type(:read) do
      authorize_if expr(exists(memberships, user_id == ^actor(:id)))
    end

    policy action_type(:update) do
      authorize_if expr(exists(memberships, user_id == ^actor(:id) and role in [:owner, :editor]))
    end

    policy action_type(:destroy) do
      authorize_if expr(exists(memberships, user_id == ^actor(:id) and role == :owner))
    end
  end

  attributes do
    uuid_primary_key :id

    attribute :title, :string, allow_nil?: false, public?: true

    attribute :slug, :string do
      allow_nil? false
      public? true
      constraints match: ~r/^[a-z0-9]+(?:-[a-z0-9]+)*$/
      description "URL-safe identifier, e.g. `my-podcast`"
    end

    attribute :subtitle, :string, public?: true
    attribute :description, :string, public?: true
    attribute :author, :string, public?: true
    attribute :email, :string, public?: true
    attribute :language, :string, allow_nil?: false, default: "en", public?: true
    attribute :keywords, {:array, :string}, allow_nil?: false, default: [], public?: true

    attribute :categories, {:array, :string} do
      allow_nil? false
      default []
      public? true
      description "Apple Podcasts categories, e.g. `Technology`"
    end

    attribute :explicit, :boolean, allow_nil?: false, default: false, public?: true

    attribute :site_url, :string do
      public? true
      description "Public URL the generated site is published at"
    end

    attribute :license, :string, public?: true
    attribute :license_url, :string, public?: true
    attribute :fediverse_url, :string, public?: true
    attribute :apple_podcasts_url, :string, public?: true

    attribute :episodes_per_feed_page, :integer do
      allow_nil? false
      default 100
      public? true
      constraints min: 1
    end

    timestamps()
  end

  relationships do
    has_many :memberships, Qed.Podcasts.Membership, public?: true
    has_many :episodes, Qed.Podcasts.Episode, public?: true
    has_many :publish_targets, Qed.Podcasts.PublishTarget, public?: true

    many_to_many :members, Qed.Accounts.User do
      through Qed.Podcasts.Membership
      source_attribute_on_join_resource :podcast_id
      destination_attribute_on_join_resource :user_id
      public? true
    end
  end

  identities do
    identity :unique_slug, [:slug]
  end
end
