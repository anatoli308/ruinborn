//! AI subsystems for NPC behaviour.
//!
//! - [`goap`] — Goal-Oriented Action Planning (per-agent state machine
//!   driven by JSON-authored agents).
//! - [`boids`] — Reynolds flocking steering used as a movement layer
//!   underneath GOAP's `move_to_target` behaviour.
//!
//! Future modules (EANN reactive layer, squad coordination) will live
//! as siblings here.

pub mod boids;
pub mod goap;
