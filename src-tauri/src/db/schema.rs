/// Database schema version.
/// Incremented whenever `schema.sql` changes.
pub const SCHEMA_VERSION: &str = "1.0.0";

/// Catalog file extension.
pub const CATALOG_EXTENSION: &str = "praetorian";

/// Full schema SQL is loaded from `schema.sql` by the migrations module.
/// This module exists to expose schema constants and version info.
