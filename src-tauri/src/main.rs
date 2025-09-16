#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use aes_gcm::{aead::{Aead, AeadCore, KeyInit, OsRng}, Aes256Gcm, Nonce, Key};
use argon2::{Argon2, Algorithm, Version, Params};
use argon2::password_hash::SaltString;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tauri::{Manager, Emitter};
use tauri_plugin_fs;
use ed25519_dalek::SigningKey;
use x25519_dalek::{StaticSecret, PublicKey};
use rand::{RngCore, rngs::OsRng as RandOsRng};
use base64::{Engine as _, engine::general_purpose};
use std::fs::{create_dir_all, write, read};
use std::sync::Mutex;
use std::path::PathBuf;
use nostr_sdk::{Client, Options, Keys, Kind, Filter, Tag, TagKind, ToBech32, FromBech32, EventBuilder, RelayPoolNotification};
use std::borrow::Cow;
use std::time::Duration;
use tokio::spawn;
use hex;
use tracing::{info, error, debug};

#[derive(Serialize, Deserialize, Clone)]
struct LoginData {
    username: String,
    ed25519_private: String,
    x25519_private: String,
    nostr_private: String,
}

#[derive(Serialize, Debug)]
struct Response {
    success: bool,
    message: String,
    data: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct EncryptedPayload {
    ciphertext: String,
    nonce: String,
}

#[derive(Default)]
struct AppState {
    login: Mutex<Option<LoginData>>,
    nostr_client: Mutex<Option<Client>>,
}

fn get_account_path(app_handle: &tauri::AppHandle, username: &str) -> Result<PathBuf, String> {
    info!("Getting account path for username: {}", username);
    let app_data = app_handle.path().app_data_dir().map_err(|e| {
        let err = format!("Failed to get app data dir: {}", e);
        error!("{}", err);
        err
    })?;
    let accounts_dir = app_data.join("accounts");
    create_dir_all(&accounts_dir).map_err(|e| {
        let err = format!("Failed to create accounts dir: {}", e);
        error!("{}", err);
        err
    })?;
    debug!("Account path: {:?}", accounts_dir.join(format!("{}.enc", username)));
    Ok(accounts_dir.join(format!("{}.enc", username)))
}

#[tauri::command]
async fn create_account(
    state: tauri::State<'_, AppState>,
    username: String,
    password: String,
    app_handle: tauri::AppHandle
) -> Result<Response, String> {
    info!("Creating account for username: {}", username);
    if username.is_empty() || password.is_empty() {
        error!("Create account failed: Username or password empty");
        return Err("Username and password cannot be empty".to_string());
    }
    let path = get_account_path(&app_handle, &username)?;
    if path.exists() {
        error!("Create account failed: Username already exists at {:?}", path);
        return Err("Username already exists on this device.".to_string());
    }
    let mut recovery_key_bytes = [0u8; 32];
    OsRng.fill_bytes(&mut recovery_key_bytes);
    let recovery_key = general_purpose::STANDARD.encode(&recovery_key_bytes);
    let mut csprng = RandOsRng;
    let ed25519_keypair = SigningKey::generate(&mut csprng);
    let x25519_secret = StaticSecret::random_from_rng(&mut csprng);
    let nostr_keys = Keys::generate();
    let login_data = LoginData {
        username: username.clone(),
        ed25519_private: hex::encode(ed25519_keypair.to_bytes()),
        x25519_private: hex::encode(x25519_secret.to_bytes()),
        nostr_private: nostr_keys.secret_key().to_bech32().map_err(|e| {
            let err = format!("Bech32 error: {}", e);
            error!("{}", err);
            err
        })?,
    };
    let plaintext = serde_json::to_string(&login_data).map_err(|e| {
        let err = e.to_string();
        error!("Serialization failed: {}", err);
        err
    })?;
    let salt = SaltString::generate(&mut OsRng);
    let salt_bytes = salt.as_ref().as_bytes();
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, Params::new(65536, 2, 1, Some(32)).unwrap());
    let mut key = [0u8; 32];
    argon2.hash_password_into(password.as_bytes(), salt_bytes, &mut key).map_err(|e| {
        let err = e.to_string();
        error!("Password hashing failed: {}", err);
        err
    })?;
    let cipher = Aes256Gcm::new(&Key::<Aes256Gcm>::from_slice(&key));
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher.encrypt(&nonce, plaintext.as_bytes())
        .map_err(|e| {
            let err = format!("Encryption failed: {}", e);
            error!("{}", err);
            err
        })?;
    let mut encrypted_data = Vec::new();
    encrypted_data.extend_from_slice(&(salt_bytes.len() as u16).to_be_bytes());
    encrypted_data.extend_from_slice(salt_bytes);
    encrypted_data.extend_from_slice(nonce.as_slice());
    encrypted_data.extend_from_slice(&ciphertext);
    write(&path, encrypted_data).map_err(|e| {
        let err = format!("File write failed: {}", e);
        error!("{}", err);
        err
    })?;
    state.login.lock().unwrap().replace(login_data.clone());
    key.fill(0);
    info!("Account created successfully for username: {}", username);
    Ok(Response {
        success: true,
        message: "Account created. Save your recovery key!".to_string(),
        data: Some(recovery_key),
    })
}

#[tauri::command]
async fn login(
    state: tauri::State<'_, AppState>,
    username: String,
    password: String,
    recovery_key: Option<String>,
    app_handle: tauri::AppHandle
) -> Result<Response, String> {
    info!("Attempting login for username: {}", username);
    debug!("Password provided: {}, Recovery key: {:?}", !password.is_empty(), recovery_key);
    if username.is_empty() {
        error!("Login failed: Username is empty");
        return Err("Username cannot be empty".to_string());
    }
    let path = get_account_path(&app_handle, &username)?;
    debug!("Checking if account exists at: {:?}", path);
    if !path.exists() {
        error!("Login failed: Account does not exist at {:?}", path);
        return Err("Account does not exist".to_string());
    }
    let encrypted_data = read(&path).map_err(|e| {
        let err = format!("File read failed: {}", e);
        error!("{}", err);
        err
    })?;
    debug!("Encrypted data length: {}", encrypted_data.len());
    if encrypted_data.len() < 2 {
        error!("Login failed: Invalid encrypted data length");
        return Err("Invalid encrypted data".to_string());
    }
    let salt_len = u16::from_be_bytes([encrypted_data[0], encrypted_data[1]]) as usize;
    debug!("Salt length: {}, Total data length: {}", salt_len, encrypted_data.len());
    if encrypted_data.len() < 2 + salt_len + 12 {
        error!("Login failed: Invalid encrypted data format");
        return Err("Invalid encrypted data format".to_string());
    }
    let salt_bytes = &encrypted_data[2..2 + salt_len];
    let nonce_bytes = &encrypted_data[2 + salt_len..2 + salt_len + 12];
    let ciphertext = &encrypted_data[2 + salt_len + 12..];
    let mut key_bytes = if let Some(recovery_key) = recovery_key {
        let recovery_key_bytes = general_purpose::STANDARD.decode(&recovery_key)
            .map_err(|e| {
                let err = format!("Invalid recovery key: {}", e);
                error!("{}", err);
                err
            })?;
        debug!("Recovery key bytes length: {}", recovery_key_bytes.len());
        if recovery_key_bytes.len() != 32 {
            error!("Login failed: Invalid recovery key length");
            return Err("Invalid recovery key length".to_string());
        }
        recovery_key_bytes
    } else {
        let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, Params::new(65536, 2, 1, Some(32)).unwrap());
        let mut key = [0u8; 32];
        argon2.hash_password_into(password.as_bytes(), salt_bytes, &mut key).map_err(|e| {
            let err = e.to_string();
            error!("{}", err);
            err
        })?;
        debug!("Password hashed successfully");
        key.to_vec()
    };
    let cipher = Aes256Gcm::new(&Key::<Aes256Gcm>::from_slice(&key_bytes));
    let nonce = Nonce::from_slice(nonce_bytes);
    debug!("Attempting decryption...");
    let plaintext = cipher.decrypt(nonce, ciphertext).map_err(|e| {
        let err = format!("Decryption failed: {}", e);
        error!("{}", err);
        err
    })?;
    let login_data: LoginData = serde_json::from_slice(&plaintext).map_err(|e| {
        let err = format!("Deserialization failed: {}", e);
        error!("{}", err);
        err
    })?;
    debug!("Login data deserialized: {:?}", login_data.username);
    state.login.lock().unwrap().replace(login_data.clone());
    key_bytes.fill(0);
    info!("Login successful for username: {}", username);
    Ok(Response {
        success: true,
        message: "Login successful".to_string(),
        data: None,
    })
}

#[tauri::command]
async fn debug_login_state(state: tauri::State<'_, AppState>) -> Result<Response, String> {
    info!("Checking login state");
    let login_data = state.login.lock().unwrap();
    if let Some(login) = login_data.as_ref() {
        info!("Login state: User {} is logged in", login.username);
        Ok(Response {
            success: true,
            message: format!("Logged in as {}", login.username),
            data: Some(login.username.clone()),
        })
    } else {
        error!("Login state: No user logged in");
        Err("No user logged in".to_string())
    }
}

#[tauri::command]
async fn get_user_info(
    state: tauri::State<'_, AppState>,
) -> Result<Response, String> {
    info!("Fetching user info");
    let login_data = {
        let guard = state.login.lock().unwrap();
        guard.as_ref().ok_or_else(|| {
            let err = "Not logged in".to_string();
            error!("{}", err);
            err
        })?.clone()
    };
    debug!("Login data retrieved: username={}", login_data.username);
    let nostr_keys = Keys::parse(&login_data.nostr_private).map_err(|e| {
        let err = e.to_string();
        error!("Keys parse failed: {}", err);
        err
    })?;
    debug!("Nostr keys parsed successfully");
    let x25519_secret = hex::decode(&login_data.x25519_private).map_err(|e| {
        let err = e.to_string();
        error!("X25519 decode failed: {}", err);
        err
    })?;
    let x25519_secret: [u8; 32] = x25519_secret.try_into().map_err(|_| {
        let err = "Invalid x25519 private key".to_string();
        error!("{}", err);
        err
    })?;
    let x25519_public = PublicKey::from(&StaticSecret::from(x25519_secret));
    debug!("X25519 public key generated");
    let response = Response {
        success: true,
        message: "User info retrieved".to_string(),
        data: Some(json!({
            "username": login_data.username,
            "nostr_public": nostr_keys.public_key().to_bech32().map_err(|e| {
                let err = e.to_string();
                error!("Bech32 encode failed: {}", err);
                err
            })?,
            "x25519_public": hex::encode(x25519_public.to_bytes())
        }).to_string()),
    };
    debug!("User info response: {:?}", response);
    info!("User info retrieved successfully");
    Ok(response)
}

#[tauri::command]
async fn init_nostr_client(
    state: tauri::State<'_, AppState>,
) -> Result<Response, String> {
    info!("Initializing Nostr client");
    let login_data = {
        let guard = state.login.lock().unwrap();
        guard.as_ref().ok_or_else(|| {
            let err = "Not logged in".to_string();
            error!("{}", err);
            err
        })?.clone()
    };
    debug!("Login data retrieved for Nostr client: username={}", login_data.username);
    let nostr_keys = Keys::parse(&login_data.nostr_private).map_err(|e| {
        let err = e.to_string();
        error!("Keys parse failed: {}", err);
        err
    })?;
    debug!("Nostr keys parsed successfully");
    let opts = Options::new().timeout(Duration::from_secs(30));
    let client = Client::with_opts(nostr_keys, opts);
    debug!("Nostr client created with options");
    if let Err(e) = client.add_relay("wss://relay.damus.io").await {
        let err = format!("Failed to add relay wss://relay.damus.io: {}", e);
        error!("{}", err);
        return Err(err);
    }
    debug!("Added relay wss://relay.damus.io");
    if let Err(e) = client.add_relay("wss://relay.nostr.io").await {
        let err = format!("Failed to add relay wss://relay.nostr.io: {}", e);
        error!("{}", err);
        return Err(err);
    }
    debug!("Added relay wss://relay.nostr.io");
    client.connect().await;
    debug!("Nostr client connected to relays");
    state.nostr_client.lock().unwrap().replace(client.clone());
    info!("Nostr client initialized successfully");
    Ok(Response { success: true, message: "Nostr client initialized".to_string(), data: None })
}

#[tauri::command]
async fn send_nostr_message(
    state: tauri::State<'_, AppState>,
    recipient_nostr_pub: String,
    recipient_x_pub: String,
    text: String,
) -> Result<Response, String> {
    info!("Sending Nostr message to {}", recipient_nostr_pub);
    let (sender_x_priv_hex, nostr_priv_hex) = {
        let guard = state.login.lock().unwrap();
        let login_data = guard.as_ref().ok_or_else(|| {
            let err = "Not logged in".to_string();
            error!("{}", err);
            err
        })?.clone();
        (login_data.x25519_private, login_data.nostr_private)
    };
    debug!("Retrieved login data for sending message");
    let sender_priv_bytes = hex::decode(&sender_x_priv_hex).map_err(|e| {
        let err = e.to_string();
        error!("Sender privkey decode failed: {}", err);
        err
    })?;
    let sender_priv: [u8; 32] = sender_priv_bytes.try_into().map_err(|_| {
        let err = "Invalid privkey".to_string();
        error!("{}", err);
        err
    })?;
    let sender_secret = StaticSecret::from(sender_priv);
    debug!("Sender X25519 secret generated");
    let recip_x_pub_bytes = hex::decode(&recipient_x_pub).map_err(|e| {
        let err = e.to_string();
        error!("Recipient pubkey decode failed: {}", err);
        err
    })?;
    let recip_x_pub: [u8; 32] = recip_x_pub_bytes.try_into().map_err(|_| {
        let err = "Invalid pubkey".to_string();
        error!("{}", err);
        err
    })?;
    let recip_pubkey = PublicKey::from(recip_x_pub);
    debug!("Recipient X25519 public key parsed");
    let shared_secret = sender_secret.diffie_hellman(&recip_pubkey);
    let key = Key::<Aes256Gcm>::from_slice(shared_secret.as_bytes());
    let cipher = Aes256Gcm::new(key);
    debug!("Shared secret and cipher created");
    let payload = json!({ "text": text });
    let plaintext = serde_json::to_string(&payload).map_err(|e| {
        let err = e.to_string();
        error!("Payload serialization failed: {}", err);
        err
    })?;
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher.encrypt(&nonce, plaintext.as_bytes()).map_err(|e| {
        let err = e.to_string();
        error!("Encryption failed: {}", err);
        err
    })?;
    let enc_payload = EncryptedPayload {
        ciphertext: general_purpose::STANDARD.encode(&ciphertext),
        nonce: general_purpose::STANDARD.encode(nonce.as_slice()),
    };
    let enc_json = serde_json::to_string(&enc_payload).map_err(|e| {
        let err = e.to_string();
        error!("Payload serialization failed: {}", err);
        err
    })?;
    debug!("Encrypted payload created: {:?}", enc_payload);
    let nostr_keys = Keys::parse(&nostr_priv_hex).map_err(|e| {
        let err = e.to_string();
        error!("Keys parse failed: {}", err);
        err
    })?;
    let recip_nostr_pub = nostr_sdk::PublicKey::from_bech32(&recipient_nostr_pub).map_err(|e| {
        let err = e.to_string();
        error!("Recipient Nostr pubkey parse failed: {}", err);
        err
    })?;
    debug!("Nostr keys and recipient public key parsed");
    let sender_x_pub = hex::encode(PublicKey::from(&sender_secret).to_bytes());
    let event = EventBuilder::new(Kind::EncryptedDirectMessage, enc_json, vec![Tag::public_key(recip_nostr_pub)])
        .add_tags(vec![Tag::custom(TagKind::Custom(Cow::Owned("x_pub".to_string())), vec![sender_x_pub])])
        .pow(16)
        .sign_with_keys(&nostr_keys)
        .map_err(|e| {
            let err = e.to_string();
            error!("Event signing failed: {}", err);
            err
        })?;
    debug!("Nostr event created and signed");
    let client = state.nostr_client.lock().unwrap().clone().ok_or_else(|| {
        let err = "No client".to_string();
        error!("{}", err);
        err
    })?;
    client.send_event(event).await.map_err(|e| {
        let err = e.to_string();
        error!("Send event failed: {}", err);
        err
    })?;
    info!("Message sent successfully to {}", recipient_nostr_pub);
    Ok(Response { success: true, message: "Sent via Nostr".to_string(), data: None })
}

#[tauri::command]
async fn receive_nostr_messages(
    state: tauri::State<'_, AppState>,
    window: tauri::Window,
) -> Result<Response, String> {
    info!("Starting to receive Nostr messages");
    let login_data = {
        let guard = state.login.lock().unwrap();
        guard.as_ref().ok_or_else(|| {
            let err = "Not logged in".to_string();
            error!("{}", err);
            err
        })?.clone()
    };
    debug!("Login data retrieved for receiving messages: username={}", login_data.username);
    let nostr_keys = Keys::parse(&login_data.nostr_private).map_err(|e| {
        let err = e.to_string();
        error!("Keys parse failed: {}", err);
        err
    })?;
    let our_pubkey = nostr_keys.public_key();
    debug!("Nostr public key retrieved: {:?}", our_pubkey);
    let client = state.nostr_client.lock().unwrap().clone().ok_or_else(|| {
        let err = "No client".to_string();
        error!("{}", err);
        err
    })?;
    debug!("Nostr client retrieved for subscription");
    let filter = Filter::new()
        .kind(Kind::EncryptedDirectMessage)
        .pubkey(our_pubkey)
        .limit(50);
    debug!("Subscribing with filter: {:?}", filter);
    client.subscribe(vec![filter], None).await.map_err(|e| {
        let err = e.to_string();
        error!("Subscribe failed: {}", err);
        err
    })?;
    debug!("Subscribed to Nostr events");
    let window_clone = window.clone();
    let client_clone = client.clone();
    let x_priv_hex_clone = login_data.x25519_private.clone();
    spawn(async move {
        let mut notifications = client_clone.notifications();
        debug!("Started listening for notifications");
        while let Ok(notif) = notifications.recv().await {
            debug!("Received notification: {:?}", notif);
            if let RelayPoolNotification::Event { event, .. } = notif {
                let ev = event.clone();
                let sender_npub = ev.pubkey.to_bech32().unwrap_or_default();
                let mut sender_x_pub_hex = None;
                for tag in ev.tags.iter() {
                    if let TagKind::Custom(ref kind) = tag.kind() {
                        if *kind == "x_pub" {
                            let tag_values = tag.clone().to_vec();
                            sender_x_pub_hex = tag_values.get(1).cloned();
                            break;
                        }
                    }
                }
                if let Some(sender_x_pub_hex) = sender_x_pub_hex {
                    debug!("Found x_pub tag: {}", sender_x_pub_hex);
                    if let Ok(x_priv_bytes) = hex::decode(&x_priv_hex_clone) {
                        if let Ok(x_priv_arr) = <[u8; 32]>::try_from(x_priv_bytes.as_slice()) {
                            let our_secret = StaticSecret::from(x_priv_arr);
                            if let Ok(sx_pub_bytes) = hex::decode(&sender_x_pub_hex) {
                                if let Ok(sx_pub_arr) = <[u8; 32]>::try_from(sx_pub_bytes.as_slice()) {
                                    let sender_pub = PublicKey::from(sx_pub_arr);
                                    let shared_secret = our_secret.diffie_hellman(&sender_pub);
                                    let key = Key::<Aes256Gcm>::from_slice(shared_secret.as_bytes());
                                    let cipher = Aes256Gcm::new(key);
                                    debug!("Shared secret and cipher created for decryption");
                                    if let Ok(enc_payload) = serde_json::from_str::<EncryptedPayload>(&ev.content) {
                                        let ct = match general_purpose::STANDARD.decode(&enc_payload.ciphertext) {
                                            Ok(v) => v,
                                            Err(e) => {
                                                error!("Ciphertext decode failed: {}", e);
                                                continue;
                                            }
                                        };
                                        let n_bytes = match general_purpose::STANDARD.decode(&enc_payload.nonce) {
                                            Ok(v) => v,
                                            Err(e) => {
                                                error!("Nonce decode failed: {}", e);
                                                continue;
                                            }
                                        };
                                        if n_bytes.len() != 12 {
                                            error!("Invalid nonce length: {}", n_bytes.len());
                                            continue;
                                        }
                                        let nonce = Nonce::from_slice(&n_bytes);
                                        if let Ok(pt) = cipher.decrypt(nonce, ct.as_slice()) {
                                            if let Ok(payload) = serde_json::from_slice::<serde_json::Value>(&pt) {
                                                if let Some(text) = payload["text"].as_str() {
                                                    debug!("Emitting new_message: sender_npub={}, timestamp={}", sender_npub, ev.created_at.as_u64());
                                                    let _ = window_clone.emit("new_message", json!({
                                                        "sender_npub": sender_npub,
                                                        "text": text,
                                                        "timestamp": ev.created_at.as_u64() as i64
                                                    }));
                                                } else {
                                                    error!("Payload text field missing");
                                                }
                                            } else {
                                                error!("Payload deserialization failed");
                                            }
                                        } else {
                                            error!("Decryption failed");
                                        }
                                    } else {
                                        error!("Encrypted payload parse failed");
                                    }
                                } else {
                                    error!("Sender x_pub decode failed");
                                }
                            } else {
                                error!("Sender x_pub hex decode failed");
                            }
                        } else {
                            error!("Private key decode failed");
                        }
                    } else {
                        error!("Private key hex decode failed");
                    }
                } else {
                    error!("No x_pub tag found");
                }
            }
        }
    });
    info!("Started listening for Nostr messages");
    Ok(Response {
        success: true,
        message: "Started listening for Nostr DMs. Listen for 'new_message' events in frontend.".to_string(),
        data: None,
    })
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("debug")
        .init();
    info!("Starting DumbChat Tauri app");
    tauri::Builder::default()
        .manage(AppState::default())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            create_account,
            login,
            debug_login_state,
            get_user_info,
            init_nostr_client,
            send_nostr_message,
            receive_nostr_messages
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}