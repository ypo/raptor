mod common;
mod decoder;
mod encoder;
mod encodingsymbols;
mod raptor;
mod sparse_matrix;
mod tables;

pub use decoder::decode_source_block;
pub use encoder::encode_source_block;
pub use encoder::SourceBlockEncoder;

#[cfg(test)]
mod tests {
    pub fn init() {
        std::env::set_var("RUST_LOG", "debug");
        env_logger::builder().is_test(true).try_init().ok();
    }
}
