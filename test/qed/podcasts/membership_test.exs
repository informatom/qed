defmodule Qed.Podcasts.MembershipTest do
  use Qed.DataCase, async: false

  import Qed.Fixtures

  alias Qed.Podcasts

  setup do
    team_fixture()
  end

  test "owners add members", %{podcast: podcast, owner: owner} do
    new_user = user_fixture()
    membership = Podcasts.add_member!(podcast.id, new_user.id, :editor, actor: owner)
    assert membership.role == :editor
  end

  test "editors and outsiders can't add members", team do
    new_user = user_fixture()

    for actor <- [team.editor, team.outsider] do
      assert {:error, %Ash.Error.Forbidden{}} =
               Podcasts.add_member(team.podcast.id, new_user.id, :editor, actor: actor)
    end
  end

  test "a user can only be a member once", %{podcast: podcast, owner: owner, editor: editor} do
    assert {:error, %Ash.Error.Invalid{}} =
             Podcasts.add_member(podcast.id, editor.id, :owner, actor: owner)
  end

  test "rejects unknown roles", %{podcast: podcast, owner: owner} do
    assert {:error, %Ash.Error.Invalid{}} =
             Podcasts.add_member(podcast.id, user_fixture().id, :admin, actor: owner)
  end

  test "owners change roles and remove members", %{owner: owner, editor: editor} do
    membership = Enum.find(Podcasts.list_members!(actor: owner), &(&1.user_id == editor.id))

    assert %{role: :owner} = Podcasts.change_member_role!(membership, :owner, actor: owner)
    assert :ok = Podcasts.remove_member!(membership, actor: owner)
  end

  test "editors can't change roles", %{owner: owner, editor: editor} do
    membership = Enum.find(Podcasts.list_members!(actor: owner), &(&1.user_id == editor.id))

    assert {:error, %Ash.Error.Forbidden{}} =
             Podcasts.change_member_role(membership, :owner, actor: editor)
  end

  test "outsiders can't see memberships", %{outsider: outsider} do
    assert [] = Podcasts.list_members!(actor: outsider)
  end
end
