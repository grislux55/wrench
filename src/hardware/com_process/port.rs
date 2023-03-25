use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    },
    time::Duration,
};

use serialport::SerialPort;
use tracing::{debug, error};

use crate::hardware::{
    message::wrc::WRCPacket,
    sm7bits::{self, SM7BitControlBits, SM_7_BIT_END_BYTE},
};

fn open_port<'a>(
    port: impl Into<std::borrow::Cow<'a, str>>,
    exit_required: Arc<AtomicBool>,
) -> Option<Box<dyn SerialPort>> {
    let port = port.into();

    while !exit_required.load(Ordering::Acquire) {
        if let Ok(p) = serialport::new(port.clone(), 115_200)
            .timeout(Duration::from_millis(1000))
            .open()
        {
            return Some(p);
        } else {
            error!("无法打开端口, 原因: {}, 将在 1 秒后进行重试", port);
            std::thread::sleep(Duration::from_millis(1000));
        }
    }

    None
}

fn read_packet(
    exit_required: Arc<AtomicBool>,
    port: &mut Box<dyn serialport::SerialPort>,
) -> Vec<u8> {
    let mut readed = vec![];
    let mut serial_buf = [0];

    while !exit_required.load(Ordering::Acquire) {
        match port.read(&mut serial_buf) {
            Ok(readed_size) => {
                if readed_size == 0 {
                    break;
                }
                if readed.is_empty()
                    && serial_buf[0] != SM7BitControlBits::USBLocal as u8
                    && serial_buf[0] != SM7BitControlBits::WRC as u8
                {
                    continue;
                }

                readed.push(serial_buf[0]);
                if serial_buf[0] == SM_7_BIT_END_BYTE {
                    break;
                }
            }
            Err(_) => {
                break;
            }
        }
    }

    readed
}

fn reader(exit_required: Arc<AtomicBool>, port: &mut Box<dyn SerialPort>) -> Option<WRCPacket> {
    let readed = read_packet(exit_required, port);
    if readed.is_empty() {
        return None;
    }

    match sm7bits::decode(&readed) {
        Ok((SM7BitControlBits::WRC, decoded)) => match WRCPacket::try_from(decoded) {
            Ok(p) => Some(p),
            Err(e) => {
                error!("无法解析字节流内容: {readed:02X?}, 原因: {e}");
                None
            }
        },
        Err(e) => {
            error!("无法按照sm7bits协议转换字节流: {readed:02X?}, 原因: {e}");
            None
        }
        _ => None,
    }
}

pub fn read_write_loop<'a>(
    rx: mpsc::Receiver<WRCPacket>,
    tx: mpsc::Sender<WRCPacket>,
    port: impl Into<std::borrow::Cow<'a, str>>,
    exit_required: Arc<AtomicBool>,
) {
    let port = port.into();
    let mut opened_port = None;

    debug!("启动读写循环");
    while !exit_required.load(Ordering::Acquire) {
        if opened_port.is_none() {
            opened_port = open_port(port.clone(), exit_required.clone());
            if opened_port.is_some() {
                debug!("端口成功打开");
            }
            continue;
        }

        if let Some(readed) = reader(exit_required.clone(), opened_port.as_mut().unwrap()) {
            if let Err(e) = tx.send(readed) {
                error!("无法发送数据包至串口处理线程: {}", e);
            }
        }

        if let Ok(packet) = rx.try_recv() {
            match TryInto::<Vec<u8>>::try_into(packet) {
                Ok(data) => {
                    let encoded = sm7bits::encode(&data, SM7BitControlBits::WRC);
                    if let Err(e) = opened_port.as_mut().unwrap().write_all(&encoded) {
                        error!("无法写入数据至端口: {}", e);
                        opened_port = None;
                    }
                }
                Err(e) => {
                    error!("无法转换数据包到字节流: {}", e);
                }
            }
        }
    }
}
