defmodule Qed.Podcasts.EpisodeTest do
  use Qed.DataCase, async: false

  import Qed.Fixtures

  alias Qed.Podcasts

  setup do
    team_fixture()
  end

  test "members create episodes as drafts", %{podcast: podcast, editor: editor} do
    episode = episode_fixture(podcast, editor, %{shownotes: "## Links\n\n* one\n"})

    assert episode.state == :draft
    assert episode.published_at == nil
    assert episode.shownotes == "## Links\n\n* one\n"
  end

  test "outsiders can't create episodes", %{podcast: podcast, outsider: outsider} do
    assert {:error, %Ash.Error.Forbidden{}} =
             Podcasts.create_episode(
               %{podcast_id: podcast.id, title: "x", slug: "x"},
               actor: outsider
             )
  end

  test "outsiders can't see episodes", %{podcast: podcast, owner: owner, outsider: outsider} do
    episode_fixture(podcast, owner)
    assert [_] = Podcasts.list_episodes!(actor: owner)
    assert [] = Podcasts.list_episodes!(actor: outsider)
  end

  test "slugs are unique per podcast", %{podcast: podcast, owner: owner} do
    episode_fixture(podcast, owner, %{slug: "ep1"})

    assert {:error, %Ash.Error.Invalid{}} =
             Podcasts.create_episode(%{podcast_id: podcast.id, title: "x", slug: "ep1"},
               actor: owner
             )

    other = podcast_fixture(owner)
    assert %{slug: "ep1"} = episode_fixture(other, owner, %{slug: "ep1"})
  end

  test "stores chapters", %{podcast: podcast, owner: owner} do
    chapters = [
      %{start: "00:00:00", title: "Intro"},
      %{start: "00:01:30.500", title: "Main topic", href: "https://example.com"}
    ]

    episode = episode_fixture(podcast, owner, %{chapters: chapters})
    reloaded = Podcasts.get_episode!(episode.id, actor: owner)

    assert [%{start: "00:00:00", title: "Intro"}, %{start: "00:01:30.500"}] = reloaded.chapters
  end

  test "rejects malformed chapter times", %{podcast: podcast, owner: owner} do
    assert {:error, %Ash.Error.Invalid{}} =
             Podcasts.create_episode(
               %{
                 podcast_id: podcast.id,
                 title: "x",
                 slug: "x",
                 chapters: [%{start: "1:30", title: "Bad"}]
               },
               actor: owner
             )
  end

  describe "publishing" do
    test "publish sets state and date", %{podcast: podcast, editor: editor} do
      episode = episode_fixture(podcast, editor)
      published = Podcasts.publish_episode!(episode, actor: editor)

      assert published.state == :published
      assert DateTime.diff(DateTime.utc_now(), published.published_at) < 5
    end

    test "publish keeps a date that was set", %{podcast: podcast, editor: editor} do
      date = ~U[2026-01-01 10:00:00Z]
      episode = episode_fixture(podcast, editor, %{published_at: date})

      assert %{published_at: ^date} = Podcasts.publish_episode!(episode, actor: editor)
    end

    test "unpublish goes back to draft", %{podcast: podcast, editor: editor} do
      episode = podcast |> episode_fixture(editor) |> Podcasts.publish_episode!(actor: editor)
      assert %{state: :draft} = Podcasts.unpublish_episode!(episode, actor: editor)
    end
  end

  test "deleting a podcast deletes its episodes", %{podcast: podcast, owner: owner} do
    episode_fixture(podcast, owner)
    Podcasts.destroy_podcast!(podcast, actor: owner)
    assert [] = Ash.read!(Qed.Podcasts.Episode, authorize?: false)
  end
end
