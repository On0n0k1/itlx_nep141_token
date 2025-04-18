use near_sdk::{json_types::U128, AccountId, NearToken};
use near_workspaces::{Account, Contract, DevNetwork, Worker};
use tokio::sync::OnceCell;

const INITIAL_BALANCE: NearToken = NearToken::from_near(30);
pub const ONE_YOCTO: NearToken = NearToken::from_yoctonear(1);

// static FUNGIBLE_TOKEN_CONTRACT_WASM: LazyLock<Vec<u8>> = LazyLock::new(|| {
//     let artifact = cargo_near_build::build(BuildOpts {
//         no_abi: true,
//         no_embed_abi: true,
//         ..Default::default()
//     })
//     .expect("Could not compile Fungible Token contract for tests");

//     std::fs::read(&artifact.path).unwrap_or_else(|err| {
//         panic!(
//             "Could not read Fungible Token WASM file from {}\nErr: {err}",
//             artifact.path,
//         )
//     })
// });

// static DEFI_CONTRACT_WASM: LazyLock<Vec<u8>> = LazyLock::new(|| {
//     let artifact_path = "tests/contracts/defi/res/defi.wasm";

//     std::fs::read(artifact_path).unwrap_or_else(|err| {
//         panic!(
//             "Could not read DeFi WASM file from {}\nErr: {err}",
//             artifact_path
//         )
//     })
// });
static FUNGIBLE_TOKEN_CONTRACT_WASM: OnceCell<Vec<u8>> = OnceCell::const_new();
static DEFI_CONTRACT_WASM: OnceCell<Vec<u8>> = OnceCell::const_new();

async fn ft_contract_wasm_init() -> Vec<u8> {
    near_workspaces::compile_project(".").await.unwrap()
}

async fn defi_contract_wasm_init() -> Vec<u8> {
    near_workspaces::compile_project("./tests/contracts/defi/")
        .await
        .unwrap()
}

pub async fn init_accounts(root: &Account) -> anyhow::Result<(Account, Account, Account, Account)> {
    // create accounts
    let alice = root
        .create_subaccount("alice")
        .initial_balance(INITIAL_BALANCE)
        .transact()
        .await?
        .into_result()?;
    let bob = root
        .create_subaccount("bob")
        .initial_balance(INITIAL_BALANCE)
        .transact()
        .await?
        .into_result()?;
    let charlie = root
        .create_subaccount("charlie")
        .initial_balance(INITIAL_BALANCE)
        .transact()
        .await?
        .into_result()?;
    let dave = root
        .create_subaccount("dave")
        .initial_balance(INITIAL_BALANCE)
        .transact()
        .await?
        .into_result()?;

    Ok((alice, bob, charlie, dave))
}

pub async fn init_contracts(
    worker: &Worker<impl DevNetwork>,
    initial_balance: U128,
    account: &Account,
) -> anyhow::Result<(Contract, Contract)> {
    let ft_contract_wasm = FUNGIBLE_TOKEN_CONTRACT_WASM
        .get_or_init(ft_contract_wasm_init)
        .await;
    let defi_contract_wasm = DEFI_CONTRACT_WASM
        .get_or_init(defi_contract_wasm_init)
        .await;
    let ft_contract = worker.dev_deploy(ft_contract_wasm).await?;

    let res = ft_contract
        .call("new_default_meta")
        .args_json((ft_contract.id(), initial_balance))
        .max_gas()
        .transact()
        .await?;
    assert!(res.is_success());

    let defi_contract = worker.dev_deploy(defi_contract_wasm).await?;

    let res = defi_contract
        .call("new")
        .args_json((ft_contract.id(),))
        .max_gas()
        .transact()
        .await?;
    assert!(res.is_success());

    let res = ft_contract
        .call("storage_deposit")
        .args_json((account.id(), Option::<bool>::None))
        .deposit(near_sdk::env::storage_byte_cost().saturating_mul(125))
        .max_gas()
        .transact()
        .await?;
    assert!(res.is_success());

    Ok((ft_contract, defi_contract))
}

pub async fn register_user(contract: &Contract, account_id: &AccountId) -> anyhow::Result<()> {
    let res = contract
        .call("storage_deposit")
        .args_json((account_id, Option::<bool>::None))
        .max_gas()
        .deposit(near_sdk::env::storage_byte_cost().saturating_mul(125))
        .transact()
        .await?;
    assert!(res.is_success());

    Ok(())
}
