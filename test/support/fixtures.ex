defmodule Qed.Fixtures do
  @moduledoc "Test data helpers."

  alias Qed.Podcasts

  def unique, do: System.unique_integer([:positive])

  def user_fixture do
    Ash.Seed.seed!(Qed.Accounts.User, %{
      email: "user-#{unique()}@example.com",
      hashed_password: "not-a-real-hash"
    })
  end

  def podcast_fixture(owner, attrs \\ %{}) do
    n = unique()

    %{title: "Podcast #{n}", slug: "podcast-#{n}"}
    |> Map.merge(attrs)
    |> Podcasts.create_podcast!(actor: owner)
  end

  def episode_fixture(podcast, actor, attrs \\ %{}) do
    n = unique()

    %{podcast_id: podcast.id, title: "Episode #{n}", slug: "episode-#{n}"}
    |> Map.merge(attrs)
    |> Podcasts.create_episode!(actor: actor)
  end

  @doc "Creates a podcast with an owner and an editor."
  def team_fixture do
    owner = user_fixture()
    editor = user_fixture()
    podcast = podcast_fixture(owner)
    Podcasts.add_member!(podcast.id, editor.id, :editor, actor: owner)
    %{owner: owner, editor: editor, outsider: user_fixture(), podcast: podcast}
  end
end
