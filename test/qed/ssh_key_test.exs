defmodule Qed.SshKeyTest do
  use ExUnit.Case, async: true

  alias Qed.SshKey

  test "generates an OpenSSH Ed25519 key pair" do
    {private, public} = SshKey.generate("my-comment")

    assert private =~ ~r/\A-----BEGIN OPENSSH PRIVATE KEY-----\n/
    assert private =~ ~r/-----END OPENSSH PRIVATE KEY-----\n\z/
    assert [_, _, "my-comment"] = String.split(public, " ")
    assert String.starts_with?(public, "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5")
  end

  test "private key lines are at most 70 characters" do
    {private, _public} = SshKey.generate()
    assert private |> String.split("\n") |> Enum.all?(&(String.length(&1) <= 70))
  end

  test "each call generates a new key" do
    {_, public1} = SshKey.generate()
    {_, public2} = SshKey.generate()
    refute public1 == public2
  end

  test "public_key/1 derives the public key from the private key" do
    {private, public} = SshKey.generate("qed")
    assert {:ok, ^public} = SshKey.public_key(private)
  end

  test "public_key/1 rejects garbage" do
    assert :error = SshKey.public_key("not a key")
  end

  @tag :tmp_dir
  test "ssh-keygen accepts the private key", %{tmp_dir: dir} do
    case System.find_executable("ssh-keygen") do
      nil ->
        :ok

      ssh_keygen ->
        {private, public} = SshKey.generate("qed")
        path = Path.join(dir, "id_ed25519")
        File.write!(path, private)
        File.chmod!(path, 0o600)

        assert {derived, 0} = System.cmd(ssh_keygen, ["-y", "-f", path])
        assert String.trim(derived) == public
    end
  end
end
