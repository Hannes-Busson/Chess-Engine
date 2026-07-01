use std::sync::OnceLock;
static KEYS: OnceLock<[u64; 781]> = OnceLock::new();

pub fn keys() -> &'static [u64; 781] {
    KEYS.get_or_init(|| {
        let mut result = [0u64; 781];
        let mut s = 0x123456789ABCDEFu64;
        for r in result.iter_mut() {
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            *r = s;
        }
        result
    })
}
