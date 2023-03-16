use std::ops::{Div, Mul};

use anyhow::bail;
use bitvec::prelude::*;

pub const SM_7_BIT_END_BYTE: u8 = 0x80;
#[allow(clippy::upper_case_acronyms)]
#[derive(Debug)]
pub enum SM7BitControlBits {
    USBLocal = 0x02,
    WRC = 0x04,
}

pub fn decode(data: &[u8]) -> anyhow::Result<(SM7BitControlBits, Vec<u8>)> {
    let pkg_type = {
        match data.last() {
            Some(&SM_7_BIT_END_BYTE) => (),
            _ => bail!("Invalid SM7BitControlBits"),
        };
        match data.first() {
            Some(0x02) => SM7BitControlBits::USBLocal,
            Some(0x04) => SM7BitControlBits::WRC,
            _ => bail!("Invalid SM7BitControlBits"),
        }
    };

    let (start, end) = (1usize, data.len().saturating_sub(2));
    let mut bv: BitVec<u8, Msb0> = BitVec::new();

    for i in data.iter().skip(start).take(end) {
        for j in (1..=7).rev() {
            bv.push(i & (1 << j) != 0);
        }
    }

    bv.truncate(bv.len().div(8).mul(8));

    Ok((pkg_type, bv.into_vec()))
}

pub fn encode(data: &[u8], pkg_type: SM7BitControlBits) -> Vec<u8> {
    let mut bv: BitVec<u8, Msb0> = BitVec::new();

    bv.append(&mut BitVec::<_, Msb0>::from_element(pkg_type as u8));
    for i in data {
        for j in (0..=7).rev() {
            if bv.len() % 8 == 7 {
                bv.push(true);
            }
            bv.push(i & (1 << j) != 0);
        }
    }

    while bv.len() % 8 != 0 {
        if bv.len() % 8 == 7 {
            bv.push(true);
        } else {
            bv.push(false);
        }
    }

    bv.append(&mut BitVec::<_, Msb0>::from_element(SM_7_BIT_END_BYTE));

    bv.into_vec()
}

#[cfg(test)]
mod tests {
    use super::decode;

    #[test]
    fn decode_test() {
        let data = vec![0x02, 0xcb, 0x7f, 0xaf, 0x57, 0xe1, 0x80];
        let decoded = decode(&data).unwrap();
        assert_eq!(decoded.0 as u8, SM7BitControlBits::USBLocal as u8);
        assert_eq!(decoded.1, vec![0xca, 0xfe, 0xba, 0xbe]);

        assert!(decode(&[]).is_err());
    }

    use super::encode;
    use super::SM7BitControlBits;
    #[test]
    fn encode_test() {
        let data = vec![0xca, 0xfe, 0xba, 0xbe];
        assert_eq!(
            encode(&data, SM7BitControlBits::USBLocal),
            vec![0x02, 0xcb, 0x7f, 0xaf, 0x57, 0xe1, 0x80]
        );

        assert_eq!(encode(&[], SM7BitControlBits::USBLocal), vec![0x02, 0x80]);
    }
}
