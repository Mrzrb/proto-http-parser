//! Generated API modules
//! 
//! This module contains the generated controllers and service traits
//! from the Protocol Buffer definitions. The code is generated at
//! build time using proto-http-parser-v2.

// Generated modules
pub mod book_service_controller;
pub mod book_service_service;
pub mod author_service_controller;
pub mod author_service_service;

// Re-export for convenience
pub use book_service_controller::*;
pub use book_service_service::*;
pub use author_service_controller::*;
pub use author_service_service::*;