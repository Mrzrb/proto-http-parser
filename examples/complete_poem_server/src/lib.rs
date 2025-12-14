//! Complete Poem Server Example Library
//!
//! This demonstrates a working example of proto-http-parser-v2 integration
//! with poem-openapi for building REST APIs from Protocol Buffer definitions.

pub mod generated;
pub mod proto;
pub mod services;

pub use generated::*;
pub use proto::*;
pub use services::*;

