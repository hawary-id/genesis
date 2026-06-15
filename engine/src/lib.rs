//! # Genesis Engine
//!
//! Genesis is an Artificial Civilization Engine built on data-oriented principles using Bevy ECS.
//! This crate contains the core simulation engine.

#![allow(clippy::field_reassign_with_default)]
#![allow(clippy::manual_range_contains)]

pub mod agent;
pub mod app;
pub mod config;
pub mod persistence;
pub mod rng;
pub mod testing;
pub mod time;
pub mod validation;
pub mod world;
