# ClearMesh Core

A Rust crate with shared data models and local primitives for ClearMesh repositories.

---

## Where this crate fits

`clearmesh-core` is a library. It does not talk to any network. It does not upload or download chunks, manage authentication, query a database, or implement a CLI or server.

It provides the local building blocks that the ClearMesh CLI and API server both need: chunking file bytes, hashing content, encrypting and decrypting chunks, generating deterministic commit IDs, and working with shared structs.

If you are looking for the CLI tool, see [clearmesh-cli](https://github.com/clearmeshhq/clearmesh-cli).

---

## What this crate includes

- Fixed-size chunking and adaptive chunk sizing
- Streaming chunk processing via callback
- BLAKE3 hashing
- Chunk encryption and decryption (ChaCha20-Poly1305)
- Passphrase-to-key derivation (see [security notes](#security-notes))
- Deterministic commit ID generation
- Shared data model structs (`Commit`, `FileEntry`, `ChunkRef`)
- Type aliases for IDs (`UserId`, `OrgId`, `RepoId`, etc.)
- Slug normalization
- Permission roles and action checks

## What this crate does not include

- No network client
- No database layer
- No FUSE or VFS implementation
- No command-line interface
- No object storage client
- No web server or API handler
- No full repository sync engine

---

## Adding as a dependency

This crate is not published to crates.io. Add it via git:

```toml
[dependencies]
clearmesh-core = { git = "https://github.com/clearmeshhq/clearmesh-core" }
```

To pin to a specific commit:

```toml
clearmesh-core = { git = "https://github.com/clearmeshhq/clearmesh-core", rev = "abcdef1" }
```

---

## Quick example

```rust
use clearmesh_core::{
    adaptive_chunk_size, chunk_bytes, hash_bytes,
    demo_key_from_passphrase, encrypt_chunk, decrypt_chunk,
};

// Chunk a file
let data: Vec<u8> = std::fs::read("model.gguf").unwrap();
let chunk_size = adaptive_chunk_size(data.len() as u64);
let chunks = chunk_bytes(&data, chunk_size);

// Hash and encrypt each chunk
let key = demo_key_from_passphrase("my passphrase");
for (i, chunk) in chunks.iter().enumerate() {
    let hash = hash_bytes(chunk);
    let ciphertext = encrypt_chunk(&key, chunk).unwrap();
    println!("chunk {i}: hash={hash}, encrypted_len={}", ciphertext.len());
}
```

---

## Chunking

`chunk_bytes` splits a byte slice into fixed-size pieces. The last piece may be shorter.

```rust
use clearmesh_core::chunk_bytes;

let chunks = chunk_bytes(b"abcdefghij", 4);
// chunks == [b"abcd", b"efgh", b"ij"]
```

`adaptive_chunk_size` picks a chunk size based on file size:

| File size | Chunk size |
|---|---|
| < 64 MiB | 1 MiB |
| 64 MiB – 1 GiB | 4 MiB |
| > 1 GiB | 8 MiB |

```rust
use clearmesh_core::adaptive_chunk_size;

let size = adaptive_chunk_size(500 * 1024 * 1024); // 500 MiB file
assert_eq!(size, 4 * 1024 * 1024);                 // → 4 MiB chunks
```

---

## Streaming chunk processing

`process_chunks_streaming` calls a closure once per chunk. It returns the total number of chunks processed. If the callback returns an error, processing stops.

```rust
use clearmesh_core::process_chunks_streaming;

let data = b"abcdefghijklmnop";
let n = process_chunks_streaming(data, 4, |index, chunk| {
    println!("chunk {index}: {} bytes", chunk.len());
    Ok(())
}).unwrap();
// n == 4
```

**Note:** This function takes a `&[u8]`, so the full byte slice must already be in memory. It is "streaming" in the sense that your callback receives one chunk at a time without intermediate allocation of the full chunk list. If you need chunk-by-chunk I/O without reading the whole file first, you will need to implement that on top.

---

## Hashing

`hash_bytes` returns a BLAKE3 hex string.

```rust
use clearmesh_core::hash_bytes;

let h = hash_bytes(b"hello");
// 64-character lowercase hex string
```

The same bytes always produce the same hash. Hash values are used as chunk IDs when tracking what has already been uploaded.

---

## Encryption

Chunks are encrypted with ChaCha20-Poly1305. The nonce is derived deterministically from the key and the plaintext using `blake3::keyed_hash`, so encrypting the same bytes with the same key always produces the same ciphertext. The output format is `nonce (12 bytes) || ciphertext`.

```rust
use clearmesh_core::{demo_key_from_passphrase, encrypt_chunk, decrypt_chunk};

let key = demo_key_from_passphrase("my passphrase");
let ciphertext = encrypt_chunk(&key, b"private data").unwrap();
let plaintext = decrypt_chunk(&key, &ciphertext).unwrap();
assert_eq!(plaintext, b"private data");
```

`decrypt_chunk` returns an error if the ciphertext is too short or the authentication tag does not verify.

**Important:** `demo_key_from_passphrase` is a simple one-pass BLAKE3 key derivation. It is fast by design, which makes it weak against brute-force if used with short or guessable passphrases. See [security notes](#security-notes).

---

## Commits and IDs

`finalize_commit` takes a `Commit` struct, puts it into canonical form, and sets its `id` field to a BLAKE3 hash of the canonical JSON. Two commits with the same content, tree, and metadata will produce the same ID.

```rust
use clearmesh_core::{finalize_commit, models::{Commit, FileEntry, ChunkRef}};
use chrono::Utc;

let commit = Commit::new(
    "add dataset",
    "user@example.com",
    Utc::now(),
    vec![],  // parent commit IDs
    vec![FileEntry {
        path: "data.bin".into(),
        size: 1024,
        mode: 0o100644,
        chunk_size: 1024 * 1024,
        chunks: vec![ChunkRef {
            hash: "abc123...".into(),
            size: 1024,
            index: 0,
        }],
    }],
);

let finalized = finalize_commit(commit);
println!("commit id: {}", finalized.id); // 64-char hex
```

Canonicalization steps before hashing:
- Files sorted by path.
- Chunks within each file sorted by index.
- Parent IDs sorted lexicographically.
- Timestamp formatted as nanosecond RFC3339.

The `id` field starts as an empty string in `Commit::new`. It is only meaningful after calling `finalize_commit`.

---

## Models

The shared structs, all serializable with serde:

```rust
pub struct ChunkRef {
    pub hash: String,   // BLAKE3 hex, used as the chunk's storage key
    pub size: u64,      // byte count of this chunk
    pub index: u32,     // position within the file
}

pub struct FileEntry {
    pub path: String,          // relative path within the repo
    pub size: u64,             // total file size in bytes
    pub mode: u32,             // Unix file mode (e.g. 0o100644)
    pub chunks: Vec<ChunkRef>,
    pub chunk_size: usize,     // chunk size used when this file was chunked
}

pub struct Commit {
    pub id: String,                  // BLAKE3 hex; set by finalize_commit
    pub message: String,
    pub author_email: String,
    pub created_at: DateTime<Utc>,
    pub parent_ids: Vec<String>,
    pub files: Vec<FileEntry>,
}
```

`FileEntry.chunk_size` defaults to 1 MiB when deserializing older data that did not include the field.

---

## IDs and slugs

Type aliases for all ID types:

```rust
pub type UserId     = Uuid;
pub type OrgId      = Uuid;
pub type RepoId     = Uuid;
pub type SessionId  = Uuid;
pub type CommitId   = String;   // BLAKE3 hex
pub type BranchName = String;
pub type ChunkHash  = String;   // BLAKE3 hex
```

`slugify` converts a string to a lowercase, hyphen-separated slug:

```rust
use clearmesh_core::slugify;

slugify("  Clear Mesh V2!! ")  // → "clear-mesh-v2"
slugify("Models___2026")       // → "models-2026"
slugify("###")                 // → "untitled"
```

Rules: trim whitespace, lowercase, replace runs of non-alphanumeric characters with a single hyphen, strip trailing hyphens, return `"untitled"` if the result is empty.

`Slug::new(input)` wraps the result in a newtype that implements `Serialize`/`Deserialize`.

---

## Permissions

```rust
use clearmesh_core::{OrgRole, RepoRole, RepoAction, role_allows_action};
```

`OrgRole` variants: `Owner`, `Admin`, `Member`, `Viewer`

`RepoRole` and what each role is allowed to do:

| Role | Read | Write | Manage | Delete |
|---|---|---|---|---|
| `Admin` | yes | yes | yes | yes |
| `Maintainer` | yes | yes | yes | no |
| `Writer` | yes | yes | no | no |
| `Reader` | yes | no | no | no |

```rust
role_allows_action(RepoRole::Writer, RepoAction::Write)   // true
role_allows_action(RepoRole::Writer, RepoAction::Manage)  // false
role_allows_action(RepoRole::Reader, RepoAction::Read)    // true
```

---

## Design notes

**Deterministic encryption.** The nonce for each chunk is derived from the key and the plaintext (`blake3::keyed_hash(key, plaintext)`). This means encrypting the same chunk with the same key always gives the same ciphertext. This enables repo-scoped deduplication, but it also means equality of plaintext chunks encrypted under the same key can be observed by comparing ciphertexts or chunk identifiers. This is a known tradeoff for content-addressed encrypted storage.

**Key derivation is labeled demo.** `demo_key_from_passphrase` uses `blake3::derive_key`, a fast single-pass KDF with no memory cost or iteration count. It is appropriate for high-entropy programmatic keys. It is not appropriate for user-typed passwords without additional hardening applied before calling it. Calling applications should apply a memory-hard KDF such as Argon2 before deriving encryption keys from user-entered passphrases.

**`process_chunks_streaming` takes `&[u8]`.** The function is called "streaming" because it visits one chunk at a time and does not allocate the full chunk list. The caller still has to supply a fully loaded byte slice. Reading a large file in smaller I/O passes is outside the scope of this crate.

**Commit IDs are stable across builds.** The canonical form used for hashing does not change between builds of the same version. If the serialization format ever needs to change, it will require a new ID scheme and cannot be applied retroactively to existing commits.

---

## Security notes

- `demo_key_from_passphrase` is fast. A short or common passphrase is weak when put through a fast KDF. For user-entered passphrases, the calling code should apply a memory-hard function (such as Argon2) before passing the result here.
- Losing the passphrase means encrypted chunks cannot be decrypted. There is no recovery mechanism in this crate or in the hosted service.
- Deterministic encryption (same key + same plaintext = same ciphertext) is intentional but has ciphertext indistinguishability implications. See [design notes](#design-notes).
- The ChaCha20-Poly1305 authentication tag detects tampering. `decrypt_chunk` returns an error if the tag does not verify.
- This crate has not gone through a formal external security audit.
- ClearMesh is beta. Do not rely on it as your only copy of important data.

---

## Development

```bash
cargo fmt
cargo check
cargo test
cargo build --release
```

`target/` is in `.gitignore` and should not be committed.

---

## Testing

11 unit tests cover:

- Chunking stability and correctness
- Adaptive chunk size thresholds (small, medium, and large files)
- Streaming callback invocation count and ordering
- Encrypt/decrypt round-trip
- Deterministic encryption (same key and plaintext produce the same output)
- BLAKE3 hash output against a known vector
- Deterministic commit ID generation
- Slug normalization including edge cases
- Permission role checks

Run them with `cargo test`. The tests do not require network access or external services.

---

## Versioning and stability

This crate is at `0.1.0` and is beta. The public API may change. The chunking behavior, hash algorithm, and serialized commit format are stable for the current version, but no compatibility guarantees are made across versions yet.

---

## License

Apache-2.0. See [LICENSE](LICENSE).
