#[derive(Debug)]
pub struct USBLocalPayloadRFStatus {
    pub rssi: i8,
    pub snr: i8,
    pub rscp: i8,
}

#[derive(Debug)]
pub struct LoRaProperties {
    pub sf: u8,
    pub bw: u8,
    pub cr: u8,
    pub ldro: u8,
}

#[derive(Debug)]
pub struct USBLocalPayloadRFControl {
    pub freq_hz: u32,
    pub rsvd: u8,
    pub txpower: u8,
    pub lora: LoRaProperties,
}

#[derive(Debug)]
pub struct USBLocalPayloadMACMode {
    pub mode: u8,
}

#[derive(Debug)]
pub enum USBLocalPayload {
    RFStatus(USBLocalPayloadRFStatus),
    RFControl(USBLocalPayloadRFControl),
    MACMode(USBLocalPayloadMACMode),
}

#[derive(Debug)]
pub struct USBLocalPacket {
    pub packet_type: i8,
    pub payload_len: i8,
    pub payload: USBLocalPayload,
}

impl TryFrom<Vec<u8>> for USBLocalPacket {
    type Error = &'static str;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        if value.len() < 3 {
            return Err("Packet too short");
        }

        let packet_type = value[0] as i8;
        let payload_len = value[1] as i8;
        let payload = &value[2..];

        if payload.len() != payload_len as usize {
            return Err("Payload length mismatch");
        }

        let payload = match packet_type {
            1 => USBLocalPayload::RFStatus(USBLocalPayloadRFStatus {
                rssi: payload[0] as i8,
                snr: payload[1] as i8,
                rscp: payload[2] as i8,
            }),
            2 => USBLocalPayload::RFControl(USBLocalPayloadRFControl {
                freq_hz: u32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]),
                rsvd: payload[4],
                txpower: payload[5],
                lora: LoRaProperties {
                    sf: payload[6],
                    bw: payload[7],
                    cr: payload[8],
                    ldro: payload[9],
                },
            }),
            3 => USBLocalPayload::MACMode(USBLocalPayloadMACMode { mode: payload[0] }),
            _ => return Err("Invalid packet type"),
        };

        Ok(USBLocalPacket {
            packet_type,
            payload_len,
            payload,
        })
    }
}

impl TryInto<Vec<u8>> for USBLocalPacket {
    type Error = &'static str;

    fn try_into(self) -> Result<Vec<u8>, Self::Error> {
        let mut result = vec![self.packet_type as u8, self.payload_len as u8];

        match self.payload {
            USBLocalPayload::RFStatus(payload) => {
                result.push(payload.rssi as u8);
                result.push(payload.snr as u8);
                result.push(payload.rscp as u8);
            }
            USBLocalPayload::RFControl(payload) => {
                result.extend_from_slice(&payload.freq_hz.to_le_bytes());
                result.push(payload.rsvd);
                result.push(payload.txpower);
                result.push(payload.lora.sf);
                result.push(payload.lora.bw);
                result.push(payload.lora.cr);
                result.push(payload.lora.ldro);
            }
            USBLocalPayload::MACMode(payload) => {
                result.push(payload.mode);
            }
        }

        Ok(result)
    }
}
