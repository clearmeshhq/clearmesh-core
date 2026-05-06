# ClearMesh Core

ClearMesh Core is the shared protocol, encryption, and chunk manifest crate used by ClearMesh clients.

It contains client-safe primitives for:

- commit, file, and chunk manifest structs
- deterministic commit IDs
- chunking helpers
- BLAKE3 hashing
- client-side chunk encryption and decryption
- shared permission enums

ClearMesh encrypted repositories use client-side encryption. Passphrases and derived keys should stay on the client and should not be sent to ClearMesh services.

Licensed under Apache-2.0.

## Development

```bash
cargo check
cargo test
```
