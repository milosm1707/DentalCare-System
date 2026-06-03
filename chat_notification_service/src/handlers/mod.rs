use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    response::Response,
    Json,
};
use chrono::Utc;
use futures_util::{SinkExt, StreamExt};
use mongodb::bson::doc;
use tokio::sync::broadcast;

use crate::{
    errors::AppError,
    models::{
        Claims, Message as ChatMessage, MessageType,
        Notification, SendMessageRequest, WsMessage,
    },
    AppState,
};

pub async fn send_message(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    Json(req): Json<SendMessageRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let sender_id = claims.sub.clone();
    let chat_id = create_chat_id(&sender_id, &req.receiver_id);

    let message = ChatMessage {
        id: None,
        chat_id,
        sender_id: sender_id.clone(),
        receiver_id: req.receiver_id.clone(),
        content: req.content.clone(),
        message_type: req.message_type,
        is_read: false,
        created_at: Utc::now(),
    };

    let collection = state.db.collection::<ChatMessage>("messages");
    collection
        .insert_one(&message, None)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    // Pošalji poruku primaocu kroz WebSocket ako je online
    // Dohvati informacije o pošiljaocu da ih pošaljemo primaocu
    let ws_clients = state.ws_clients.read().await;
    if let Some(tx) = ws_clients.get(&req.receiver_id) {
        let ws_msg = serde_json::json!({
        "sender_id": sender_id,
        "sender_email": "",
        "content": req.content,
        "created_at": Utc::now().to_rfc3339()
    });
        let _ = tx.send(ws_msg.to_string());
    }

    Ok(Json(serde_json::json!({ "status": "sent" })))
}

pub async fn get_chat_history(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path(other_user_id): Path<String>,
) -> Result<Json<Vec<ChatMessage>>, AppError> {
    let my_id = claims.sub.clone();
    let chat_id = create_chat_id(&my_id, &other_user_id);

    let collection = state.db.collection::<ChatMessage>("messages");
    let mut cursor = collection
        .find(doc! { "chat_id": &chat_id }, None)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let mut messages = Vec::new();
    while cursor.advance().await.map_err(|e| AppError::DatabaseError(e.to_string()))? {
        let msg = cursor.deserialize_current()
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        messages.push(msg);
    }

    Ok(Json(messages))
}

pub async fn get_notifications(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
) -> Result<Json<Vec<Notification>>, AppError> {
    let collection = state.db.collection::<Notification>("notifications");
    let mut cursor = collection
        .find(doc! { "user_id": &claims.sub }, None)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    let mut notifications = Vec::new();
    while cursor.advance().await.map_err(|e| AppError::DatabaseError(e.to_string()))? {
        let notif = cursor.deserialize_current()
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        notifications.push(notif);
    }

    Ok(Json(notifications))
}

pub async fn create_notification(
    State(state): State<AppState>,
    Json(notif): Json<Notification>,
) -> Result<Json<serde_json::Value>, AppError> {
    let collection = state.db.collection::<Notification>("notifications");
    collection
        .insert_one(&notif, None)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(Json(serde_json::json!({ "status": "created" })))
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state, claims))
}

async fn handle_socket(socket: WebSocket, state: AppState, claims: Claims) {
    let (mut sender, mut receiver) = socket.split();
    let user_id = claims.sub.clone();

    tracing::info!("WebSocket konekcija: {}", user_id);

    // Registruj korisnika
    let (tx, mut rx) = broadcast::channel::<String>(100);
    {
        let mut clients = state.ws_clients.write().await;
        clients.insert(user_id.clone(), tx);
    }

    // Task koji šalje poruke ovom korisniku
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg.into())).await.is_err() {
                break;
            }
        }
    });

    // Čitaj poruke od klijenta (keepalive)
    let mut recv_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Close(_)) => break,
                Err(_) => break,
                _ => {}
            }
        }
    });

    // Čekaj da jedan od taskova završi
    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    }

    // Odjavi korisnika
    let mut clients = state.ws_clients.write().await;
    clients.remove(&user_id);
    tracing::info!("WebSocket zatvoren: {}", user_id);
}

fn create_chat_id(user1: &str, user2: &str) -> String {
    let mut ids = vec![user1, user2];
    ids.sort();
    ids.join("_")
}