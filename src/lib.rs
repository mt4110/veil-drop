//! Short-secret encryption and bearer-URL generation.
//!
//! A complete link contains the decryption key and must be handled as a secret.
//! See SECURITY.md for the hosting and transport trust boundaries.
//!
//! ```
//! use veil_drop::core::engine::generate_share_url;
//! let link = generate_share_url("synthetic example", None)?;
//! assert!(link.starts_with("https://mt4110.github.io/veil-drop/#payload="));
//! # Ok::<(), veil_drop::core::engine::EngineError>(())
//! ```

pub mod core;
pub mod interface;
