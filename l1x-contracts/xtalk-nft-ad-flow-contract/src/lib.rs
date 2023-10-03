use borsh::{BorshDeserialize, BorshSerialize};
use l1x_sdk::types::{U128, U64};
use l1x_sdk::{contract, store::LookupMap};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const STORAGE_CONTRACT_KEY: &[u8; 12] = b"iterops-flow";
const STORAGE_EVENTS_KEY: &[u8; 6] = b"events";
const STORAGE_STATE_KEY: &[u8; 5] = b"state";

const STATE_1: &str = "mint";
const STATE_2: &str = "transfer";
const STATE_3: &str = "finish";

const START_EVENT: &str = "AdvertisementStarted";
const TRANSFER_EVENT: &str = "AdvertisementTransferred";
const FINISH_EVENT: &str = "AdvertisementFinished";

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub enum Event {
    AdvertisementStarted(AdvertisementStarted),
    AdvertisementTransferred(AdvertisementTransferred),
    AdvertisementFinished(AdvertisementFinished),
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub enum State {
    MintData(MintData),
    SubmitBurningTxData(SubmitBurningTxData),
    TransferFundsToSellerData(TransferFundsToSellerData),
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub enum Payload {
    Mint(Mint),
    SubmitBurningTx(SubmitBurningTx),
    TransferFundsToSeller(TransferFundsToSeller),
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct InterOpsFlow {
    events: LookupMap<String, Event>,
    state: LookupMap<String, State>,
    total_events: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Serialize, Deserialize)]
pub struct MintData {
    to: String,
    global_tx_id: String,
    token_uri: String,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Serialize, Deserialize)]
pub struct Mint {
    data: MintData,
    hash: String,
}
#[derive(BorshSerialize, BorshDeserialize, Clone, Serialize, Deserialize)]
pub struct SubmitBurningTxData {
    global_tx_id: String,
    token_address: String,
    token_id: U128,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Serialize, Deserialize)]
pub struct SubmitBurningTx {
    data: SubmitBurningTxData,
    hash: String,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Serialize, Deserialize)]
pub struct TransferFundsToSellerData {
    global_tx_id: String,
    seller: String,
    amount: U128,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Serialize, Deserialize)]
pub struct TransferFundsToSeller {
    data: TransferFundsToSellerData,
    hash: String,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Serialize, Deserialize)]
pub struct AdvertisementStarted {
    nft_contract: String,
    token_id: U128,
    token_uri: String,
    owner: String,
    price: U128,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Serialize, Deserialize)]
pub struct AdvertisementTransferred {
    global_tx_id: String,
    to: String,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Serialize, Deserialize)]
pub struct AdvertisementFinished {
    global_tx_id: String,
    nft_contract: String,
    token_id: U128,
    old_owner: String,
    new_owner: String,
    price: U128,
}

impl Default for InterOpsFlow {
    fn default() -> Self {
        Self {
            events: LookupMap::new(STORAGE_EVENTS_KEY.to_vec()),
            state: LookupMap::new(STORAGE_STATE_KEY.to_vec()),
            total_events: u64::default(),
        }
    }
}

#[contract]
impl InterOpsFlow {
    fn load() -> Self {
        match l1x_sdk::storage_read(STORAGE_CONTRACT_KEY) {
            Some(bytes) => Self::try_from_slice(&bytes).unwrap(),
            None => panic!("The contract isn't initialized"),
        }
    }

    fn save(&mut self) {
        let encoded_contract = borsh::BorshSerialize::try_to_vec(self).unwrap();
        l1x_sdk::storage_write(STORAGE_CONTRACT_KEY, &encoded_contract);
    }

    fn to_key(global_tx_id: &String, event_type: &String) -> String {
        global_tx_id.clone() + event_type
    }

    pub fn new() {
        let mut contract = Self::default();
        contract.save();
    }

    pub fn update_state(global_tx_id: String) {
        let mut contract = Self::load();

        if contract
            .state
            .get(&(global_tx_id.to_owned() + STATE_1))
            .is_none()
        {
            if let Event::AdvertisementStarted(start) = contract
                .events
                .get(&(global_tx_id.to_owned() + START_EVENT))
                .unwrap()
            {
                let mint = MintData {
                    to: "0x17E66991AC9be7599eC66c08e3B2D63254458549".to_string(),
                    global_tx_id: global_tx_id.clone(),
                    token_uri: start.token_uri.clone(),
                };
                contract
                    .state
                    .insert(global_tx_id.to_owned() + STATE_1, State::MintData(mint));
            } else {
                panic!("This is not an AdvertisementStarted variant.");
            }
        } else if contract
            .state
            .get(&(global_tx_id.to_owned() + STATE_2))
            .is_none()
        {
            if let Event::AdvertisementStarted(start) = contract
                .events
                .get(&(global_tx_id.to_owned() + START_EVENT))
                .unwrap()
            {
                let burn = SubmitBurningTxData {
                    global_tx_id: global_tx_id.clone(),
                    token_address: start.nft_contract.clone(),
                    token_id: start.token_id,
                };
                contract.state.insert(
                    global_tx_id.to_owned() + STATE_2,
                    State::SubmitBurningTxData(burn),
                );
            } else {
                panic!("This is not an AdvertisementStarted variant.");
            }
        } else if let Some(Event::AdvertisementTransferred(transfer)) = contract
            .events
            .get(&(global_tx_id.to_owned() + TRANSFER_EVENT))
        {
            if let Some(Event::AdvertisementFinished(finish)) = contract
                .events
                .get(&(global_tx_id.to_owned() + FINISH_EVENT))
            {
                let transfer_funds = TransferFundsToSellerData {
                    global_tx_id: global_tx_id.clone(),
                    seller: transfer.to.clone(),
                    amount: finish.price,
                };

                contract.state.insert(
                    global_tx_id.to_owned() + STATE_3,
                    State::TransferFundsToSellerData(transfer_funds),
                );
            } else {
                panic!("This is not an AdvertisementFinished variant.");
            }
        } else {
            panic!("invalid global transaction id: {}", global_tx_id);
        }
    }

    pub fn save_event_data(global_tx_id: String, event_type: String, event_data: Event) {
        let mut contract = Self::load();

        let key = Self::to_key(&global_tx_id, &event_type);
        contract.events.insert(key, event_data);
        contract.total_events += 1;

        contract.save();
    }

    pub fn get_payload_to_sign(global_tx_id: String) -> Payload {
        let contract = Self::load();

        if let Some(state) = contract.state.get(&(global_tx_id.to_owned() + STATE_3)) {
            if let State::TransferFundsToSellerData(data) = state {
                let mut hasher = Sha256::new();
                hasher.update(
                    (data.global_tx_id.clone() + &data.seller + &data.amount.0.to_string())
                        .as_bytes(),
                );
                let hash_result = format!("{:X}", hasher.finalize());

                let output = TransferFundsToSeller {
                    data: data.clone(),
                    hash: hash_result,
                };

                return Payload::TransferFundsToSeller(output);
            }
        } else if let Some(state) = contract.state.get(&(global_tx_id.to_owned() + STATE_2)) {
            if let State::SubmitBurningTxData(data) = state {
                let mut hasher = Sha256::new();
                hasher.update(
                    (data.global_tx_id.clone()
                        + &data.token_address
                        + &data.token_id.0.to_string())
                        .as_bytes(),
                );
                let hash_result = format!("{:X}", hasher.finalize());

                let output = SubmitBurningTx {
                    data: data.clone(),
                    hash: hash_result,
                };
                return Payload::SubmitBurningTx(output);
            }
        } else if let Some(state) = contract.state.get(&(global_tx_id.to_owned() + STATE_1)) {
            if let State::MintData(data) = state {
                let mut hasher = Sha256::new();
                hasher.update((data.to.clone() + &data.global_tx_id + &data.token_uri).as_bytes());
                let hash_result = format!("{:X}", hasher.finalize());
                let output = Mint {
                    data: data.clone(),
                    hash: hash_result,
                };
                return Payload::Mint(output);
            }
        }
        panic!("invalid global transaction id: {}", global_tx_id);
    }

    pub fn total_events() -> U64 {
        let contract = Self::load();

        contract.total_events.into()
    }
}
