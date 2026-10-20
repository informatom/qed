defmodule Qed.Podcasts.PublishTargetTest do
  use Qed.DataCase, async: false

  import Qed.Fixtures

  alias Qed.Podcasts

  setup do
    team_fixture()
  end

  defp ssh_attrs(podcast) do
    %{
      podcast_id: podcast.id,
      name: "My webspace",
      kind: :ssh,
      ssh_host: "example.com",
      ssh_user: "podcast",
      ssh_path: "/var/www/podcast"
    }
  end

  describe "ssh" do
    test "generates a key pair on create", %{podcast: podcast, owner: owner} do
      target = Podcasts.create_publish_target!(ssh_attrs(podcast), actor: owner)

      assert target.ssh_port == 22
      assert String.starts_with?(target.ssh_public_key, "ssh-ed25519 ")

      target = Ash.load!(target, :ssh_private_key, actor: owner)
      assert {:ok, public} = Qed.SshKey.public_key(target.ssh_private_key)

      assert String.replace_suffix(public, " qed", "") ==
               String.replace_suffix(target.ssh_public_key, " qed", "")
    end

    test "stores the private key encrypted", %{podcast: podcast, owner: owner} do
      target = Podcasts.create_publish_target!(ssh_attrs(podcast), actor: owner)

      %{rows: [[raw]]} =
        Repo.query!("SELECT encrypted_ssh_private_key FROM publish_targets WHERE id = ?", [
          target.id
        ])

      assert is_binary(raw)
      refute raw =~ "OPENSSH"
    end

    test "does not load the private key by default", %{podcast: podcast, owner: owner} do
      Podcasts.create_publish_target!(ssh_attrs(podcast), actor: owner)
      [target] = Podcasts.list_publish_targets!(actor: owner)
      assert %Ash.NotLoaded{} = target.ssh_private_key
    end

    test "requires host, user and path", %{podcast: podcast, owner: owner} do
      attrs = Map.drop(ssh_attrs(podcast), [:ssh_host, :ssh_path])

      assert {:error, %Ash.Error.Invalid{} = error} =
               Podcasts.create_publish_target(attrs, actor: owner)

      fields =
        Enum.flat_map(error.errors, &List.wrap(Map.get(&1, :fields) || Map.get(&1, :field)))

      assert :ssh_host in fields
      assert :ssh_path in fields
    end

    test "regenerating replaces the key", %{podcast: podcast, owner: owner} do
      target = Podcasts.create_publish_target!(ssh_attrs(podcast), actor: owner)
      updated = Podcasts.regenerate_ssh_key!(target, actor: owner)

      refute updated.ssh_public_key == target.ssh_public_key
    end

    test "an update keeps the key", %{podcast: podcast, owner: owner} do
      target = Podcasts.create_publish_target!(ssh_attrs(podcast), actor: owner)
      updated = Podcasts.update_publish_target!(target, %{ssh_port: 2222}, actor: owner)

      assert updated.ssh_port == 2222
      assert updated.ssh_public_key == target.ssh_public_key
    end
  end

  test "zip needs no settings and gets no key", %{podcast: podcast, owner: owner} do
    target =
      Podcasts.create_publish_target!(%{podcast_id: podcast.id, name: "Download", kind: :zip},
        actor: owner
      )

    assert target.ssh_public_key == nil

    assert {:error, %Ash.Error.Invalid{}} =
             Podcasts.regenerate_ssh_key(target, actor: owner)
  end

  describe "github pages" do
    test "stores the token encrypted", %{podcast: podcast, owner: owner} do
      target =
        Podcasts.create_publish_target!(
          %{
            podcast_id: podcast.id,
            name: "GitHub",
            kind: :github_pages,
            github_repo: "me/my-podcast",
            github_token: "ghp_secret"
          },
          actor: owner
        )

      assert target.github_branch == "gh-pages"
      assert %{github_token: "ghp_secret"} = Ash.load!(target, :github_token, actor: owner)
    end

    test "validates the repository name", %{podcast: podcast, owner: owner} do
      assert {:error, %Ash.Error.Invalid{}} =
               Podcasts.create_publish_target(
                 %{
                   podcast_id: podcast.id,
                   name: "GitHub",
                   kind: :github_pages,
                   github_repo: "nope"
                 },
                 actor: owner
               )
    end
  end

  describe "authorization" do
    test "only owners create targets", %{podcast: podcast} = team do
      for actor <- [team.editor, team.outsider] do
        assert {:error, %Ash.Error.Forbidden{}} =
                 Podcasts.create_publish_target(ssh_attrs(podcast), actor: actor)
      end
    end

    test "editors see targets but can't change them", %{podcast: podcast} = team do
      target = Podcasts.create_publish_target!(ssh_attrs(podcast), actor: team.owner)

      assert [_] = Podcasts.list_publish_targets!(actor: team.editor)
      assert [] = Podcasts.list_publish_targets!(actor: team.outsider)

      assert {:error, %Ash.Error.Forbidden{}} =
               Podcasts.update_publish_target(target, %{name: "x"}, actor: team.editor)

      assert {:error, %Ash.Error.Forbidden{}} =
               Podcasts.destroy_publish_target(target, actor: team.editor)
    end
  end
end
