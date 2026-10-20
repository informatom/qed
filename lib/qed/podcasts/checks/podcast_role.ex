defmodule Qed.Podcasts.Checks.PodcastRole do
  @moduledoc """
  Authorizes create actions on podcast children: the actor must be a member of
  the podcast with one of the given roles.

  The podcast is found via an attribute on the changeset: `podcast_id` directly
  or, with `via: :episode_id`, through the episode.

      authorize_if {Qed.Podcasts.Checks.PodcastRole, roles: [:owner]}
  """
  use Ash.Policy.SimpleCheck

  require Ash.Query

  alias Qed.Podcasts.{Episode, Membership}

  @impl true
  def describe(opts) do
    "actor is #{Enum.join(opts[:roles] || [:owner, :editor], " or ")} of the podcast"
  end

  @impl true
  def match?(nil, _context, _opts), do: false

  def match?(actor, %{subject: %Ash.Changeset{} = changeset}, opts) do
    roles = opts[:roles] || [:owner, :editor]

    case podcast_id(changeset, opts[:via] || :podcast_id) do
      nil ->
        false

      podcast_id ->
        Membership
        |> Ash.Query.filter(podcast_id == ^podcast_id and user_id == ^actor.id and role in ^roles)
        |> Ash.exists?(authorize?: false)
    end
  end

  def match?(_actor, _context, _opts), do: false

  defp podcast_id(changeset, :podcast_id), do: Ash.Changeset.get_attribute(changeset, :podcast_id)

  defp podcast_id(changeset, :episode_id) do
    with episode_id when not is_nil(episode_id) <-
           Ash.Changeset.get_attribute(changeset, :episode_id),
         {:ok, episode} <- Ash.get(Episode, episode_id, authorize?: false) do
      episode.podcast_id
    else
      _ -> nil
    end
  end
end
