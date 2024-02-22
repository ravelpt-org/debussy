use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct ContainerOptions {
    pub image: String,
    pub host_config: HostConfig,
    pub tty: bool,
    pub attach_stdin: bool,
    pub attach_stdout: bool,
    pub attach_stderr: bool,
    pub open_stdin: bool,
    pub stdin_once: bool,
    pub env: Option<Vec<String>>,
    pub volumes: Option<HashMap<String, HashMap<String, String>>>,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct HostConfig {
    pub binds: Option<Vec<String>>,
    pub auto_remove: bool,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct CreateContainerSuccessResponse {
    pub id: String,
    pub warnings: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct DockerApiError {
    pub message: String,
}

#[derive(Debug)]
pub enum DockerErrors {
    CreateContainerError,
    StartContainerError,
    ContainerAlreadyStarted,
}

impl std::fmt::Display for DockerErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CreateContainerError => write!(f, "Error creating container"),
            Self::ContainerAlreadyStarted => write!(f, "Container already started"),
            Self::StartContainerError => write!(f, "Error starting container"),
        }
    }
}

pub async fn create_container(
    container_options: ContainerOptions,
    name: String,
    url: String,
) -> Result<String> {
    let json_data = serde_json::to_string(&container_options)?;

    let client = Client::new();

    let response = client
        .post(format!("{}/containers/create?name={}", url, name))
        .header("Content-Type", "application/json")
        .body(json_data)
        .send()
        .await?;

    return if response.status().is_success() {
        println!("Container '{}' created successfully!", name);
        Ok(response.json::<CreateContainerSuccessResponse>().await?.id)
    } else {
        println!("Failed to create container: {}", response.status());
        let error = response.json::<DockerApiError>().await?.message;
        Err(anyhow!(DockerErrors::CreateContainerError).context(error))
    };
}

pub async fn start_container(name: String, url: String) -> Result<()> {
    let client = Client::new();

    let response = client
        .post(format!("{}/containers/{}/start", url, name))
        .header("Content-Type", "application/json")
        .send()
        .await?;

    return if response.status().is_success() {
        println!("Container '{}' started successfully!", name);
        Ok(())
    } else if response.status().is_redirection() {
        println!("Failed to start container: {}", response.status());
        Err(anyhow!(DockerErrors::ContainerAlreadyStarted))
    } else {
        println!("Failed to start container: {}", response.status());
        let error = response.json::<DockerApiError>().await?.message;
        Err(anyhow!(DockerErrors::StartContainerError).context(error))
    };
}
