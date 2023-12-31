defmodule Poketwo.Database.Models.Generation do
  use Ecto.Schema
  alias Poketwo.Database.{Models, Utils, V1}

  schema "generations" do
    field :identifier, :string

    has_many :info, Models.GenerationInfo
    belongs_to :main_region, Models.Region
  end

  @spec to_protobuf(any) :: V1.Generation.t() | nil
  def to_protobuf(_)

  def to_protobuf(%Models.Generation{} = generation) do
    V1.Generation.new(
      id: generation.id,
      identifier: generation.identifier,
      info: Utils.map_if_loaded(generation.info, &Models.GenerationInfo.to_protobuf/1),
      main_region: Utils.if_loaded(generation.main_region, &Models.Region.to_protobuf/1)
    )
  end

  def to_protobuf(_), do: nil
end
