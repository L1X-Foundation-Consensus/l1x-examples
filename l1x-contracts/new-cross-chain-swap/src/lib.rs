use borsh::{BorshDeserialize, BorshSerialize};
use ethers::abi::{encode_packed, ethabi, ParamType, Token};
use ethers::prelude::{parse_log, EthEvent};
use ethers::types::{Address, Signature};
use l1x_sdk::types::U64;
use l1x_sdk::{contract, store::LookupMap};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

const STORAGE_CONTRACT_KEY: &[u8; 21] = b"cross-chain-swap-flow";
const STORAGE_EVENTS_KEY: &[u8; 6] = b"events";
const STORAGE_STATE_KEY: &[u8; 8] = b"payloads";

const PAYLOAD_1: &str = "execute_swap";
const PAYLOAD_2: &str = "finalize_swap";

const INITIATE_EVENT: &str = "SwapInitiated";
const EXECUTE_EVENT: &str = "SwapExecuted";

const BSC_PROVIDER: &str =
    "https://rpc.ankr.com/bsc/042948a376b97fb943df8bb4398415769202661365ac0186dcae8455810f2e9e";
const ETHEREUM_PROVIDER: &str = "https://rpc.ankr.com/eth/042948a376b97fb943df8bb4398415769202661365ac0186dcae8455810f2e9e";
const AVAX_PROVIDER: &str = "https://rpc.ankr.com/avalanche-c/042948a376b97fb943df8bb4398415769202661365ac0186dcae8455810f2e9e";

const BSC_RECEIVER: &str = "0x351a25893e8E729045BE22eF0049E351527168D3";
const ETHEREUM_RECEIVER: &str = "0x351a25893e8E729045BE22eF0049E351527168D3";
const AVAX_RECEIVER: &str = "0x57FAE9445cD6532aCAC902372a319457b591417a";

const BSC_CHAIN_ID: u64 = 56;
const ETHEREUM_CHAIN_ID: u64 = 1;
const AVAX_CHAIN_ID: u64 = 43114;

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub enum Event {
    /// Emitted when swap is initiated.
    SwapInitiated(SwapInitiatedEvent),
    /// Emitted when swap is executed.
    SwapExecuted(SwapFullfilled),
}

#[derive(Clone, Debug, EthEvent)]
#[ethevent(name = "SwapInitiated")]
struct SwapInitiatedSolidityEvent {
    #[ethevent(indexed)]
    global_tx_id: [u8; 32],
    fee_percent: ethers::types::U256,
    source_amount: ethers::types::U256,
    destination_amount: ethers::types::U256,
    sender_address: ethers::types::Address,
    #[ethevent(indexed)]
    receiver_address: ethers::types::Address,
    source_asset_address: ethers::types::Address,
    #[ethevent(indexed)]
    destination_asset_address: ethers::types::Address,
    destination_asset_symbol: String,
    source_asset_symbol: String,
    destination_network: String,
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct SwapInitiatedEvent {
    global_tx_id: [u8; 32],
    fee_percent: l1x_sdk::types::U256,
    source_amount: l1x_sdk::types::U256,
    destination_amount: l1x_sdk::types::U256,
    sender_address: l1x_sdk::types::Address,
    receiver_address: l1x_sdk::types::Address,
    source_asset_address: l1x_sdk::types::Address,
    destination_asset_address: l1x_sdk::types::Address,
    destination_asset_symbol: String,
    source_asset_symbol: String,
    destination_network: String,
}

#[derive(Clone, Debug, EthEvent, Serialize, Deserialize)]
#[ethevent(name = "SwapFullfilled")]
pub struct SwapFullfilledSolidityEvent {
    #[ethevent(indexed)]
    global_tx_id: [u8; 32],
    destination_amount: ethers::types::U256,
    #[ethevent(indexed)]
    receiver_address: ethers::types::Address,
    #[ethevent(indexed)]
    destination_asset_address: ethers::types::Address,
    destination_asset_symbol: String,
    destination_network: String,
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct SwapFullfilled {
    global_tx_id: [u8; 32],
    destination_amount: l1x_sdk::types::U256,
    receiver_address: l1x_sdk::types::Address,
    destination_asset_address: l1x_sdk::types::Address,
    destination_asset_symbol: String,
    destination_network: String,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub enum Payload {
    ExecuteSwap(SwapFullfilled),
    FinalizeSwap(FinalizeSwapPayload),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetPayloadResponse {
    input_data: String,
    provider: String,
    chain_id: u64,
    to: Address,
}

#[derive(Clone, Debug, EthEvent, Serialize, Deserialize)]
#[ethevent(name = "FinalizeSwapPayload")]
pub struct FinalizeSwapSolidityPayload {
    #[ethevent(indexed)]
    global_tx_id: [u8; 32],
    user: ethers::types::Address,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Serialize, Deserialize)]
pub struct FinalizeSwapPayload {
    global_tx_id: [u8; 32],
    user: l1x_sdk::types::Address,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct CrossChainSwapFlow {
    events: LookupMap<String, Event>,
    payloads: LookupMap<String, Payload>,
    total_events: u64,
}

impl From<SwapInitiatedSolidityEvent> for SwapInitiatedEvent {
    fn from(event: SwapInitiatedSolidityEvent) -> Self {
        let mut fee_percent = vec![0u8; 32];
        let mut source_amount = vec![0u8; 32];
        let mut destination_amount = vec![0u8; 32];
        event.fee_percent.to_little_endian(&mut fee_percent);
        event.source_amount.to_little_endian(&mut source_amount);
        event
            .destination_amount
            .to_little_endian(&mut destination_amount);
        Self {
            global_tx_id: event.global_tx_id,
            fee_percent: l1x_sdk::types::U256::from_little_endian(&fee_percent),
            source_amount: l1x_sdk::types::U256::from_little_endian(&source_amount),
            destination_amount: l1x_sdk::types::U256::from_little_endian(&destination_amount),
            sender_address: l1x_sdk::types::Address::from(event.sender_address.0),
            receiver_address: l1x_sdk::types::Address::from(event.receiver_address.0),
            source_asset_address: l1x_sdk::types::Address::from(event.source_asset_address.0),
            destination_asset_address: l1x_sdk::types::Address::from(
                event.destination_asset_address.0,
            ),
            destination_asset_symbol: event.destination_asset_symbol,
            source_asset_symbol: event.source_asset_symbol,
            destination_network: event.destination_network,
        }
    }
}

impl From<SwapFullfilledSolidityEvent> for SwapFullfilled {
    fn from(event: SwapFullfilledSolidityEvent) -> Self {
        let mut destination_amount = vec![0u8; 32];
        event
            .destination_amount
            .to_little_endian(&mut destination_amount);
        Self {
            global_tx_id: event.global_tx_id,
            destination_amount: l1x_sdk::types::U256::from_little_endian(&destination_amount),
            receiver_address: l1x_sdk::types::Address::from(event.receiver_address.0),
            destination_asset_address: l1x_sdk::types::Address::from(
                event.destination_asset_address.0,
            ),
            destination_asset_symbol: event.destination_asset_symbol,
            destination_network: event.destination_network,
        }
    }
}

impl From<SwapFullfilled> for SwapFullfilledSolidityEvent {
    fn from(swap: SwapFullfilled) -> Self {
        let mut destination_amount = vec![0u8; 32];
        swap.destination_amount
            .to_little_endian(&mut destination_amount);
        Self {
            global_tx_id: swap.global_tx_id,
            destination_amount: ethers::types::U256::from_little_endian(&destination_amount),
            receiver_address: ethers::types::Address::from_slice(swap.receiver_address.as_bytes()),
            destination_asset_address: ethers::types::Address::from_slice(
                swap.destination_asset_address.as_bytes(),
            ),
            destination_asset_symbol: swap.destination_asset_symbol,
            destination_network: swap.destination_network,
        }
    }
}

impl From<FinalizeSwapPayload> for FinalizeSwapSolidityPayload {
    fn from(payload: FinalizeSwapPayload) -> Self {
        Self {
            global_tx_id: payload.global_tx_id,
            user: ethers::types::Address::from_slice(payload.user.as_bytes()),
        }
    }
}

impl Default for CrossChainSwapFlow {
    fn default() -> Self {
        Self {
            events: LookupMap::new(STORAGE_EVENTS_KEY.to_vec()),
            payloads: LookupMap::new(STORAGE_STATE_KEY.to_vec()),
            total_events: u64::default(),
        }
    }
}

#[contract]
impl CrossChainSwapFlow {
    /// Generate contract based on bytes in storage
    fn load() -> Self {
        match l1x_sdk::storage_read(STORAGE_CONTRACT_KEY) {
            Some(bytes) => match Self::try_from_slice(&bytes) {
                Ok(contract) => contract,
                Err(_) => {
                    panic!("Unable to parse contract bytes")
                }
            },
            None => {
                panic!("The contract isn't initialized")
            }
        }
    }

    /// Save contract to storage
    fn save(&mut self) {
        match borsh::BorshSerialize::try_to_vec(self) {
            Ok(encoded_contract) => {
                l1x_sdk::storage_write(STORAGE_CONTRACT_KEY, &encoded_contract);
                log::info!("Saved event data successfully");
            }
            Err(_) => panic!("Unable to save contract"),
        };
    }

    /// Generate key based on given inputs
    ///
    /// - `global_tx_id`: Global transaction identifier
    /// - `event_type`: Type of event
    fn to_key(global_tx_id: &str, event_type: &str) -> String {
        global_tx_id.to_owned() + event_type
    }

    /// Instantiate and save contract to storage
    pub fn new() {
        let mut contract = Self::default();
        contract.save();
    }

    /// Save event to contract storage
    ///
    /// - `global_tx_id`: Global transaction identifier
    /// - `source_id`: Source Identifier
    /// - `event_data`: Date to store in contract's storage
    pub fn save_event_data(global_tx_id: String, source_id: U64, event_data: String) {
        let mut contract = Self::load();
        log::info!("Received event data!!!");
        let event_data = match base64::decode(event_data.as_bytes()) {
            Ok(data) => data,
            Err(_) => panic!("Can't decode base64 event_data"),
        };

        // Save swap event based on source_id
        match source_id.0 {
            0 => contract.save_swap_initiated_data(&global_tx_id, event_data),
            1 => contract.save_swap_executed_data(&global_tx_id, event_data),
            _ => {
                panic!("Unknown source id: {}", source_id.0);
            }
        };
        contract.save()
    }

    /// Retrieve payload hash to sign
    ///
    /// - `global_tx_id`: Global transaction identifier
    pub fn get_payload_hash_to_sign(global_tx_id: String) -> String {
        let contract = Self::load();

        if let Some(payloads) = contract
            .payloads
            .get(&(global_tx_id.to_owned() + PAYLOAD_2))
        {
            if let Payload::FinalizeSwap(_data) = payloads {
                //return PayloadResponse::FinalizeSwap(data.clone().into());
            }
        } else if let Some(Payload::ExecuteSwap(data)) = contract
            .payloads
            .get(&(global_tx_id.to_owned() + PAYLOAD_1))
        {
            let payload: SwapFullfilledSolidityEvent = data.clone().into();

            let mut buf = [0; 32];
            payload.destination_amount.to_big_endian(&mut buf);

            let bytes =
                encode_packed(&vec![Token::FixedBytes(payload.global_tx_id.into())]).unwrap();

            return hex::encode(ethers::utils::keccak256(bytes));
        }
        panic!("invalid global transaction id: {}", global_tx_id);
    }

    /// Retrieve payload from the signature
    ///
    /// - `global_tx_id`: Global transaction identifier
    ///  - `signature`: Signature of the payload
    pub fn get_pay_load(global_tx_id: String, signature: String) -> GetPayloadResponse {
        let contract = Self::load();
        let signature: Signature = match Signature::from_str(&signature) {
            Ok(signature) => signature,
            Err(error) => panic!("{:?}", error.to_string()),
        };

        if let Some(payloads) = contract
            .payloads
            .get(&(global_tx_id.to_owned() + PAYLOAD_2))
        {
            if let Payload::FinalizeSwap(_data) = payloads {
                //return PayloadResponse::FinalizeSwap(data.clone().into());
            }
        } else if let Some(Payload::ExecuteSwap(data)) = contract
            .payloads
            .get(&(global_tx_id.to_owned() + PAYLOAD_1))
        {
            let payload: SwapFullfilledSolidityEvent = data.clone().into();
            let function_selector = hex::encode(ethabi::short_signature(
                "executeSwap",
                &[
                    ParamType::Uint(256),
                    ParamType::Address,
                    ParamType::Address,
                    ParamType::String,
                    ParamType::String,
                    ParamType::FixedBytes(32),
                    ParamType::Bytes,
                ],
            ));

            // Construct the transaction data for encoding
            let transaction_data = vec![
                Token::Uint(payload.destination_amount),
                Token::Address(payload.receiver_address),
                Token::Address(payload.destination_asset_address),
                Token::String(payload.destination_asset_symbol),
                Token::String(payload.destination_network.clone()),
                Token::FixedBytes(payload.global_tx_id.to_vec()),
                Token::Bytes(signature.into()),
            ];

            // Encode the transaction data into bytes
            let encoded_transaction_data = ethabi::encode(&transaction_data);
            let data_without_function_signature = hex::encode(encoded_transaction_data);
            let data = function_selector.to_owned() + &data_without_function_signature;

            let mut _provider = AVAX_PROVIDER;
            let mut chain_id = AVAX_CHAIN_ID;
            let mut receiver = AVAX_RECEIVER;
            
            if payload.destination_network == "BSC" {
                _provider = BSC_PROVIDER;
                chain_id = BSC_CHAIN_ID;
                receiver = BSC_RECEIVER;
            } else if payload.destination_network == "ETH" {
                _provider = ETHEREUM_PROVIDER;
                chain_id = ETHEREUM_CHAIN_ID;
                receiver = ETHEREUM_RECEIVER
            }
            return GetPayloadResponse {
                input_data: data,
                provider: _provider.to_string(),
                chain_id,
                to: receiver
                    .to_string()
                    .parse::<Address>()
                    .expect("Unable to parse contract address"),
            };
        }
        panic!("invalid global transaction id: {}", global_tx_id);
    }

    /// Retrieve total number of events
    pub fn total_events() -> U64 {
        let contract = Self::load();
        contract.total_events.into()
    }

    /// Save swap initiated event
    ///
    /// - `global_tx_id`: Global transaction identifier
    /// - `event_data`: Data to save
    fn save_swap_initiated_data(&mut self, global_tx_id: &str, event_data: Vec<u8>) {
        match serde_json::from_slice(&event_data)
            .map_err(|error| error.to_string())
            .and_then(|log: ethers::types::Log| {
                parse_log::<SwapInitiatedSolidityEvent>(log).map_err(|error| error.to_string())
            }) {
            Ok(event) => {
                let key = Self::to_key(global_tx_id, INITIATE_EVENT);
                let event_data: SwapInitiatedEvent = event.clone().into();
                self.events
                    .insert(key, Event::SwapInitiated(event_data.clone()));

                let fulfill_swap = SwapFullfilled {
                    destination_amount: event_data.destination_amount,
                    receiver_address: event_data.receiver_address,
                    destination_asset_address: event_data.destination_asset_address,
                    destination_asset_symbol: event_data.destination_asset_symbol,
                    destination_network: event_data.destination_network,
                    global_tx_id: event_data.global_tx_id,
                };
                self.payloads.insert(
                    global_tx_id.to_owned() + PAYLOAD_1,
                    Payload::ExecuteSwap(fulfill_swap),
                );
                self.total_events = match self.total_events.checked_add(1) {
                    Some(result) => result,
                    None => panic!("Arithmetic Overflow"),
                };
            }
            Err(error) => {
                panic!("{}", error.to_string())
            }
        }
    }

    /// Save swap executed event
    ///
    /// - `global_tx_id`: Global transaction identifier
    /// - `event_data`: Data to save
    fn save_swap_executed_data(&mut self, global_tx_id: &str, event_data: Vec<u8>) {
        match serde_json::from_slice(&event_data)
            .map_err(|error| error.to_string())
            .and_then(|log: ethers::types::Log| {
                parse_log::<SwapFullfilledSolidityEvent>(log).map_err(|error| error.to_string())
            }) {
            Ok(event) => {
                let key = Self::to_key(global_tx_id, EXECUTE_EVENT);
                let event_data: SwapFullfilled = event.clone().into();
                self.events
                    .insert(key, Event::SwapExecuted(event_data.clone()));
                let finalize_swap = FinalizeSwapPayload {
                    global_tx_id: event_data.global_tx_id,
                    user: event_data.receiver_address,
                };
                self.payloads.insert(
                    global_tx_id.to_owned() + PAYLOAD_2,
                    Payload::FinalizeSwap(finalize_swap),
                );
                self.total_events = match self.total_events.checked_add(1) {
                    Some(result) => result,
                    None => panic!("Arithmetic Overflow"),
                };
            }
            Err(error) => {
                panic!("{}", error.to_string())
            }
        }
    }
}
