defmodule Qed.Podcasts.PodcastTest do
  use Qed.DataCase, async: false

  import Qed.Fixtures

  alias Qed.Podcasts

  describe "create" do
    test "makes the creator the owner" do
      user = user_fixture()

      podcast =
        Podcasts.create_podcast!(%{title: "Tech Talk", slug: "tech-talk"}, actor: user)

      assert [membership] = Podcasts.list_members!(actor: user)
      assert membership.podcast_id == podcast.id
      assert membership.user_id == user.id
      assert membership.role == :owner
    end

    test "uses sensible defaults" do
      podcast = podcast_fixture(user_fixture())

      assert podcast.language == "en"
      assert podcast.explicit == false
      assert podcast.keywords == []
      assert podcast.episodes_per_feed_page == 100
    end

    test "stores keywords and categories as lists" do
      user = user_fixture()
      podcast = podcast_fixture(user, %{keywords: ["elixir", "web"], categories: ["Technology"]})

      reloaded = Podcasts.get_podcast!(podcast.id, actor: user)
      assert reloaded.keywords == ["elixir", "web"]
      assert reloaded.categories == ["Technology"]
    end

    test "requires an actor" do
      assert_raise Ash.Error.Forbidden, fn ->
        Podcasts.create_podcast!(%{title: "x", slug: "x"})
      end
    end

    test "rejects slugs that are not URL-safe" do
      user = user_fixture()

      for slug <- ["Has Spaces", "UPPER", "trailing-", "ümlaut"] do
        assert {:error, %Ash.Error.Invalid{}} =
                 Podcasts.create_podcast(%{title: "x", slug: slug}, actor: user)
      end
    end

    test "slugs are unique" do
      user = user_fixture()
      podcast_fixture(user, %{slug: "taken"})

      assert {:error, %Ash.Error.Invalid{}} =
               Podcasts.create_podcast(%{title: "x", slug: "taken"}, actor: user)
    end
  end

  describe "authorization" do
    setup do
      team_fixture()
    end

    test "members see the podcast, outsiders don't", %{podcast: podcast} = team do
      assert [%{id: id}] = Podcasts.list_podcasts!(actor: team.owner)
      assert id == podcast.id
      assert [%{id: ^id}] = Podcasts.list_podcasts!(actor: team.editor)
      assert [] = Podcasts.list_podcasts!(actor: team.outsider)
    end

    test "editors can update, outsiders can't", %{podcast: podcast} = team do
      assert %{title: "New"} =
               Podcasts.update_podcast!(podcast, %{title: "New"}, actor: team.editor)

      assert {:error, %Ash.Error.Forbidden{}} =
               Podcasts.update_podcast(podcast, %{title: "Nope"}, actor: team.outsider)
    end

    test "only owners can delete", %{podcast: podcast} = team do
      assert {:error, %Ash.Error.Forbidden{}} =
               Podcasts.destroy_podcast(podcast, actor: team.editor)

      assert :ok = Podcasts.destroy_podcast!(podcast, actor: team.owner)
      assert [] = Podcasts.list_podcasts!(actor: team.owner)
    end
  end
end
