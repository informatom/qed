defmodule Qed.Podcasts.Chapter do
  @moduledoc "A chapter mark (Podlove Simple Chapters): start time plus title."
  use Ash.Resource, data_layer: :embedded

  attributes do
    attribute :start, :string do
      allow_nil? false
      public? true
      constraints match: ~r/^\d{1,2}:\d{2}:\d{2}(\.\d{1,3})?$/
      description "Normal play time, e.g. `00:01:30` or `00:01:30.500`"
    end

    attribute :title, :string, allow_nil?: false, public?: true
    attribute :href, :string, public?: true
  end
end
