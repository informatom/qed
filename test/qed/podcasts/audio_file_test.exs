defmodule Qed.Podcasts.AudioFileTest do
  use Qed.DataCase, async: false

  import Qed.Fixtures

  alias Qed.Podcasts

  setup do
    team = team_fixture()
    Map.put(team, :episode, episode_fixture(team.podcast, team.owner))
  end

  test "an external audio file needs a URL", %{episode: episode, editor: editor} do
    assert {:error, %Ash.Error.Invalid{}} =
             Podcasts.create_audio_file(%{episode_id: episode.id, format: :mp3, source: :url},
               actor: editor
             )

    file =
      Podcasts.create_audio_file!(
        %{
          episode_id: episode.id,
          format: :mp3,
          source: :url,
          url: "https://cdn.example.com/ep1.mp3",
          byte_size: 1_234_567
        },
        actor: editor
      )

    assert file.url == "https://cdn.example.com/ep1.mp3"
  end

  test "an uploaded audio file needs a path", %{episode: episode, editor: editor} do
    assert {:error, %Ash.Error.Invalid{}} =
             Podcasts.create_audio_file(%{episode_id: episode.id, format: :m4a}, actor: editor)

    assert %{source: :upload} =
             Podcasts.create_audio_file!(
               %{episode_id: episode.id, format: :m4a, path: "uploads/ep1.m4a"},
               actor: editor
             )
  end

  test "one file per format and episode", %{episode: episode, editor: editor} do
    attrs = %{episode_id: episode.id, format: :ogg, path: "a.ogg"}
    Podcasts.create_audio_file!(attrs, actor: editor)

    assert {:error, %Ash.Error.Invalid{}} =
             Podcasts.create_audio_file(%{attrs | path: "b.ogg"}, actor: editor)
  end

  test "rejects unknown formats", %{episode: episode, editor: editor} do
    assert {:error, %Ash.Error.Invalid{}} =
             Podcasts.create_audio_file(%{episode_id: episode.id, format: :wav, path: "a.wav"},
               actor: editor
             )
  end

  test "mime_type is derived from the format", %{episode: episode, editor: editor} do
    file =
      Podcasts.create_audio_file!(%{episode_id: episode.id, format: :mp3, path: "a.mp3"},
        actor: editor
      )
      |> Ash.load!(:mime_type, actor: editor)

    assert file.mime_type == "audio/mpeg"
  end

  test "outsiders can't add audio files", %{episode: episode, outsider: outsider} do
    assert {:error, %Ash.Error.Forbidden{}} =
             Podcasts.create_audio_file(%{episode_id: episode.id, format: :mp3, path: "a.mp3"},
               actor: outsider
             )
  end

  test "episodes list their audio files", %{episode: episode, editor: editor} do
    for format <- [:mp3, :m4a] do
      Podcasts.create_audio_file!(%{episode_id: episode.id, format: format, path: "a.#{format}"},
        actor: editor
      )
    end

    episode = Ash.load!(episode, :audio_files, actor: editor)
    assert episode.audio_files |> Enum.map(& &1.format) |> Enum.sort() == [:m4a, :mp3]
  end
end
