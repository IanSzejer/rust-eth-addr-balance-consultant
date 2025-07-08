use std::time::Duration;
use dotenv::dotenv;
use std::env;
use std::io::{self, Write};

use ethers::{
    prelude::{ Http, Middleware, Provider, U256 }, types::H160
};
use eyre::{ContextCompat, Ok, Result};

enum ConnectionState{
    Disconnected,
    Connected,
}

struct EthConnection {
    eth_endpoint: String,
    state: ConnectionState,
    provider: Option<Provider<Http>>,
}

impl EthConnection{
    //Constructor of EthConnection
    fn new(eth_endpoint: String) -> EthConnection{
        return EthConnection{
            eth_endpoint: eth_endpoint.to_string(),
            state: ConnectionState::Disconnected,
            provider: None
        }
    }

    //Method to connect provider to blockchain
    async fn connect_provider(&mut self) -> Result<()>{
        match self.state {
            ConnectionState::Disconnected => {
                let provider = Provider::try_from(&self.eth_endpoint)?
                    .interval(Duration::from_millis(10));
                self.provider = Some(provider);
                self.state = ConnectionState::Connected;
                Ok(())
            }
            ConnectionState::Connected => {
                eyre::bail!("Eth connection already established");
            }
        }
    }

    //Method to disconnect provider to blockchain
    async fn disconnect_provider(&mut self) -> Result<()>{
        match self.state {
            ConnectionState::Connected => {
                self.provider = None;
                self.state = ConnectionState::Disconnected;
                Ok(())
            }
            ConnectionState::Disconnected => {
                eyre::bail!("Eth connection is not established");
            }
        }
    }

    //Method to get address balance
    async fn get_balance_of_adrress(&mut self, address: H160) -> Result<U256>{
        if let ConnectionState::Disconnected = self.state{
            eyre::bail!("Eth connection is not stablished");
        }
        let provider = self.provider.as_ref().context("Provider not initialized")?;
        let balance = provider.get_balance(address, None).await?;

        Ok(balance)
    }
}

const EHT_IN_WEI: f64 = 1_000_000_000_000_000_000.0;

#[tokio::main]
async fn main() -> Result<()> {
//Get api_key from .env file
    dotenv().ok(); 
    let api_endpoint = env::var("INFURA_API_KEY").expect("API key not found");
    println!("API endpoint: {}", api_endpoint);
//Create eth connection
    let mut eth_connection = EthConnection::new(api_endpoint);
    eth_connection.connect_provider().await?;
    let mut keep_consulting = true;
    while keep_consulting{
        print!("Insert eth seplia address to consult its balance: ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Error reading standard input");
        
        let address: H160 = match input.parse() {
            Result::Ok(addr) => addr,
            Err(_) => {
                println!("Invalid Ethereum address. Please try again");
                continue;
            }
        };
        let balance = eth_connection.get_balance_of_adrress(address).await.inspect_err(|err|{
            println!("Failed to read balance of address {} due to error: {}",address, err)
        });
        match balance {
            Result::Ok(balance) => println!("The balance of the address: {} is {} eth",address,balance.as_u128() as f64 / EHT_IN_WEI),
            Err(_) => continue
        }

        loop {
            print!("Do you want to consult another address? (y/n): ");
            io::stdout().flush().unwrap();
            let mut again = String::new();
            io::stdin().read_line(&mut again).expect("Error reading standard input");
            let again = again.trim().to_lowercase();
            if again == "y" || again == "yes" {
                break; 
            } else if again == "n" || again == "no" {
                keep_consulting = false;
                break
            } else {
                println!("Please enter 's' to continue or 'n' to exit.");
            }
        }
    }
    eth_connection.disconnect_provider().await?;
    
    return Ok(())
}