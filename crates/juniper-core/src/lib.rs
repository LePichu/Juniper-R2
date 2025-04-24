// juniper-core/src/lib.rs
use std::error::Error;
use std::path::Path;
use std::process::{Child, Command};
use std::sync::Arc;
use tokio::sync::Mutex;
use sqlx::{postgres::PgPoolOptions, PgPool};
use warp::Filter;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Chat {
    pub id: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub title: String,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Message {
    pub id: Uuid,
    pub chat_id: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct MessageRequest {
    pub content: String,
}

pub struct OllamaInstance {
    process: Option<Child>,
    port: u16,
    model: String,
}

impl OllamaInstance {
    pub fn new(port: u16, model: String) -> Self {
        Self {
            process: None,
            port,
            model,
        }
    }

    pub fn start(&mut self) -> Result<(), Box<dyn Error>> {
        let mut cmd = Command::new("ollama")
            .args(&["serve", "--port", &self.port.to_string()])
            .spawn()?;
        
        self.process = Some(cmd);
        std::thread::sleep(std::time::Duration::from_secs(2));
        Ok(())
    }

    pub async fn query(&self, prompt: &str) -> Result<String, Box<dyn Error>> {
        let client = reqwest::Client::new();
        let res = client.post(&format!("http://localhost:{}/api/generate", self.port))
            .json(&serde_json::json!({
                "model": self.model,
                "prompt": prompt
            }))
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;
        
        let response = res["response"].as_str()
            .ok_or("Invalid response from Ollama")?
            .to_string();
        
        Ok(response)
    }
}

impl Drop for OllamaInstance {
    fn drop(&mut self) {
        if let Some(mut process) = self.process.take() {
            let _ = process.kill();
        }
    }
}

pub trait Runtime {
    async fn initialize(&mut self) -> Result<(), Box<dyn Error>>;
    async fn start(&self) -> Result<(), Box<dyn Error>>;
    async fn shutdown(&self) -> Result<(), Box<dyn Error>>;
}

pub struct JuniperRuntime {
    db_pool: Option<PgPool>,
    port: u16,
    ollama: Arc<Mutex<OllamaInstance>>,
}

impl JuniperRuntime {
    pub fn new(port: u16) -> Self {
        Self {
            db_pool: None,
            port,
            ollama: Arc::new(Mutex::new(OllamaInstance::new(11434, "llama2".to_string()))),
        }
    }

    async fn ensure_database_exists(&mut self) -> Result<(), Box<dyn Error>> {
        let db_path = "juniper.db";
        
        let postgres_url = if Path::new(db_path).exists() {
            "postgres://postgres:postgres@localhost/juniper"
        } else {
            let pool = PgPoolOptions::new()
                .max_connections(5)
                .connect("postgres://postgres:postgres@localhost/postgres")
                .await?;
                
            sqlx::query("CREATE DATABASE juniper")
                .execute(&pool)
                .await?;
                
            "postgres://postgres:postgres@localhost/juniper"
        };
        
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(postgres_url)
            .await?;
            
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS chats (
                id UUID PRIMARY KEY,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                title TEXT NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await?;
        
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS messages (
                id UUID PRIMARY KEY,
                chat_id UUID NOT NULL REFERENCES chats(id),
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                content TEXT NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await?;
        
        self.db_pool = Some(pool);
        
        Ok(())
    }
    
    async fn get_chats(&self) -> Result<Vec<Chat>, sqlx::Error> {
        let pool = self.db_pool.as_ref().unwrap();
        
        let chats = sqlx::query_as::<_, Chat>("SELECT * FROM chats ORDER BY created_at DESC")
            .fetch_all(pool)
            .await?;
            
        Ok(chats)
    }
    
    async fn create_chat(&self, title: &str) -> Result<Chat, sqlx::Error> {
        let pool = self.db_pool.as_ref().unwrap();
        let id = Uuid::new_v4();
        
        let chat = sqlx::query_as::<_, Chat>(
            "INSERT INTO chats (id, title) VALUES ($1, $2) RETURNING *"
        )
        .bind(id)
        .bind(title)
        .fetch_one(pool)
        .await?;
        
        Ok(chat)
    }
    
    async fn send_message(&self, chat_id: Uuid, content: &str) -> Result<Message, sqlx::Error> {
        let pool = self.db_pool.as_ref().unwrap();
        let id = Uuid::new_v4();
        
        let message = sqlx::query_as::<_, Message>(
            "INSERT INTO messages (id, chat_id, content) VALUES ($1, $2, $3) RETURNING *"
        )
        .bind(id)
        .bind(chat_id)
        .bind(content)
        .fetch_one(pool)
        .await?;
        
        Ok(message)
    }
}

impl Runtime for JuniperRuntime {
    async fn initialize(&mut self) -> Result<(), Box<dyn Error>> {
        self.ensure_database_exists().await?;
        
        let mut ollama = self.ollama.lock().await;
        ollama.start()?;
        
        Ok(())
    }
    
    async fn start(&self) -> Result<(), Box<dyn Error>> {
        let db_pool = self.db_pool.clone().unwrap();
        let db = Arc::new(Mutex::new(db_pool));
        let ollama = self.ollama.clone();
        
        let db_clone = db.clone();
        let chats_route = warp::path("chats")
            .and(warp::path::end())
            .and(warp::get())
            .and_then(move || {
                let db = db_clone.clone();
                async move {
                    let pool = db.lock().await;
                    let chats = sqlx::query_as::<_, Chat>("SELECT * FROM chats ORDER BY created_at DESC")
                        .fetch_all(&*pool)
                        .await;
                        
                    match chats {
                        Ok(result) => Ok(warp::reply::json(&result)),
                        Err(e) => Err(warp::reject::custom(e)),
                    }
                }
            });
            
        let db_clone = db.clone();
        let new_chat_route = warp::path("new")
            .and(warp::path::end())
            .and(warp::post())
            .and_then(move || {
                let db = db_clone.clone();
                async move {
                    let pool = db.lock().await;
                    let id = Uuid::new_v4();
                    let result = sqlx::query_as::<_, Chat>(
                        "INSERT INTO chats (id, title) VALUES ($1, $2) RETURNING *"
                    )
                    .bind(id)
                    .bind("New Chat")
                    .fetch_one(&*pool)
                    .await;
                    
                    match result {
                        Ok(chat) => Ok(warp::reply::json(&chat)),
                        Err(e) => Err(warp::reject::custom(e)),
                    }
                }
            });
            
        let db_clone = db.clone();
        let ollama_clone = ollama.clone();
        let message_route = warp::path("message")
            .and(warp::path::param::<Uuid>())
            .and(warp::path::end())
            .and(warp::post())
            .and(warp::body::json())
            .and_then(move |chat_id: Uuid, request: MessageRequest| {
                let db = db_clone.clone();
                let ollama = ollama_clone.clone();
                async move {
                    let pool = db.lock().await;
                    
                    let chat_exists = sqlx::query("SELECT 1 FROM chats WHERE id = $1")
                        .bind(chat_id)
                        .fetch_optional(&*pool)
                        .await;
                        
                    if let Err(e) = chat_exists {
                        return Err(warp::reject::custom(e));
                    }
                    
                    if chat_exists.unwrap().is_none() {
                        return Err(warp::reject::not_found());
                    }
                    
                    let id = Uuid::new_v4();
                    let result = sqlx::query_as::<_, Message>(
                        "INSERT INTO messages (id, chat_id, content) VALUES ($1, $2, $3) RETURNING *"
                    )
                    .bind(id)
                    .bind(chat_id)
                    .bind(&request.content)
                    .fetch_one(&*pool)
                    .await;
                    
                    if let Ok(message) = result {
                        let ollama_instance = ollama.lock().await;
                        match ollama_instance.query(&request.content).await {
                            Ok(response) => {
                                let response_id = Uuid::new_v4();
                                let ai_response = sqlx::query_as::<_, Message>(
                                    "INSERT INTO messages (id, chat_id, content) VALUES ($1, $2, $3) RETURNING *"
                                )
                                .bind(response_id)
                                .bind(chat_id)
                                .bind(response)
                                .fetch_one(&*pool)
                                .await;
                                
                                match ai_response {
                                    Ok(response_msg) => Ok(warp::reply::json(&response_msg)),
                                    Err(e) => Err(warp::reject::custom(e)),
                                }
                            },
                            Err(e) => {
                                let err_str = format!("Error from Ollama: {}", e);
                                Err(warp::reject::custom(err_str))
                            }
                        }
                    } else {
                        Err(warp::reject::custom(result.err().unwrap()))
                    }
                }
            });
            
        let routes = chats_route
            .or(new_chat_route)
            .or(message_route)
            .with(warp::cors().allow_any_origin());
            
        println!("Starting server on port {}", self.port);
        warp::serve(routes).run(([127, 0, 0, 1], self.port)).await;
        
        Ok(())
    }
    
    async fn shutdown(&self) -> Result<(), Box<dyn Error>> {
        if let Some(pool) = &self.db_pool {
            pool.close().await;
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_create_runtime() {
        let runtime = JuniperRuntime::new(8080);
        assert_eq!(runtime.port, 8080);
        assert!(runtime.db_pool.is_none());
    }
}