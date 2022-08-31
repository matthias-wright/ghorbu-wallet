pub mod error;
pub mod fee;
pub mod transaction;
pub mod utxo;

static BITCOIN_API: &'static str = "https://mempool.space/api";
static BITCOIN_TESTNET_API: &'static str = "https://mempool.space/testnet/api";
