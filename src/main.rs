use alloy::providers::{Provider, ProviderBuilder, RootProvider, WsConnect};
use alloy::pubsub::PubSubFrontend;
use alloy::rpc::types::eth::*;
use eyre::Result;
use futures_util::StreamExt;
use std::env;
use std::sync::Arc;
#[derive(Clone)]
struct AppState {
    provider: Arc<RootProvider<PubSubFrontend>>,
    address_to_watch: String,
}
impl AppState {
    pub async fn new() -> Result<Self> {
        let args = env::args().skip(1).collect::<Vec<String>>();

        if args.is_empty() {
            print_usage_details();
        }
        if args.len() != 2 {
            print_usage_details();
        }
        let address_to_watch = args[0].clone();
        let rpc_url = args[1].clone();
        let ws = WsConnect::new(rpc_url);
        let provider = ProviderBuilder::new().on_ws(ws).await?;
        Ok(Self {
            provider: Arc::new(provider),
            address_to_watch,
        })
    }

    pub fn get_watch_address(&self) -> &String {
        &self.address_to_watch
    }
}
type ProviderState = Arc<AppState>;

async fn initialize_state() -> Result<ProviderState> {
    let state = AppState::new().await?;
    Ok(Arc::new(state))
}
#[tokio::main]
async fn main() -> Result<()> {
    let state = initialize_state().await?;
    let provider = Arc::clone(&state.provider);

    let subscription = provider.subscribe_blocks().await?;

    let mut stream = subscription.into_stream();

    println!("Listening for new blocks... :)");

    println!("Waching incoming txns for : {}", state.address_to_watch);
    while let Some(block) = stream.next().await {
        let app_state = Arc::clone(&state);
        tokio::spawn(async move {
            process_block(block, app_state).await;
        });
    }
    Ok(())
}

async fn process_block(block: Block, app_state: ProviderState) {
    println!("Block: {}", block.header.number.expect("no number"));
    let full_block_result = app_state
        .provider
        .get_block(
            BlockId::number(block.header.number.expect("Invalid Block number found")),
            BlockTransactionsKind::Full,
        )
        .await;
    match full_block_result {
        Ok(block_res) => {
            for tx in block_res.expect("no block").transactions.txns() {
                if tx.to.unwrap().to_string().to_lowercase()
                    == app_state.get_watch_address().to_string().to_lowercase()
                {
                    print_txn_data(&tx);
                }
            }
        }
        Err(_) => {}
    }
}

fn print_txn_data(tx: &Transaction) {
    println!("------------------");
    println!("Transaction Hash: {}", tx.hash);
    println!("From: {}", tx.from);
    println!("To: {}", tx.to.expect("no to address"));
    println!("Tx {}", tx.input);
    println!("Value: {}", tx.value);
    println!("------------------");
}

fn print_usage_details() {
    println!("________________________________________________________");
    println!("Invalid arguments provided :(");
    println!("Example:");
    println!("./address-monitor REPLACE_ADDRESS_HERE YOUR_WSS_RPC_URL");
    println!("________________________________________________________");
    std::process::exit(1);
}
