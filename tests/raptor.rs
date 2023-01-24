mod tests {

    use rand::{Rng, RngCore};

    pub fn init() {
        std::env::set_var("RUST_LOG", "debug");
        env_logger::builder().is_test(true).try_init().ok();
    }

    fn create_source_block_data(length: usize) -> Vec<u8> {
        let mut output = vec![0u8; length];

        // Random buffer
        let mut rng = rand::thread_rng();
        rng.fill_bytes(output.as_mut());

        output
    }

    fn create_source_block(data: &[u8], encoding_symbol_size: usize) -> Vec<Vec<u8>> {
        data.chunks(encoding_symbol_size)
            .map(|data| {
                let mut source_symbols = data.to_vec();
                source_symbols.resize(encoding_symbol_size, 0);
                source_symbols
            })
            .collect()
    }

    fn encode(data: &[u8], encoding_symbol_size: usize, nb_repair: usize) -> (Vec<Vec<u8>>, usize) {
        let source_block = create_source_block(data, encoding_symbol_size);
        (
            raptor::encode_source_block(&source_block, nb_repair),
            source_block.len(),
        )
    }

    fn network_transfer(encoding_symbols: &[Vec<u8>], loss: u32) -> Vec<Option<Vec<u8>>> {
        let mut output = Vec::new();
        let mut rng = rand::thread_rng();

        for symbol in encoding_symbols {
            let n = rng.gen_range(0..100); // generates a random number between 0 and 100
            if n < loss {
                output.push(None);
            } else {
                output.push(Some(symbol.clone()));
            }
        }

        output
    }

    fn encode_decode(
        source_block_length: usize,
        encoding_symbol_size: usize,
        nb_repair: usize,
        network_loss: u32,
    ) {
        let source_block_data = create_source_block_data(source_block_length);

        assert!(source_block_data.len() == source_block_length);

        let (encoding_symbols, nb_source_symbols) =
            encode(&source_block_data, encoding_symbol_size, nb_repair);

        // Simulate network transfer
        let received_symbols = network_transfer(&encoding_symbols, network_loss);

        let decoded_source_block = raptor::decode_source_block(
            &received_symbols,
            nb_source_symbols,
            source_block_length,
            encoding_symbol_size,
        )
        .unwrap();

        assert!(decoded_source_block.len() == source_block_data.len());
        assert!(decoded_source_block == source_block_data);
    }

    #[test]
    pub fn test_encode_decode_100k_repair100_loss5() {
        init();
        encode_decode(1000 * 1000, 1024, 100, 5);
    }

    #[test]
    pub fn test_encode_decode_no_loss() {
        init();
        encode_decode(1000, 10, 0, 0);
    }

    #[test]
    pub fn test_encode_decode_64k_repair25_loss5() {
        init();
        encode_decode(64 * 1000, 1024, 25, 5);
    }

    #[test]
    pub fn test_encode_decode_1k_repair0() {
        init();
        encode_decode(1024, 1024, 0, 0);
    }
}
