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

    // -------------------------------------------------------------------
    // Error-path / corner-case tests
    // -------------------------------------------------------------------

    #[test]
    pub fn test_encode_source_block_too_small_returns_err() {
        // max_source_symbols=3 partitions into k=3 (<4), which the encoder
        // must reject as not fully specified.
        init();
        let data = vec![1u8, 2, 3, 4, 5, 6];
        let result = raptor_code::encode_source_block(&data, 3, 5);
        assert!(result.is_err(), "expected Err for k<4, got Ok");
    }

    #[test]
    pub fn test_decode_source_block_insufficient_returns_none() {
        // Encode then drop more symbols than nb_repair so decoding must fail.
        init();
        let source_block_length = 4096;
        let data = create_source_block_data(source_block_length);
        let max_source_symbols = 8;
        let nb_repair = 2;

        let (encoding_symbols, k) =
            raptor_code::encode_source_block(&data, max_source_symbols, nb_repair).unwrap();

        // Drop ALL symbols -> decoder must return None
        let received: Vec<Option<Vec<u8>>> = encoding_symbols.iter().map(|_| None).collect();
        let result =
            raptor_code::decode_source_block(&received, k as usize, source_block_length);
        assert!(result.is_none(), "expected None when no symbols received");
    }

    #[test]
    pub fn test_decoder_not_fully_specified_returns_none() {
        // Build a decoder, push fewer symbols than k, decode() must return None.
        init();
        let nb_source_symbols = 16;
        let mut decoder = raptor_code::SourceBlockDecoder::new(nb_source_symbols);
        assert!(!decoder.fully_specified());
        // Push only 2 symbols (way less than k)
        decoder.push_encoding_symbol(&vec![0u8; 64], 0);
        decoder.push_encoding_symbol(&vec![0u8; 64], 1);
        assert!(!decoder.fully_specified());
        assert!(decoder.decode(64 * nb_source_symbols).is_none());
    }

    #[test]
    pub fn test_systematic_fountain_returns_source_symbols() {
        // After encoding, fountain(esi) for esi < k must equal the partitioned
        // source symbol (systematic property used by N1 short-circuit).
        init();
        let source_block_length = 4096;
        let encoding_symbol_size = 256;
        let max_source_symbols =
            (source_block_length + encoding_symbol_size - 1) / encoding_symbol_size;
        let data = create_source_block_data(source_block_length);

        let mut encoder =
            raptor_code::SourceBlockEncoder::new(&data, max_source_symbols).unwrap();
        let k = encoder.nb_source_symbols() as usize;

        // Reconstruct expected source symbols by simple chunking (matches RFC partition
        // when source_length is a multiple of nb_symbols).
        assert_eq!(source_block_length % max_source_symbols, 0);
        let chunk = source_block_length / max_source_symbols;
        for esi in 0..k {
            let expected = &data[esi * chunk..(esi + 1) * chunk];
            let actual = encoder.fountain(esi as u32);
            assert_eq!(
                actual, expected,
                "systematic property violated for esi={}",
                esi
            );
        }
    }

    #[test]
    pub fn test_decode_using_only_repair_symbols() {
        // Drop ALL systematic symbols (esi < k) and decode using only repair symbols.
        // Validates the LT decoding path end-to-end.
        init();
        let source_block_length = 8192;
        let encoding_symbol_size = 256;
        let max_source_symbols =
            (source_block_length + encoding_symbol_size - 1) / encoding_symbol_size;
        // Need enough repair symbols so we can recover from losing all source symbols.
        let nb_repair = max_source_symbols + 8;
        let data = create_source_block_data(source_block_length);

        let (encoding_symbols, k) =
            raptor_code::encode_source_block(&data, max_source_symbols, nb_repair).unwrap();

        let received: Vec<Option<Vec<u8>>> = encoding_symbols
            .iter()
            .enumerate()
            .map(|(esi, sym)| {
                if (esi as u32) < k {
                    None // drop all systematic
                } else {
                    Some(sym.clone())
                }
            })
            .collect();

        let decoded =
            raptor_code::decode_source_block(&received, k as usize, source_block_length)
                .expect("decoding using only repair symbols should succeed");
        assert_eq!(decoded, data);
    }

    // -------------------------------------------------------------------
    // Several K values, losses and reception orders
    // -------------------------------------------------------------------

    fn decode_in_order(
        encoding_symbols: &[Vec<u8>],
        order: &[usize],
        nb_source_symbols: usize,
        source_block_length: usize,
        push_all: bool,
    ) -> Vec<u8> {
        let mut decoder = raptor_code::SourceBlockDecoder::new(nb_source_symbols);
        for &esi in order {
            if !push_all && decoder.fully_specified() {
                break;
            }
            decoder.push_encoding_symbol(&encoding_symbols[esi], esi as u32);
        }
        assert!(decoder.fully_specified());
        let decoded = decoder.decode(source_block_length).unwrap();
        // A second decode gives the same result
        assert!(decoder.decode(source_block_length).unwrap() == decoded);
        decoded
    }

    #[test]
    pub fn test_encode_decode_k_values_and_orders() {
        use rand::seq::SliceRandom;
        use rand::SeedableRng;

        init();
        for k in [1usize, 4, 5, 10, 63, 64, 65, 100, 1000] {
            let mut rng = rand::rngs::StdRng::seed_from_u64(k as u64);
            let encoding_symbol_size = 8 + k % 5;
            // Not a multiple of k: the block has long and short source symbols
            let source_block_length = k * encoding_symbol_size - k / 3;
            let mut data = vec![0u8; source_block_length];
            rng.fill_bytes(&mut data);
            let nb_repair = k / 2 + 20;

            let (encoding_symbols, nb_source_symbols) =
                raptor_code::encode_source_block(&data, k, nb_repair).unwrap();
            assert_eq!(nb_source_symbols as usize, k);
            assert_eq!(encoding_symbols.len(), k + nb_repair);

            // Drop 1 source symbol out of 3 (the only one when k = 1) and 1
            // repair symbol out of 5
            let is_received = |esi: usize| {
                if esi < k {
                    esi % 3 != 0
                } else {
                    esi % 5 != 0
                }
            };
            let in_order: Vec<usize> = (0..encoding_symbols.len())
                .filter(|&esi| is_received(esi))
                .collect();
            let repair_first: Vec<usize> = in_order
                .iter()
                .copied()
                .filter(|&esi| esi >= k)
                .chain(in_order.iter().copied().filter(|&esi| esi < k))
                .collect();
            let reverse: Vec<usize> = in_order.iter().rev().copied().collect();
            let mut shuffled = in_order.clone();
            shuffled.shuffle(&mut rng);

            for order in [&in_order, &repair_first, &reverse, &shuffled] {
                for push_all in [false, true] {
                    let decoded =
                        decode_in_order(&encoding_symbols, order, k, source_block_length, push_all);
                    assert!(decoded == data, "k={} push_all={}", k, push_all);
                }
            }

            let received: Vec<Option<Vec<u8>>> = encoding_symbols
                .iter()
                .enumerate()
                .map(|(esi, symbol)| is_received(esi).then(|| symbol.clone()))
                .collect();
            let decoded =
                raptor_code::decode_source_block(&received, k, source_block_length).unwrap();
            assert!(decoded == data, "k={}", k);
        }
    }
}
