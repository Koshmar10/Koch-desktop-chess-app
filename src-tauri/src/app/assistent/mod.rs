use std::time::Duration;

use tauri::{AppHandle, Manager};

use crate::app::{assistent::openai::ModelResponse, error::AppError};
use crate::db::{self, services::settings::SettingsService};

pub mod commands;
pub mod message;
mod openai;
