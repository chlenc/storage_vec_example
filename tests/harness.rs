use std::{str::FromStr};

use fuels::{prelude::*, tx::ContractId};
use rand::prelude::Rng;

// Load abi from json
abigen!(MyContract, "out/debug/storage_vec_example-abi.json");
abigen!(
    TokenContract,
    "tests/artefacts/token/token_contract-abi.json"
);

async fn create_wallet() -> WalletUnlocked {
    let mut wallets = launch_custom_provider_and_get_wallets(
        WalletsConfig::new(
            Some(1),             /* Single wallet */
            Some(1),             /* Single coin (UTXO) */
            Some(1_000_000_000), /* Amount per coin */
        ),
        None,
        None,
    )
    .await;
    wallets.pop().unwrap()
}

async fn get_contract_instance() -> (MyContract, WalletUnlocked) {
    let wallet = create_wallet().await;

    let id = Contract::deploy(
        "./out/debug/storage_vec_example.bin",
        &wallet,
        TxParameters::default(),
        StorageConfiguration::with_storage_path(Some(
            "./out/debug/storage_vec_example-storage_slots.json".to_string(),
        )),
    )
    .await
    .unwrap();

    let instance = MyContract::new(id.clone(), wallet.clone());

    (instance, wallet)
}

#[tokio::test]
async fn my_test() {
    let (instance, wallet) = get_contract_instance().await;

    let usdc_config = DeployTokenConfig {
        name: String::from("USD Coin"),
        symbol: String::from("USDC"),
        decimals: 6,
        mint_amount: 1,
    };
    let usdc_instance = get_token_contract_instance(&wallet, &usdc_config).await;
    let usdc_contarct_id = ContractId::from(usdc_instance.get_contract_id());

    let uni_config = DeployTokenConfig {
        name: String::from("Uniswap"),
        symbol: String::from("UNI"),
        decimals: 9,
        mint_amount: 1000,
    };
    let uni_instance = get_token_contract_instance(&wallet, &uni_config).await;
    let uni_contarct_id = ContractId::from(uni_instance.get_contract_id());

    let config = my_contract_mod::MarketConfiguration {
        base_token: usdc_contarct_id,
    };
    let assets = vec![
        my_contract_mod::AssetConfig {
            asset: ContractId::from_str(BASE_ASSET_ID.to_string().as_str())
                .expect("Cannot parse BASE_ASSET_ID to contract id"),
            decimals: 9,
        },
        my_contract_mod::AssetConfig {
            asset: usdc_contarct_id,
            decimals: usdc_config.decimals,
        },
        my_contract_mod::AssetConfig {
            asset: uni_contarct_id,
            decimals: uni_config.decimals,
        },
    ];
    let methods = instance.methods();

    methods.initialize(config, assets).call().await.unwrap();

    let _res = methods
        .get_asset_config_by_asset_id(usdc_contarct_id)
        .simulate()
        .await
        .unwrap()
        .value;

    let wallet2 = create_wallet().await;
    let usdc_asset_id = AssetId::from(*usdc_instance.get_contract_id().hash());
    let params = CallParameters::new(Some(1_000_000), Some(usdc_asset_id), None);
    let _res = instance
        .with_wallet(wallet2.clone())
        .unwrap()
        .methods()
        .supply_base()
        .call_params(params)
        .estimate_tx_dependencies(None)
        .await
        .unwrap()
        .call()
        .await
        .unwrap();
}

pub struct DeployTokenConfig {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub mint_amount: u64,
}

pub async fn get_token_contract_instance(
    wallet: &WalletUnlocked,
    deploy_config: &DeployTokenConfig,
) -> TokenContract {
    let mut name = deploy_config.name.clone();
    let mut symbol = deploy_config.symbol.clone();
    let decimals = deploy_config.decimals;

    let mut rng = rand::thread_rng();
    let salt = rng.gen::<[u8; 32]>();

    let id = Contract::deploy_with_parameters(
        "./tests/artefacts/token/token_contract.bin",
        &wallet,
        TxParameters::default(),
        StorageConfiguration::default(),
        Salt::from(salt),
    )
    .await
    .unwrap();

    let instance = TokenContract::new(id, wallet.clone());
    let methods = instance.methods();

    let mint_amount = parse_units(deploy_config.mint_amount, decimals);
    name.push_str(" ".repeat(32 - deploy_config.name.len()).as_str());
    symbol.push_str(" ".repeat(8 - deploy_config.symbol.len()).as_str());

    let config: token_contract_mod::Config = token_contract_mod::Config {
        name: fuels::core::types::SizedAsciiString::<32>::new(name).unwrap(),
        symbol: fuels::core::types::SizedAsciiString::<8>::new(symbol).unwrap(),
        decimals,
    };

    let _res = methods
        .initialize(config, mint_amount, Address::from(wallet.address()))
        .call()
        .await;
    let _res = methods.mint().append_variable_outputs(1).call().await;

    instance
}

pub fn parse_units(num: u64, decimals: u8) -> u64 {
    num * 10u64.pow(decimals as u32)
}
