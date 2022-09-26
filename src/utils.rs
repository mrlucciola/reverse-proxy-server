pub struct Payload {
    id: String,     // "000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f 1b60a8ce26f",
    height: u32,    // 0,
    version: u32,   // 1,
    timestamp: u32, // 1231006505,
    tx_count: u32,  // 1,
    size: u32,      // 285,
    weight: u32,    // 816,
    merkle_root: String, // "4a5e1e4baab 89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b",
    previousblockhash: Option<String>, // null,
    mediantime: u32, // 1231006505,
    nonce: u32,     // 2083236893,
    bits: u32,      // 486604799,
    difficulty: u32, // 1
}
