contract;
use std::{
    auth::{
        AuthError,
        msg_sender,
    },
    call_frames::{
        contract_id,
        msg_asset_id,
    },
    context::{
        balance_of,
        msg_amount,
    },
    logging::log,
    storage::StorageVec,
    token::{
        mint_to_address,
        transfer_to_address,
    },
};

enum Error {
    InvalidPayment: (),
}

pub struct MarketConfiguration {
    base_token: ContractId,
}

pub struct AssetConfig {
    asset: ContractId,
    decimals: u8,
}

abi MyContract {
    #[storage(read, write)]
    fn initialize(config: MarketConfiguration, asset_configs: Vec<AssetConfig>);

    #[storage(read)]
    fn get_asset_config_by_asset_id(asset: ContractId) -> AssetConfig;

    #[storage(read)]
    fn supply_base();
}

storage {
    config: Option<MarketConfiguration> = Option::None,
    asset_configs: StorageVec<AssetConfig> = StorageVec {},
}

#[storage(read)]
fn get_config() -> MarketConfiguration {
    match storage.config {
        Option::Some(c) => c,
        _ => revert(0),
    }
}

#[storage(read)]
fn get_asset_config_by_asset_id_internal(asset: ContractId) -> AssetConfig {
    let mut out: Option<AssetConfig> = Option::None;
    let config = get_config();
    let mut i = 0;
    while i < storage.asset_configs.len() {
        let asset_config = storage.asset_configs.get(i).unwrap();
        if asset_config.asset == asset {
            out = Option::Some(asset_config);
            break;
        }
        i += 1;
    }
    match out {
        Option::Some(v) => v,
        Option::None(_) => revert(0),
    }
}

pub fn get_caller() -> Address {
    let sender: Result<Identity, AuthError> = msg_sender();
    if let Identity::Address(address) = sender.unwrap() {
        address
    } else {
        revert(0);
    }
}

impl MyContract for Contract {
    #[storage(read, write)]
    fn initialize(config: MarketConfiguration, asset_configs: Vec<AssetConfig>) {
        storage.config = Option::Some(config);
        let mut i = 0;
        while i < asset_configs.len() {
            storage.asset_configs.push(asset_configs.get(i).unwrap());
            i += 1;
        }
    }

    #[storage(read)]
    fn get_asset_config_by_asset_id(asset: ContractId) -> AssetConfig {
        get_asset_config_by_asset_id_internal(asset)
    }

    #[storage(read)]
    fn supply_base() {
        let caller = get_caller();
        let config = get_config();
        let amount = msg_amount();

        require(amount > 0, Error::InvalidPayment);
        require(msg_asset_id() == config.base_token, Error::InvalidPayment);

        mint_to_address(amount, caller);
    }

}
