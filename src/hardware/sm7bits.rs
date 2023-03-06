pub fn decode(data: &[u8]) -> Vec<u8> {
    let (start, end) = (1usize, data.len().saturating_sub(2));
    if end <= start {
        return vec![];
    }
    let len = end - start + 1;
    let mut ret: Vec<u8> = vec![0; len * 7 / 8];
    let mut wrote_bits = 0;

    for i in start..=end {
        for j in (1..=7).rev() {
            if data[i] & (1 << j) != 0 {
                ret[wrote_bits / 8] |= 1 << (7 - (wrote_bits % 8));
            }
            wrote_bits += 1;
            if wrote_bits / 8 >= ret.len() {
                return ret;
            }
        }
    }

    ret
}

pub fn encode(data: &[u8]) -> Vec<u8> {
    let target_len = (8 * data.len() + 7 - 1) / 7 + 2;
    let mut ret: Vec<u8> = vec![0; target_len];
    ret[0] = 0x00;
    let mut wrote_bits = 8;
    for i in 0..data.len() {
        for j in (0..=7).rev() {
            if wrote_bits % 8 == 7 {
                ret[wrote_bits / 8] |= 1 << (7 - (wrote_bits % 8));
                wrote_bits += 1;
            }
            if data[i] & (1 << j) != 0 {
                ret[wrote_bits / 8] |= 1 << (7 - (wrote_bits % 8));
            }
            wrote_bits += 1;
        }
    }

    while wrote_bits / 8 != target_len - 1 {
        if wrote_bits % 8 == 7 {
            ret[wrote_bits / 8] |= 1 << (7 - (wrote_bits % 8));
            wrote_bits += 1;
        }
        ret[wrote_bits / 8] |= 0 << (7 - (wrote_bits % 8));
        wrote_bits += 1;
    }

    ret[target_len - 1] = 0x02;

    ret
}

#[cfg(test)]
mod tests {
    use super::decode;

    #[test]
    fn decode_test() {
        let data = vec![0x00, 0xcb, 0x7f, 0xaf, 0x57, 0xe1, 0x02];
        assert_eq!(decode(&data), vec![0xca, 0xfe, 0xba, 0xbe]);

        assert!(decode(&[]).is_empty());
    }

    use super::encode;
    #[test]
    fn encode_test() {
        let data = vec![0xca, 0xfe, 0xba, 0xbe];
        assert_eq!(
            encode(&data),
            vec![0x00, 0xcb, 0x7f, 0xaf, 0x57, 0xe1, 0x02]
        );

        assert_eq!(encode(&[]), vec![0x00, 0x02]);
    }
}
