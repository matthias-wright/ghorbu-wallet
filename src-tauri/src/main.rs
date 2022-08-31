#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use dirs;
use std::{
    collections::HashMap,
    str::FromStr,
    sync::{Arc, Mutex},
};
use tauri::State;

use app::coin_selection;
use app::encryption::error::WrongPasswordError;
use app::networking::{self, fee::Fees};
use app::transactions::transaction::SimplifiedTransaction;
use app::{
    keys::{
        address::{Address, SimpleAddress},
        bip39,
        bip44::{MasterPrivateKey, MasterPublicKey},
    },
    transactions::transaction::Transaction,
};

static KEY_PATH: &'static str = ".bitcoinwallet";

#[derive(Default)]
struct Database(Arc<Mutex<HashMap<String, Vec<u8>>>>);

/**
 * Key creation
 */
#[tauri::command]
fn generate_mnemonic(db: State<'_, Database>) -> String {
    let mnemonic = bip39::generate_mnemonic(256, None);
    let mnemonic_encoded: Vec<u8> = bincode::serialize(&mnemonic).unwrap();
    db.0.lock()
        .unwrap()
        .insert("mnemonic".to_string(), mnemonic_encoded.clone());
    mnemonic.join(" ")
}

#[tauri::command]
fn send_passphrase(passphrase: String, db: State<'_, Database>) {
    let passphrase_encoded: Vec<u8> = bincode::serialize(&passphrase).unwrap();
    db.0.lock()
        .unwrap()
        .insert("passphrase".to_string(), passphrase_encoded);
}

#[tauri::command]
fn create_master_key(password: String, db: State<'_, Database>) {
    let passphrase_decoded = db.0.lock().unwrap().remove("passphrase").unwrap();
    let mnemonic_decoded = db.0.lock().unwrap().remove("mnemonic").unwrap();
    let passphrase: String = bincode::deserialize(&passphrase_decoded[..]).unwrap();
    let mnemonic: Vec<&str> = bincode::deserialize(&mnemonic_decoded[..]).unwrap();
    let seed = bip39::generate_seed(mnemonic, &passphrase);
    let master_private_key = MasterPrivateKey::create_from_seed(seed);
    let path = dirs::home_dir().unwrap().join(KEY_PATH);
    master_private_key.save(path, password).unwrap();
    let master_public_key = MasterPublicKey::create_from_key(&master_private_key);
    db.0.lock().unwrap().insert(
        "master_public_key".to_string(),
        master_public_key.serialize(),
    );
}

/**
 * Keys and transaction
 */
#[tauri::command]
fn does_master_key_exist() -> bool {
    dirs::home_dir().unwrap().join(KEY_PATH).exists()
}

#[tauri::command]
fn load_master_key(password: String, db: State<'_, Database>) -> Result<(), String> {
    let path = dirs::home_dir().unwrap().join(KEY_PATH);
    let master_private_key = MasterPrivateKey::load(path, password);
    match master_private_key {
        Ok(master_private_key) => {
            let master_public_key = MasterPublicKey::create_from_key(&master_private_key);
            db.0.lock().unwrap().insert(
                "master_public_key".to_string(),
                master_public_key.serialize(),
            );
            Ok(())
        }
        Err(e) => {
            if e.is::<std::io::Error>() {
                Err("io_error".to_string())
            } else if e.is::<WrongPasswordError>() {
                Err("wrong_password_error".to_string())
            } else {
                Err("other_error".to_string())
            }
        }
    }
}

#[tauri::command]
fn get_accounts_overview(db: State<'_, Database>) -> String {
    if !db.0.lock().unwrap().contains_key("master_public_key") {
        return "".to_string();
    }
    let master_pub_key =
        db.0.lock()
            .unwrap()
            .get("master_public_key")
            .unwrap()
            .clone();
    let master_pub_key = MasterPublicKey::load(&master_pub_key[..]);
    master_pub_key.jsonify()
}

#[tauri::command]
async fn get_account_balance(
    coin_type_index: u32,
    account_index: u32,
    db: State<'_, Database>,
) -> Result<u64, String> {
    let master_pub_key =
        db.0.lock()
            .unwrap()
            .get("master_public_key")
            .unwrap()
            .clone();
    let master_pub_key = MasterPublicKey::load(&master_pub_key[..]);
    let addresses = master_pub_key.get_all_addresses(coin_type_index, account_index);
    match networking::utxo::get_account_balance(addresses).await {
        Ok(balance) => Ok(balance),
        Err(_) => Err("io_error".to_string()),
    }
}

#[tauri::command]
async fn get_simple_transactions(
    coin_type_index: u32,
    account_index: u32,
    db: State<'_, Database>,
) -> Result<Vec<SimplifiedTransaction>, String> {
    let master_pub_key =
        db.0.lock()
            .unwrap()
            .get("master_public_key")
            .unwrap()
            .clone();
    let master_pub_key = MasterPublicKey::load(&master_pub_key[..]);
    let addresses = master_pub_key.get_all_addresses(coin_type_index, account_index);
    match networking::transaction::get_addresses_simple_transactions(addresses).await {
        Ok(simple_txs) => Ok(simple_txs),
        Err(_) => Err("io_error".to_string()),
    }
}

#[tauri::command]
async fn get_recommended_fees(coin_type_index: u32) -> Result<Fees, String> {
    match networking::fee::get_recommended_fees(coin_type_index).await {
        Ok(fees) => Ok(fees),
        Err(err) => Err(err.to_string()),
    }
}

/**
 * Account and address
 */
#[tauri::command]
fn create_new_account(
    coin_type_index: u32,
    password: String,
    db: State<'_, Database>,
) -> Result<(), String> {
    let path = dirs::home_dir().unwrap().join(KEY_PATH);
    let master_private_key = MasterPrivateKey::load(path.clone(), password.clone());
    match master_private_key {
        Ok(mut master_private_key) => {
            master_private_key.add_account(coin_type_index);
            master_private_key.save(path, password).unwrap();
            let master_public_key = MasterPublicKey::create_from_key(&master_private_key);
            db.0.lock().unwrap().insert(
                "master_public_key".to_string(),
                master_public_key.serialize(),
            );
            Ok(())
        }
        Err(e) => {
            if e.is::<std::io::Error>() {
                Err("io_error".to_string())
            } else if e.is::<WrongPasswordError>() {
                Err("wrong_password_error".to_string())
            } else {
                Err("other_error".to_string())
            }
        }
    }
}

#[tauri::command]
fn get_new_receive_address(
    coin_type_index: u32,
    account_index: u32,
    password: String,
    db: State<'_, Database>,
) -> Result<String, String> {
    let path = dirs::home_dir().unwrap().join(KEY_PATH);
    let master_private_key = MasterPrivateKey::load(path.clone(), password.clone());
    match master_private_key {
        Ok(mut master_private_key) => {
            let keypair =
                master_private_key.new_receive_keypair(coin_type_index, account_index, None);
            master_private_key.save(path, password).unwrap();
            let master_public_key = MasterPublicKey::create_from_key(&master_private_key);
            db.0.lock().unwrap().insert(
                "master_public_key".to_string(),
                master_public_key.serialize(),
            );
            let address = keypair.public_key.get_address();
            Ok(address.to_string())
        }
        Err(e) => {
            if e.is::<std::io::Error>() {
                Err("io_error".to_string())
            } else if e.is::<WrongPasswordError>() {
                Err("wrong_password_error".to_string())
            } else {
                Err("other_error".to_string())
            }
        }
    }
}

#[tauri::command]
fn get_all_receive_addresses(
    coin_type_index: u32,
    account_index: u32,
    db: State<'_, Database>,
) -> Vec<String> {
    let master_pub_key =
        db.0.lock()
            .unwrap()
            .get("master_public_key")
            .unwrap()
            .clone();
    let master_pub_key = MasterPublicKey::load(&master_pub_key[..]);
    master_pub_key
        .get_all_receive_addresses(coin_type_index, account_index)
        .iter()
        .map(|address| address.to_string())
        .collect()
}

#[tauri::command]
async fn get_all_receive_addresses_marked(
    coin_type_index: u32,
    account_index: u32,
    db: State<'_, Database>,
) -> Result<Vec<SimpleAddress>, String> {
    let master_pub_key =
        db.0.lock()
            .unwrap()
            .get("master_public_key")
            .unwrap()
            .clone();
    let master_pub_key = MasterPublicKey::load(&master_pub_key[..]);
    let addresses = master_pub_key.get_all_receive_addresses(coin_type_index, account_index);
    let simple_addr = networking::transaction::mark_addresses_as_used(addresses).await;
    match simple_addr {
        Ok(simple_addr) => Ok(simple_addr),
        Err(err) => Err(err.to_string()),
    }
}

#[tauri::command]
fn validate_address(address: String, coin_type_index: u32) -> Result<(), String> {
    match Address::from_str(&address) {
        Ok(address) => {
            let addr_coin_index = if address.testnet { 1 } else { 0 };
            if coin_type_index != addr_coin_index {
                return Err("Wrong address type".to_string());
            }
            Ok(())
        }
        Err(err) => Err(err.to_string()),
    }
}

/**
 * Send transaction
 */
#[tauri::command]
async fn send_transaction(
    coin_type_index: u32,
    account_index: u32,
    address: String,
    amount: u64,
    fee: u64,
    password: String,
    db: State<'_, Database>,
) -> Result<u64, String> {
    let path = dirs::home_dir().unwrap().join(KEY_PATH);
    let mut master_private_key = match MasterPrivateKey::load(path.clone(), password.clone()) {
        Ok(master_private_key) => master_private_key,
        Err(err) => {
            if err.is::<std::io::Error>() {
                return Err("io_error".to_string());
            } else if err.is::<WrongPasswordError>() {
                return Err("wrong_password_error".to_string());
            } else {
                return Err("other_error".to_string());
            }
        }
    };
    let keypairs = master_private_key.get_all_keypairs(coin_type_index, account_index);
    let utxos = match networking::utxo::get_keypairs_boxed_utxos(keypairs).await {
        Ok(utxos) => utxos,
        Err(_) => return Err("io_error".to_string()),
    };
    let selected_coins = match coin_selection::random_improve::select_coins(utxos, 1, amount, fee) {
        Ok(selected_coins) => selected_coins,
        Err(err) => return Err(err.to_string()),
    };
    let address = Address::from_str(&address).unwrap(); // already validated
    let mut targets = vec![(address, amount)];
    if let Some(change) = selected_coins.change {
        let change_keypair =
            master_private_key.new_change_keypair(coin_type_index, account_index, None);
        targets.push((change_keypair.public_key.get_address(), change));
    }
    let selected_utxos = selected_coins.selected_utxos;
    let tx = Transaction::create(&selected_utxos, targets);
    match tx {
        Ok(mut tx) => {
            tx.sign_all_inputs(&selected_utxos);
            match networking::transaction::send_transaction(tx, coin_type_index == 1).await {
                Ok(()) => {
                    // successfully broadcasted transaction
                    master_private_key.save(path, password).unwrap();
                    let master_public_key = MasterPublicKey::create_from_key(&master_private_key);
                    db.0.lock().unwrap().insert(
                        "master_public_key".to_string(),
                        master_public_key.serialize(),
                    );
                    let mut total_amount: u64 =
                        selected_utxos.iter().map(|utxo| utxo.utxo.value).sum();
                    if let Some(change) = selected_coins.change {
                        total_amount += change;
                    }
                    Ok(total_amount)
                }
                Err(_) => Err("send_tx_error".to_string()),
            }
        }
        Err(_) => Err("create_tx_error".to_string()),
    }
}

fn main() {
    tauri::Builder::default()
        .manage(Database(Default::default()))
        .invoke_handler(tauri::generate_handler![
            generate_mnemonic,
            send_passphrase,
            create_master_key,
            does_master_key_exist,
            load_master_key,
            get_accounts_overview,
            create_new_account,
            get_new_receive_address,
            get_all_receive_addresses,
            get_all_receive_addresses_marked,
            get_account_balance,
            get_simple_transactions,
            validate_address,
            get_recommended_fees,
            send_transaction,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
