pub fn chunk_bytes(bytes: &[u8], chunk_size: usize) -> Vec<Vec<u8>> {
    assert!(chunk_size > 0, "chunk_size must be greater than zero");
    bytes
        .chunks(chunk_size)
        .map(|chunk| chunk.to_vec())
        .collect()
}

/// Adaptive chunk size based on file size
pub fn adaptive_chunk_size(file_size: u64) -> usize {
    match file_size {
        0..=67_108_863 => 1024 * 1024,                 // < 64 MiB: 1 MiB
        67_108_864..=1_073_741_823 => 4 * 1024 * 1024, // 64 MiB - 1 GiB: 4 MiB
        _ => 8 * 1024 * 1024,                          // > 1 GiB: 8 MiB
    }
}

/// Streaming chunk processor: process bytes in a streaming fashion
/// Calls the callback for each chunk formed, enabling one-pass processing
pub fn process_chunks_streaming<F>(
    bytes: &[u8],
    chunk_size: usize,
    mut callback: F,
) -> anyhow::Result<usize>
where
    F: FnMut(usize, &[u8]) -> anyhow::Result<()>,
{
    assert!(chunk_size > 0, "chunk_size must be greater than zero");
    let mut chunk_index = 0;
    for chunk in bytes.chunks(chunk_size) {
        callback(chunk_index, chunk)?;
        chunk_index += 1;
    }
    Ok(chunk_index)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunking_is_stable() {
        let chunks = chunk_bytes(b"abcdefghij", 4);
        assert_eq!(
            chunks,
            vec![b"abcd".to_vec(), b"efgh".to_vec(), b"ij".to_vec()]
        );
    }

    #[test]
    fn adaptive_chunk_size_small_files() {
        // < 64 MiB → 1 MiB
        assert_eq!(adaptive_chunk_size(1024), 1024 * 1024);
        assert_eq!(adaptive_chunk_size(64 * 1024 * 1024 - 1), 1024 * 1024);
    }

    #[test]
    fn adaptive_chunk_size_medium_files() {
        // 64 MiB to 1 GiB → 4 MiB
        assert_eq!(adaptive_chunk_size(64 * 1024 * 1024), 4 * 1024 * 1024);
        assert_eq!(adaptive_chunk_size(512 * 1024 * 1024), 4 * 1024 * 1024);
        assert_eq!(adaptive_chunk_size(1024 * 1024 * 1024 - 1), 4 * 1024 * 1024);
    }

    #[test]
    fn adaptive_chunk_size_large_files() {
        // > 1 GiB → 8 MiB
        assert_eq!(adaptive_chunk_size(1024 * 1024 * 1024), 8 * 1024 * 1024);
        assert_eq!(
            adaptive_chunk_size(10 * 1024 * 1024 * 1024),
            8 * 1024 * 1024
        );
    }

    #[test]
    fn streaming_chunks_callback_invoked() {
        let data = b"abcdefghijklmnop";
        let mut collected = Vec::new();
        let count = process_chunks_streaming(data, 4, |idx, chunk| {
            collected.push((idx, chunk.to_vec()));
            Ok(())
        })
        .unwrap();
        assert_eq!(count, 4);
        assert_eq!(collected.len(), 4);
        assert_eq!(collected[0], (0, b"abcd".to_vec()));
        assert_eq!(collected[1], (1, b"efgh".to_vec()));
        assert_eq!(collected[2], (2, b"ijkl".to_vec()));
        assert_eq!(collected[3], (3, b"mnop".to_vec()));
    }
}
