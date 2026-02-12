//! Celrix Cache - Client for Celrix high-performance cache
//!
//! Connects to Celrix on TCP port 6380

use anyhow::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use std::sync::Arc;

/// Celrix Cache client
#[derive(Clone)]
pub struct CelrixCache {
    connection_string: String,
    // Simple connection pool (mutex for now)
    // In production, use a proper pool like deadpool
    stream: Arc<Mutex<Option<TcpStream>>>,
}

impl CelrixCache {
    /// Create a new Celrix Cache client
    pub fn new(host: &str, port: u16) -> Self {
        Self {
            connection_string: format!("{}:{}", host, port),
            stream: Arc::new(Mutex::new(None)),
        }
    }

    /// Create from environment variable
    pub fn from_env() -> Self {
        let url = std::env::var("CELRIX_URL")
            .unwrap_or_else(|_| "127.0.0.1:6380".to_string());
        
        // Parse host:port
        let parts: Vec<&str> = url.split(':').collect();
        if parts.len() == 2 {
            Self::new(parts[0], parts[1].parse().unwrap_or(6380))
        } else {
            Self::new("127.0.0.1", 6380)
        }
    }

    /// Connect to Celrix
    async fn connect(&self) -> Result<TcpStream> {
        let stream = TcpStream::connect(&self.connection_string).await?;
        Ok(stream)
    }

    /// Set a key-value pair
    pub async fn set(&self, key: &str, value: &str) -> Result<()> {
        // Simple RESP-like or custom protocol simulation
        // "SET key value\r\n"
        let cmd = format!("SET {} {}\r\n", key, value);
        
        let mut guard = self.stream.lock().await;
        if guard.is_none() {
            *guard = Some(self.connect().await?);
        }
        
        if let Some(stream) = guard.as_mut() {
            stream.write_all(cmd.as_bytes()).await?;
            
            // Read response OK
            let mut buf = [0u8; 1024];
            let n = stream.read(&mut buf).await?;
            let response = String::from_utf8_lossy(&buf[..n]);
            
            if !response.starts_with("OK") {
                // Retry connection once
                *guard = Some(self.connect().await?);
                if let Some(retry_stream) = guard.as_mut() {
                    retry_stream.write_all(cmd.as_bytes()).await?;
                    let _ = retry_stream.read(&mut buf).await?;
                }
            }
        }
        
        Ok(())
    }

    /// Get a value by key
    pub async fn get(&self, key: &str) -> Result<Option<String>> {
        let cmd = format!("GET {}\r\n", key);
        
        let mut guard = self.stream.lock().await;
        if guard.is_none() {
            *guard = Some(self.connect().await?);
        }
        
        if let Some(stream) = guard.as_mut() {
            stream.write_all(cmd.as_bytes()).await?;
            
            let mut buf = [0u8; 1024];
            let n = stream.read(&mut buf).await?;
            let response = String::from_utf8_lossy(&buf[..n]);
            
            if response.trim().is_empty() || response.starts_with("ERR") || response.starts_with("NIL") {
                return Ok(None);
            }
            
            // Assuming response is the value directly for simplicity
            // In real RESP, it would be bulk string
            return Ok(Some(response.trim().to_string()));
        }

        Ok(None)
    }

    /// Check health by attempting connection
    pub async fn health(&self) -> Result<bool> {
        match TcpStream::connect(&self.connection_string).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}
