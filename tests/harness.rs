use std::str::FromStr;

use fuels::{prelude::*, tx::ContractId};
use rand::prelude::Rng;

// Load abi from json
abigen!(MyContract, "out/debug/storage_vec_example-abi.json");
abigen!(
    TokenContract,
    "tests/artefacts/token/token_contract-abi.json"
);

async fn get_contract_instance() -> (MyContract, Vec<WalletUnlocked>) {
    let wallets = launch_custom_provider_and_get_wallets(
        WalletsConfig::new(
            Some(2),             /* Single wallet */
            Some(1),             /* Single coin (UTXO) */
            Some(1_000_000_000), /* Amount per coin */
        ),
        None,
        None,
    )
    .await;

    let id = Contract::deploy(
        "./out/debug/storage_vec_example.bin",
        &wallets[0],
        TxParameters::default(),
        StorageConfiguration::with_storage_path(Some(
            "./out/debug/storage_vec_example-storage_slots.json".to_string(),
        )),
    )
    .await
    .unwrap();

    let instance = MyContract::new(id.clone(), wallets[0].clone());

    (instance, wallets)
}

#[tokio::test]
async fn my_test() {
    let (instance, wallets) = get_contract_instance().await;
    let wallet = wallets[0].clone();
    let empty_wallet = wallets[1].clone();

    let usdc_config = DeployTokenConfig {
        name: String::from("USD Coin"),
        symbol: String::from("USDC"),
        decimals: 6,
        mint_amount: 10000,
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

    let balances = wallet.get_balances().await.unwrap();
    println!("{:#?}\n", balances);

    // usdc_instance
    //     // .with_wallet(empty_wallet.clone())
    //     // .as_ref()
    //     // .unwrap()
    //     .methods()
    //     .mint()
    //     .append_variable_outputs(1)
    //     .call()
    //     .await
    //     .unwrap();

    let balances = wallet.get_balances().await.unwrap();
    println!("{:#?}\n", balances);

    let usdc_asset_id = AssetId::from(*usdc_instance.get_contract_id().hash());
    let params = CallParameters::new(Some(1_000_000), Some(usdc_asset_id), None);
    let _res = instance
        // .with_wallet(empty_wallet.clone())
        // .unwrap()
        .methods()
        .supply_base()
        .call_params(params)
        .estimate_tx_dependencies(None)
        .await
        .unwrap()
        .call()
        .await
        .unwrap();

    let balances = wallet.get_balances().await.unwrap();
    println!("{:#?}\n", balances);
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
    // let _res = methods.mint().append_variable_outputs(1).call().await;

    instance
}

pub fn parse_units(num: u64, decimals: u8) -> u64 {
    num * 10u64.pow(decimals as u32)
}
