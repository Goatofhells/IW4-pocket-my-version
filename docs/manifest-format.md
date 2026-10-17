# Manifest schema 1
The bundled catalog is a JSON array of manifests. It intentionally contains no entries.
A manifest contains schema_version (1), installation (nonempty identifier), and files.
Each file requires a relative path, byte size, and independently established 64-digit SHA-256.
Never accept a manifest from the imported game archive as trusted validation data.
Catalog updates require reviewed provenance and a separately distributed app update.
Hash equality establishes byte identity only, never purchase, licensing, or Steam ownership.
ZIP extraction rejects traversal, symbolic links, duplicate case-insensitive paths and unsupported entry types; limits are 20,000 entries and 16 GiB uncompressed.
The extractor stages files in a new app-private directory; failures remove staging and preserve the previous import.
Loose selections are flattened; use ZIP for directories. No installation is currently supported.
