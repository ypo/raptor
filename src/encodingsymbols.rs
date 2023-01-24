pub struct EncodingSymbol<'a> {
    pub data: &'a [u8],
    pub esi: u32,
}

impl<'a> EncodingSymbol<'a> {
    pub fn new(data: &'a [u8], esi: u32) -> Self {
        EncodingSymbol { data, esi }
    }

    pub fn from_block(block: &[Vec<u8>]) -> Vec<EncodingSymbol> {
        block
            .iter()
            .enumerate()
            .map(|(esi, symbols)| EncodingSymbol {
                data: symbols.as_ref(),
                esi: esi as u32,
            })
            .collect()
    }

    pub fn from_option_block(block: &[Option<Vec<u8>>]) -> Vec<EncodingSymbol> {
        block
            .iter()
            .enumerate()
            .filter(|(_, symbols)| symbols.is_some())
            .map(|(esi, symbols)| EncodingSymbol {
                data: symbols.as_ref().unwrap(),
                esi: esi as u32,
            })
            .collect()
    }
}
