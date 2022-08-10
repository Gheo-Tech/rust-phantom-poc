export async function connect_to_wallet() {
  try {
    const resp = await window.solana.connect();
    return resp.publicKey.toString();
  } catch (err) {
    return err;
  }
}
