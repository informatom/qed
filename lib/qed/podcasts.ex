defmodule Qed.Podcasts do
  @moduledoc "Podcasts, their members, episodes, audio files and publish targets."
  use Ash.Domain, otp_app: :qed

  resources do
    resource Qed.Podcasts.Podcast do
      define :create_podcast, action: :create
      define :list_podcasts, action: :read
      define :get_podcast, action: :read, get_by: [:id]
      define :get_podcast_by_slug, action: :read, get_by: [:slug]
      define :update_podcast, action: :update
      define :destroy_podcast, action: :destroy
    end

    resource Qed.Podcasts.Membership do
      define :add_member, action: :create, args: [:podcast_id, :user_id, :role]
      define :list_members, action: :read
      define :change_member_role, action: :change_role, args: [:role]
      define :remove_member, action: :destroy
    end

    resource Qed.Podcasts.Episode do
      define :create_episode, action: :create
      define :list_episodes, action: :read
      define :get_episode, action: :read, get_by: [:id]
      define :update_episode, action: :update
      define :publish_episode, action: :publish
      define :unpublish_episode, action: :unpublish
      define :destroy_episode, action: :destroy
    end

    resource Qed.Podcasts.AudioFile do
      define :create_audio_file, action: :create
      define :update_audio_file, action: :update
      define :destroy_audio_file, action: :destroy
    end

    resource Qed.Podcasts.PublishTarget do
      define :create_publish_target, action: :create
      define :list_publish_targets, action: :read
      define :get_publish_target, action: :read, get_by: [:id]
      define :update_publish_target, action: :update
      define :regenerate_ssh_key, action: :regenerate_ssh_key
      define :destroy_publish_target, action: :destroy
    end
  end
end
