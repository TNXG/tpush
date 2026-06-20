mod store;

use anyhow::{Context, Result};
use chrono::Utc;
use jni::objects::{GlobalRef, JClass, JObject, JString};
use jni::sys::{jboolean, JNI_TRUE};
use jni::{JNIEnv, JavaVM};
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
    application_context: Mutex<Option<GlobalRef>>,
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
    if let Some(runtime) = runtime() {
        if let Err(error) = runtime.store.mark_read(&id) {
            eprintln!("TPush mark_read failed: {error:?}");
        }
    }
}

pub fn delete_message(id: String) {
    if let Some(runtime) = runtime() {
        if let Err(error) = runtime.store.delete_message(&id) {
            eprintln!("TPush delete_message failed: {error:?}");
        }
    }
}

pub fn get_device_id() -> String {
    runtime()
        .map(|runtime| runtime.device_id.lock().unwrap().clone())
        .unwrap_or_default()
}

pub fn clear_all() {
    if let Some(runtime) = runtime() {
        if let Err(error) = runtime.store.clear_all() {
            eprintln!("TPush clear_all failed: {error:?}");
        }
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

fn jstring_to_string(env: &mut JNIEnv<'_>, value: JString<'_>) -> String {
    env.get_string(&value)
        .map(|java_string| java_string.to_string_lossy().into_owned())
        .unwrap_or_default()
}

fn context_files_database_path(env: &mut JNIEnv<'_>, context: &JObject<'_>) -> Result<String> {
    let files_dir = env
        .call_method(context, "getFilesDir", "()Ljava/io/File;", &[])?
        .l()?;
    let absolute_path = env
        .call_method(files_dir, "getAbsolutePath", "()Ljava/lang/String;", &[])?
        .l()?;
    Ok(format!(
        "{}/tpush.sqlite",
        jstring_to_string(env, JString::from(absolute_path))
    ))
}

#[no_mangle]
pub extern "system" fn Java_moe_tnxg_push_core_Bridge_nativeInit(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    context: JObject<'_>,
    server_base_url: JString<'_>,
) -> jboolean {
    let database_path = match context_files_database_path(&mut env, &context) {
        Ok(path) => path,
        Err(error) => {
            eprintln!("TPush database path failed: {error:?}");
            return 0;
        }
    };
    let server_base_url = jstring_to_string(&mut env, server_base_url);
    initialize(database_path, server_base_url);

    if let Some(runtime) = runtime() {
        if let Ok(java_vm) = env.get_java_vm() {
            *runtime.java_vm.lock().unwrap() = Some(java_vm);
        }
        if let Ok(global_context) = env.new_global_ref(&context) {
            *runtime.application_context.lock().unwrap() = Some(global_context);
        }
    }

    JNI_TRUE
}

#[no_mangle]
pub extern "system" fn Java_moe_tnxg_push_core_Bridge_nativeGetDeviceId(
    env: JNIEnv<'_>,
    _class: JClass<'_>,
) -> jni::sys::jstring {
    env.new_string(get_device_id())
        .map(|value| value.into_raw())
        .unwrap_or(std::ptr::null_mut())
}

#[no_mangle]
pub extern "system" fn Java_moe_tnxg_push_core_Bridge_nativeGetMessagesJson(
    env: JNIEnv<'_>,
    _class: JClass<'_>,
) -> jni::sys::jstring {
    let messages_json = serde_json::to_string(&get_messages()).unwrap_or_else(|_| "[]".to_owned());
    env.new_string(messages_json)
        .map(|value| value.into_raw())
        .unwrap_or(std::ptr::null_mut())
}

#[no_mangle]
pub extern "system" fn Java_moe_tnxg_push_core_Bridge_nativeMarkRead(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    id: JString<'_>,
) {
    let id = jstring_to_string(&mut env, id);
    if !id.is_empty() {
        mark_read(id);
    }
}

#[no_mangle]
pub extern "system" fn Java_moe_tnxg_push_core_Bridge_nativeDeleteMessage(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    id: JString<'_>,
) {
    let id = jstring_to_string(&mut env, id);
    if !id.is_empty() {
        delete_message(id);
    }
}

#[no_mangle]
pub extern "system" fn Java_moe_tnxg_push_core_Bridge_nativeClearAll(
    _env: JNIEnv<'_>,
    _class: JClass<'_>,
) {
    clear_all();
}

#[no_mangle]
pub extern "system" fn Java_moe_tnxg_push_core_Bridge_nativeIngestRealtimeMessage(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    message_json: JString<'_>,
) {
    let message_json = jstring_to_string(&mut env, message_json);
    if message_json.is_empty() {
        return;
    }
    if let Err(error) = persist_realtime_message(message_json) {
        eprintln!("TPush realtime message persist failed: {error:?}");
    }
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
