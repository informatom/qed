defmodule Qed.Podcasts.Changes.AddCreatorAsOwner do
  @moduledoc "Makes the user who creates a podcast its owner."
  use Ash.Resource.Change

  @impl true
  def change(changeset, _opts, context) do
    Ash.Changeset.after_action(changeset, fn _changeset, podcast ->
      case context.actor do
        %{id: user_id} ->
          Qed.Podcasts.Membership
          |> Ash.Changeset.for_create(:create, %{
            podcast_id: podcast.id,
            user_id: user_id,
            role: :owner
          })
          |> Ash.create(authorize?: false)
          |> case do
            {:ok, _membership} -> {:ok, podcast}
            {:error, error} -> {:error, error}
          end

        _ ->
          {:error, "a podcast needs an actor to become its owner"}
      end
    end)
  end
end
