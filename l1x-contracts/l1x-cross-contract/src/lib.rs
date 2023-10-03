use borsh::BorshSerialize;
use l1x_sdk::call_contract;
use l1x_sdk::contract;
use l1x_sdk::contract_interaction::ContractCall;
use l1x_sdk::emit_event_experimental;
use l1x_sdk::types::Address;
use serde::Serialize;

#[derive(BorshSerialize)]
struct Event {
    name: String,
}

pub struct Contract {}

#[contract]
impl Contract {
    pub fn new() {}

    pub fn hello() {
        let contract_instance_address: &[u8; 20] = b"l1x_contract\0\0\0\0\0\0\0\0";
        let address = Address::try_from(contract_instance_address.to_vec()).unwrap();
        let call = ContractCall {
            contract_address: address.clone(),
            method_name: "get_names".to_string(),
            args: "{}".as_bytes().to_vec(),
            read_only: true,
            fee_limit: 12,
        };

        match call_contract(&call) {
            Some(res) => {
                let res: Result<Vec<String>, serde_json::Error> = serde_json::from_slice(&res);
                l1x_sdk::msg(&format!("Returned by the external contract: {:?}", res));
            }
            None => {
                l1x_sdk::msg("None");
            }
        }
    }

    pub fn add_name(name: String) {
        let contract_instance_address: &[u8; 20] = b"l1x_contract\0\0\0\0\0\0\0\0";
        let address = Address::try_from(contract_instance_address.to_vec()).unwrap();

        let args = {
            #[derive(Serialize)]
            struct Args {
                name: String,
            }
            Args { name }
        };

        let call = ContractCall {
            contract_address: address.clone(),
            method_name: "add_name".to_string(),
            args: serde_json::to_vec(&args).unwrap(),
            read_only: false,
            fee_limit: 12,
        };

        match call_contract(&call) {
            Some(res) => {
                if res.is_empty() {
                    l1x_sdk::msg(&format!("The external contract returned the empty result"));
                } else {
                    let res: Result<Vec<String>, serde_json::Error> = serde_json::from_slice(&res);
                    l1x_sdk::msg(&format!("Returned by the external contract: {:?}", res));
                }
            }
            None => {
                l1x_sdk::msg("None");
            }
        }
    }

    pub fn emit_event() {
        emit_event_experimental(Event {
            name: "Hello".to_string(),
        });
    }
}
