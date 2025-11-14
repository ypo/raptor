mod tests {

    use rand::{Rng, RngCore};

    pub fn init() {
        std::env::set_var("RUST_LOG", "debug");
        env_logger::builder().is_test(true).try_init().ok();
    }

    fn create_source_block_data(length: usize) -> Vec<u8> {
        let mut output = vec![0u8; length];

        // Random buffer
        let mut rng = rand::rng();
        rng.fill_bytes(output.as_mut());

        output
    }

    fn network_transfer(encoding_symbols: &[Vec<u8>], loss: u32) -> Vec<Option<Vec<u8>>> {
        let mut output = Vec::new();
        let mut rng = rand::rng();

        for symbol in encoding_symbols {
            let n = rng.random_range(0..100); // generates a random number between 0 and 100
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
            raptor_code::encode_source_block(&source_block_data, max_source_symbols, nb_repair)
                .unwrap();

        // Simulate network transfer
        let received_symbols = network_transfer(&encoding_symbols, network_loss);

        let decoded_source_block =
            raptor_code::decode_source_block(&received_symbols, k as usize, source_block_length)
                .unwrap();

        assert!(decoded_source_block.len() == source_block_data.len());
        assert!(decoded_source_block == source_block_data);
    }

    fn on_the_fly_encode(
        source_block: &Vec<u8>,
        max_source_symbols: usize,
        nb_repair_symbols: u32,
    ) -> Vec<Vec<u8>> {
        let mut encoder =
            raptor_code::SourceBlockEncoder::new(&source_block, max_source_symbols).unwrap();
        let n = encoder.nb_source_symbols() + nb_repair_symbols;

        let mut encoded_block = Vec::new();
        for esi in 0..n as u32 {
            let encoding_symbol = encoder.fountain(esi);
            encoded_block.push(encoding_symbol);
        }

        encoded_block
    }

    fn on_the_fly_decode(
        source_block_length: usize,
        nb_source_symbols: usize,
        encoded_block: &Vec<Option<Vec<u8>>>,
    ) -> Option<Vec<u8>> {
        let mut decoder = raptor_code::SourceBlockDecoder::new(nb_source_symbols);
        for (esi, encoding_symbol) in encoded_block.iter().enumerate() {
            if decoder.fully_specified() {
                break;
            }
            if let Some(encoding_symbol) = encoding_symbol {
                decoder.push_encoding_symbol(encoding_symbol, esi as u32);
            }
        }

        assert!(decoder.fully_specified());
        decoder.decode(source_block_length as usize)
    }

    fn on_the_fly_encode_decode(
        source_block_length: usize,
        max_source_symbols: usize,
        nb_repair_symbols: u32,
        network_loss: u32,
    ) {
        let source_block_data = create_source_block_data(source_block_length);

        // Test block encoer
        let encoding_symbols =
            on_the_fly_encode(&source_block_data, max_source_symbols, nb_repair_symbols);

        // Simulate packet loss
        let received_encoding_symbols = network_transfer(&encoding_symbols, network_loss);

        // Test block decoder
        let decoded_source_block = on_the_fly_decode(
            source_block_length,
            max_source_symbols,
            &received_encoding_symbols,
        )
        .unwrap();

        // Check decoded block
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

    #[test]
    pub fn test_onthefly_3k_repair3_loss10_issue5() {
        init();
        on_the_fly_encode_decode(3684, 4, 3, 10);
    }
}
