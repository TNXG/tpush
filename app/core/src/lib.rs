#![allow(clippy::large_const_arrays)]

mod store;

use anyhow::{Context, Result};
use chrono::Utc;
use jni::Outcome;
use jni::errors::LogErrorAndDefault;
use jni::objects::{Global, JClass, JObject, JString};
use jni::signature::{MethodSignature, RuntimeMethodSignature};
use jni::strings::JNIString;
use jni::sys::jboolean;
use jni::{Env, EnvUnowned, JavaVM};
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use store::{MessageRecord, MessageStore};

uniffi::include_scaffolding!("tpush_core");

static RUNTIME: OnceCell<Arc<TpushRuntime>> = OnceCell::new();

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub title: String,
    pub content: String,
    pub payload: String,
    pub kind: String,
    pub received_at: String,
    pub read: bool,
}

struct TpushRuntime {
    store: MessageStore,
    device_id: Mutex<String>,
    java_vm: Mutex<Option<JavaVM>>,
    application_context: Mutex<Option<Global<JObject<'static>>>>,
}

#[derive(Debug, Deserialize)]
struct RealtimeMessage {
    id: String,
    title: String,
    content: String,
    #[serde(default)]
    payload: serde_json::Value,
    #[serde(default = "default_message_kind")]
    kind: String,
    #[serde(rename = "createdAt")]
    created_at: Option<String>,
}

pub fn initialize(database_path: String, server_base_url: String) {
    if let Err(error) = initialize_inner(database_path, server_base_url) {
        eprintln!("TPush initialize failed: {error:?}");
    }
}

pub fn get_messages() -> Vec<Message> {
    runtime()
        .and_then(|runtime| runtime.store.get_messages().ok())
        .unwrap_or_default()
        .into_iter()
        .map(Message::from)
        .collect()
}

pub fn mark_read(id: String) {
    if let Some(runtime) = runtime()
        && let Err(error) = runtime.store.mark_read(&id)
    {
        eprintln!("TPush mark_read failed: {error:?}");
    }
}

pub fn delete_message(id: String) {
    if let Some(runtime) = runtime()
        && let Err(error) = runtime.store.delete_message(&id)
    {
        eprintln!("TPush delete_message failed: {error:?}");
    }
}

pub fn get_device_id() -> String {
    runtime()
        .map(|runtime| runtime.device_id.lock().unwrap().clone())
        .unwrap_or_default()
}

pub fn clear_all() {
    if let Some(runtime) = runtime()
        && let Err(error) = runtime.store.clear_all()
    {
        eprintln!("TPush clear_all failed: {error:?}");
    }
}

fn initialize_inner(database_path: String, server_base_url: String) -> Result<()> {
    let store = MessageStore::open(database_path)?;
    let device_id = store.get_or_create_device_id()?;
    let _ = server_base_url;
    let runtime = TpushRuntime {
        store,
        device_id: Mutex::new(device_id),
        java_vm: Mutex::new(None),
        application_context: Mutex::new(None),
    };
    let _ = RUNTIME.set(Arc::new(runtime));
    Ok(())
}

fn runtime() -> Option<Arc<TpushRuntime>> {
    RUNTIME.get().cloned()
}

fn persist_message(
    id: String,
    kind: String,
    title: String,
    content: String,
    payload: String,
    received_at: String,
) -> Result<()> {
    let runtime = runtime().context("TPush is not initialized")?;
    let message = MessageRecord {
        id,
        title,
        content,
        payload,
        kind,
        received_at,
        read: false,
    };
    runtime.store.insert_message(&message)?;
    Ok(())
}

fn persist_realtime_message(message_json: String) -> Result<()> {
    let message: RealtimeMessage = serde_json::from_str(&message_json)?;
    persist_message(
        message.id,
        message.kind,
        message.title,
        message.content,
        message.payload.to_string(),
        message
            .created_at
            .unwrap_or_else(|| Utc::now().to_rfc3339()),
    )
}

fn default_message_kind() -> String {
    "server_push".to_owned()
}

fn jstring_to_string(env: &mut Env<'_>, value: JString<'_>) -> String {
    value.try_to_string(env).unwrap_or_default()
}

fn context_files_database_path(env: &mut Env<'_>, context: &JObject<'_>) -> Result<String> {
    let sig = RuntimeMethodSignature::from_str("()Ljava/io/File;")?;
    let ms = MethodSignature::from(&sig);
    let files_dir = env
        .call_method(context, JNIString::from("getFilesDir"), &ms, &[])?
        .l()?;

    let sig = RuntimeMethodSignature::from_str("()Ljava/lang/String;")?;
    let ms = MethodSignature::from(&sig);
    let absolute_path = env
        .call_method(files_dir, JNIString::from("getAbsolutePath"), &ms, &[])?
        .l()?;

    Ok(format!(
        "{}/tpush.sqlite",
        jstring_to_string(env, unsafe {
            JString::from_raw(env, absolute_path.into_raw())
        })
    ))
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_moe_tnxg_push_core_Bridge_nativeInit<'local>(
    mut unowned_env: EnvUnowned<'local>,
    _class: JClass<'local>,
    context: JObject<'local>,
    server_base_url: JString<'local>,
) -> jboolean {
    unowned_env
        .with_env(|env| -> jni::errors::Result<_> {
            let database_path = match context_files_database_path(env, &context) {
                Ok(path) => path,
                Err(error) => {
                    eprintln!("TPush database path failed: {error:?}");
                    return Ok(false);
                }
            };
            let server_base_url = jstring_to_string(env, server_base_url);
            initialize(database_path, server_base_url);

            if let Some(runtime) = runtime() {
                if let Ok(java_vm) = env.get_java_vm() {
                    *runtime.java_vm.lock().unwrap() = Some(java_vm);
                }
                if let Ok(global_context) = env.new_global_ref(context) {
                    *runtime.application_context.lock().unwrap() = Some(global_context);
                }
            }

            Ok(true)
        })
        .resolve::<LogErrorAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_moe_tnxg_push_core_Bridge_nativeGetDeviceId<'local>(
    mut unowned_env: EnvUnowned<'local>,
    _class: JClass<'local>,
) -> jni::sys::jstring {
    let outcome = unowned_env
        .with_env(|env| -> jni::errors::Result<_> {
            Ok(env.new_string(get_device_id())?.into_raw())
        })
        .into_outcome();
    match outcome {
        Outcome::Ok(ptr) => ptr,
        Outcome::Err(e) => {
            eprintln!("TPush nativeGetDeviceId error: {e}");
            std::ptr::null_mut()
        }
        Outcome::Panic(_) => {
            eprintln!("TPush nativeGetDeviceId panic");
            std::ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_moe_tnxg_push_core_Bridge_nativeGetMessagesJson<'local>(
    mut unowned_env: EnvUnowned<'local>,
    _class: JClass<'local>,
) -> jni::sys::jstring {
    let outcome = unowned_env
        .with_env(|env| -> jni::errors::Result<_> {
            let messages_json =
                serde_json::to_string(&get_messages()).unwrap_or_else(|_| "[]".to_owned());
            Ok(env.new_string(messages_json)?.into_raw())
        })
        .into_outcome();
    match outcome {
        Outcome::Ok(ptr) => ptr,
        Outcome::Err(e) => {
            eprintln!("TPush nativeGetMessagesJson error: {e}");
            std::ptr::null_mut()
        }
        Outcome::Panic(_) => {
            eprintln!("TPush nativeGetMessagesJson panic");
            std::ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_moe_tnxg_push_core_Bridge_nativeMarkRead<'local>(
    mut unowned_env: EnvUnowned<'local>,
    _class: JClass<'local>,
    id: JString<'local>,
) {
    unowned_env
        .with_env(|env| -> jni::errors::Result<_> {
            let id = jstring_to_string(env, id);
            if !id.is_empty() {
                mark_read(id);
            }
            Ok(())
        })
        .resolve::<LogErrorAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_moe_tnxg_push_core_Bridge_nativeDeleteMessage<'local>(
    mut unowned_env: EnvUnowned<'local>,
    _class: JClass<'local>,
    id: JString<'local>,
) {
    unowned_env
        .with_env(|env| -> jni::errors::Result<_> {
            let id = jstring_to_string(env, id);
            if !id.is_empty() {
                delete_message(id);
            }
            Ok(())
        })
        .resolve::<LogErrorAndDefault>()
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_moe_tnxg_push_core_Bridge_nativeClearAll<'local>(
    _env: EnvUnowned<'local>,
    _class: JClass<'local>,
) {
    clear_all();
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_moe_tnxg_push_core_Bridge_nativeIngestRealtimeMessage<'local>(
    mut unowned_env: EnvUnowned<'local>,
    _class: JClass<'local>,
    message_json: JString<'local>,
) {
    unowned_env
        .with_env(|env| -> jni::errors::Result<_> {
            let message_json = jstring_to_string(env, message_json);
            if message_json.is_empty() {
                return Ok(());
            }
            if let Err(error) = persist_realtime_message(message_json) {
                eprintln!("TPush realtime message persist failed: {error:?}");
            }
            Ok(())
        })
        .resolve::<LogErrorAndDefault>()
}

impl From<MessageRecord> for Message {
    fn from(record: MessageRecord) -> Self {
        Self {
            id: record.id,
            title: record.title,
            content: record.content,
            payload: record.payload,
            kind: record.kind,
            received_at: record.received_at,
            read: record.read,
        }
    }
}
