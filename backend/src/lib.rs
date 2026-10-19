//! AICTE APAAR student data & verification platform - modular monolith.
//!
//! Layering: `routes` -> `services` -> `repositories` -> database.
//! `domain` is pure and has no database access.

pub mod auth;
pub mod config;
pub mod domain;
pub mod error;
pub mod integrations;
pub mod repositories;
pub mod routes;
pub mod services;
pub mod state;
