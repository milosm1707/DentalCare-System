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
        sender_id,
        receiver_id: req.receiver_id,
        content: req.content,
        message_type: req.message_type,
        is_read: false,
        created_at: Utc::now(),
    };

    let collection = state.db.collection::<ChatMessage>("messages");
    collection
        .insert_one(&message, None)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

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

    tracing::info!("WebSocket konekcija uspostavljena za korisnika: {}", user_id);

    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(&text) {
                    let chat_id = create_chat_id(&user_id, &ws_msg.receiver_id);

                    let message = ChatMessage {
                        id: None,
                        chat_id,
                        sender_id: user_id.clone(),
                        receiver_id: ws_msg.receiver_id,
                        content: ws_msg.content.clone(),
                        message_type: MessageType::Text,
                        is_read: false,
                        created_at: Utc::now(),
                    };

                    let collection = state.db.collection::<ChatMessage>("messages");
                    if let Err(e) = collection.insert_one(&message, None).await {
                        tracing::error!("Greška pri čuvanju poruke: {}", e);
                    }

                    let response = serde_json::json!({
                        "status": "delivered",
                        "content": ws_msg.content
                    });
                    if sender.send(Message::Text(response.to_string().into())).await.is_err() {
                        break;
                    }
                }
            }
            Ok(Message::Close(_)) => {
                tracing::info!("WebSocket zatvoren za: {}", user_id);
                break;
            }
            Err(e) => {
                tracing::error!("WebSocket greška: {}", e);
                break;
            }
            _ => {}
        }
    }
}

fn create_chat_id(user1: &str, user2: &str) -> String {
    let mut ids = vec![user1, user2];
    ids.sort();
    ids.join("_")
}