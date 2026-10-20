defmodule Qed.SshKey do
  @moduledoc """
  Generates Ed25519 key pairs in OpenSSH format.

  OTP's `:ssh_file` encoder writes the private part in a layout OpenSSH can't
  read, so the (small) `openssh-key-v1` format is encoded here directly.
  """

  @key_type "ssh-ed25519"

  @doc "Returns `{private_key_pem, public_key_line}`."
  @spec generate(String.t()) :: {String.t(), String.t()}
  def generate(comment \\ "qed") do
    {pub, priv} = :crypto.generate_key(:eddsa, :ed25519)
    {encode_private(pub, priv, comment), encode_public(pub, comment)}
  end

  @doc "Derives the public key line from an OpenSSH private key."
  @spec public_key(String.t()) :: {:ok, String.t()} | :error
  def public_key(private_pem) do
    with [_, body] <-
           Regex.run(~r/-----BEGIN OPENSSH PRIVATE KEY-----(.*)-----END/s, private_pem),
         {:ok, bin} <- Base.decode64(String.replace(body, ~r/\s/, "")),
         "openssh-key-v1\0" <> rest <- bin,
         {_cipher, rest} <- read_string(rest),
         {_kdf, rest} <- read_string(rest),
         {_kdf_opts, rest} <- read_string(rest),
         <<1::32, rest::binary>> <- rest,
         {pub_blob, _rest} <- read_string(rest),
         {@key_type, pub_rest} <- read_string(pub_blob),
         {pub, ""} <- read_string(pub_rest) do
      {:ok, encode_public(pub, "qed")}
    else
      _ -> :error
    end
  end

  defp encode_public(pub, comment) do
    "#{@key_type} #{Base.encode64(public_blob(pub))} #{comment}"
  end

  defp encode_private(pub, priv, comment) do
    check = :crypto.strong_rand_bytes(4)

    section =
      check <>
        check <>
        string(@key_type) <> string(pub) <> string(priv <> pub) <> string(comment)

    body =
      "openssh-key-v1\0" <>
        string("none") <>
        string("none") <>
        string("") <>
        <<1::32>> <> string(public_blob(pub)) <> string(pad(section))

    lines =
      body
      |> Base.encode64()
      |> chunk_all(70)
      |> Enum.join("\n")

    "-----BEGIN OPENSSH PRIVATE KEY-----\n#{lines}\n-----END OPENSSH PRIVATE KEY-----\n"
  end

  defp public_blob(pub), do: string(@key_type) <> string(pub)

  defp string(bin), do: <<byte_size(bin)::32, bin::binary>>

  defp read_string(<<len::32, value::binary-size(len), rest::binary>>), do: {value, rest}
  defp read_string(_), do: :error

  # Pad with bytes 1, 2, 3, ... to a multiple of the 8-byte block size.
  defp pad(section) do
    case rem(byte_size(section), 8) do
      0 -> section
      r -> section <> :binary.list_to_bin(Enum.to_list(1..(8 - r)))
    end
  end

  defp chunk_all(str, size) when byte_size(str) <= size, do: [str]

  defp chunk_all(str, size) do
    <<part::binary-size(^size), rest::binary>> = str
    [part | chunk_all(rest, size)]
  end
end
