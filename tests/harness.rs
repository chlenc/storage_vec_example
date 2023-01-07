use std::str::FromStr;

use fuels::{prelude::*, tx::ContractId};
use rand::prelude::Rng;

// Load abi from json
abigen!(MyContract, "out/debug/storage_vec_example-abi.json");
abigen!(
    TokenContract,
    "tests/artefacts/token/token_contract-abi.json"
);
async fn get_contract_instance() -> (MyContract, WalletUnlocked) {
    // Launch a local network and deploy the contract
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
    let wallet = wallets.pop().unwrap();

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

    let btc_config = DeployTokenConfig {
        name: String::from("Bitcoin"),
        symbol: String::from("BTC"),
        decimals: 8,
        mint_amount: 1,
    };
    let btc_instance = get_token_contract_instance(&wallet, &btc_config).await;
    let btc_contarct_id = ContractId::from(btc_instance.get_contract_id());

    let uni_config = DeployTokenConfig {
        name: String::from("Uniswap"),
        symbol: String::from("UNI"),
        decimals: 9,
        mint_amount: 1000,
    };
    let uni_instance = get_token_contract_instance(&wallet, &uni_config).await;
    let uni_contarct_id = ContractId::from(uni_instance.get_contract_id());

    let config = my_contract_mod::MarketConfiguration {
        foo: 100,
        bar: 200,
    };
    let assets = vec![
        my_contract_mod::AssetConfig {
            asset: ContractId::from_str(BASE_ASSET_ID.to_string().as_str())
                .expect("Cannot parse BASE_ASSET_ID to contract id"),
            decimals: 9,
            blablabla: 200000000000000,
        },
        my_contract_mod::AssetConfig {
            asset: btc_contarct_id,
            decimals: btc_config.decimals,
            blablabla: 200000000000000,
        },
        my_contract_mod::AssetConfig {
            asset: uni_contarct_id,
            decimals: uni_config.decimals,
            blablabla: 200000000000000,
        },
    ];
    let methods = instance.methods();

    methods.initialize(config, assets).call().await.unwrap();

    let res = methods
        .get_asset_config_by_asset_id(btc_contarct_id)
        .simulate()
        .await
        .unwrap()
        .value;
    println!("{:#?}", res);
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
