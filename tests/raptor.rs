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

        let max_source_symbols =
            (source_block_length as f64 / encoding_symbol_size as f64).ceil() as usize;

        let (encoding_symbols, k) =
            raptor_code::encode_source_block(&source_block_data, max_source_symbols, nb_repair);

        // Simulate network transfer
        let received_symbols = network_transfer(&encoding_symbols, network_loss);

        let decoded_source_block =
            raptor_code::decode_source_block(&received_symbols, k as usize, source_block_length)
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
