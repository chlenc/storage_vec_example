contract;

pub struct MarketConfiguration {
    foo: u64,
    bar: u64,
    asset_configs: Vec<AssetConfig>,
}

pub struct AssetConfig {
    asset: ContractId,
    decimals: u8,
    blablabla: u64,
}

abi MyContract {
    #[storage(write)]
    fn initialize(config: MarketConfiguration);

    #[storage(read)]
    fn get_asset_config_by_asset_id(asset: ContractId) -> AssetConfig;
}

storage {
    config: Option<MarketConfiguration> = Option::None,
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
    while i < config.asset_configs.len() {
        let asset_config = config.asset_configs.get(i).unwrap();
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

impl MyContract for Contract {
    #[storage(write)]
    fn initialize(config: MarketConfiguration) {
        storage.config = Option::Some(config);
    }

    #[storage(read)]
    fn get_asset_config_by_asset_id(asset: ContractId) -> AssetConfig {
        get_asset_config_by_asset_id_internal(asset)
    }
}
