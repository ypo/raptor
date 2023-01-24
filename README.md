# raptor

## Raptor

A Rust library for implementing Forward Error Correction (FEC) using Raptor codes.

Raptor codes are a class of FEC codes that are designed to be highly efficient in the presence of packet erasures.
This library provides functionality for encoding source blocks into encoding symbols and decoding source blocks from a set of encoding symbols.

This library implements on the fly Gaussian Elimination to spread  decoding complexity during packets reception.

## Examples

Encode and decode a source block using `raptor::encode_source_block` and `raptor::decode_source_block`


```rust

let source_data: Vec<u8> = vec![1,2,3,4,5,6,7,8,9,10,11,12];
let encoding_symbol_length = 3;
let source_block:Vec<Vec<u8>> = source_data.chunks(encoding_symbol_length)
                                    .map(|source_symbol| source_symbol.to_vec()).collect();
let nb_repair = 3;


// Step 1 - Generate the encoding symbols (source symbols + repair symbols)
let encoding_symbols = raptor::encode_source_block(&source_block, nb_repair);

// Step 2 - Re-construct the source data from the encoding symbols

let nb_source_symbols = source_block.len();
let source_block_length = source_data.len();

let mut received_symbols: Vec<Option<Vec<u8>>> = encoding_symbols.into_iter().map(|symbols| Some(symbols)).collect();
received_symbols[0] = None; // simulate encoding symbol ESI=0 lost

let reconstructed_data = raptor::decode_source_block(&received_symbols, nb_source_symbols, source_block_length).unwrap();

// Source data and decoded data should be identical
assert!(reconstructed_data == source_data)
```

## Credit

RFC 5053 <https://www.rfc-editor.org/rfc/rfc5053.html>

On the fly Gaussian Elimination for LT codes, Valerio Bioglio, Marco Grangetto, 2009

Reuse ideas and concepts of [gofountain](https://github.com/google/gofountain)

