use borsh::{BorshDeserialize, BorshSerialize};
use l1x_sdk::{contract, store::LookupMap};
use serde::{Serialize, Deserialize};

#[derive(BorshSerialize, BorshDeserialize)]
pub struct InterOpsFlow {
    events: LookupMap<String, Vec<u8>>,
    state: LookupMap<String, Vec<u8>>,
    total_events: u64,
}

#[derive(Serialize, Deserialize, Default)]
pub struct Mint {
    to: String,
    global_tx_id: String,
    token_uri: String
}

#[derive(Serialize, Deserialize, Default)]
pub struct SubmitBurningTx {
    global_tx_id: String,
    token_address: String,
    token_id: u128
}

#[derive(Serialize, Deserialize, Default)]
pub struct TransferFundsToSeller {
    global_tx_id: String,
    seller: String,
    amount: u128
}


#[derive(Serialize, Deserialize, Default)]
pub struct AdvertisementStarted {
    nft_contract: String,
    token_id: u128,
    token_uri: String,
    owner: String,
    price: u128
}

#[derive(Serialize, Deserialize, Default)]
pub struct AdvertisementTransferred {
    global_tx_id: String,
    to: String
}

#[derive(Serialize, Deserialize, Default)]
pub struct AdvertisementFinished {
    global_tx_id: String,
    nft_contract: String,
    token_id: String,
    old_owner: String,
    new_owner: String,
    price: u128 
}

impl Default for InterOpsFlow {
    fn default() -> Self {
        Self {
            events: LookupMap::new(b"events".to_vec()),
            state: LookupMap::new(b"default".to_vec()),
            total_events: Default::default(),
        }
    }
}



#[contract]
impl InterOpsFlow {
    fn load() -> Self {
        match l1x_sdk::storage_read(b"iterops-flow") {
            Some(vec) => borsh::BorshDeserialize::deserialize(&mut &vec[..]).unwrap(),
            None => Self::default(),
        }
    }

    fn save(&self) {
        let encoded_contract = borsh::BorshSerialize::try_to_vec(self).unwrap();
        l1x_sdk::storage_write(b"iterops-flow", &encoded_contract);
    }

    fn to_key(global_tx_id: &String, event_type: &String) -> String {
        global_tx_id.clone() + event_type
    }

    pub fn update_state(global_tx_id: String) {
        let mut contract = Self::load();

        if contract.state.get(&(global_tx_id.to_owned()+"mint")).is_none() {
            let payload = contract.events.get(&(global_tx_id.to_owned()+"AdvertisementStarted")).unwrap().clone();

            let start: AdvertisementStarted = serde_json::from_slice(&payload).unwrap();
            
            let mint = Mint {
                to: "0x17E66991AC9be7599eC66c08e3B2D63254458549".to_string(),
                global_tx_id: global_tx_id.clone(),
                token_uri: start.token_uri
            };
            contract.state.insert(global_tx_id.to_owned()+"mint", serde_json::to_string(&mint).unwrap().into_bytes());
        } else if contract.state.get(&(global_tx_id.to_owned()+"mint")).is_some() {
            let payload = contract.events.get(&(global_tx_id.to_owned()+"AdvertisementTransferred")).unwrap().clone();
            let start: AdvertisementStarted = serde_json::from_slice(&payload).unwrap();
            let burn = SubmitBurningTx {
                global_tx_id: global_tx_id.clone(),
                token_address: start.nft_contract,
                token_id: start.token_id
            };
            contract.state.insert(global_tx_id.to_owned()+"transfer", serde_json::to_string(&burn).unwrap().into_bytes());

        } else if contract.state.get(&(global_tx_id.to_owned()+"transfer")).is_none() {
            let payload = contract.events.get(&(global_tx_id.to_owned()+"AdvertisementTransferred")).unwrap().clone();
            let transfer: AdvertisementTransferred = serde_json::from_slice(&payload).unwrap();

            let payload = contract.events.get(&(global_tx_id.to_owned()+"AdvertisementFinished")).unwrap().clone();
            let finish: AdvertisementFinished = serde_json::from_slice(&payload).unwrap();

            let transfer_funds = TransferFundsToSeller {
                global_tx_id: global_tx_id.clone(),
                seller: transfer.to,
                amount: finish.price
            };

            contract.state.insert(global_tx_id.to_owned()+"finish", serde_json::to_string(&transfer_funds).unwrap().into_bytes());

        } else {
            
        }
    }

    pub fn save_event_data(global_tx_id: String, event_type: String, event_data: Vec<u8>) {
        let mut contract = Self::load();

        let key = Self::to_key(&global_tx_id, &event_type);
        contract.events.insert(key, event_data);
        contract.total_events += 1;

        contract.save();
    }

    pub fn get_payload_to_sign(global_tx_id: String) -> Vec<u8> {
        let contract = Self::load();
        if let Some(data) = contract.state.get(&(global_tx_id.to_owned()+"finish")) {
            return data.clone();
        } else if let Some(data) = contract.state.get(&(global_tx_id.to_owned()+"transfer")) {
            return data.clone()
        } else if let Some(data) = contract.state.get(&(global_tx_id.to_owned()+"mint")) {
            return data.clone()
        }  else {
            return vec![];
        }
    }

    pub fn total_events() -> u64 {
        let contract = Self::load();

        contract.total_events
    }
}