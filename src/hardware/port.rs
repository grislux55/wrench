use std::{
    collections::HashMap,
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

use crate::hardware::message::wrc::WRCPacket;

use super::message::usb::USBLocalPacket;
use super::sm7bits;
use super::sm7bits::SM_7_BIT_END_BYTE;
use std::sync::Arc;
use tracing::{debug, error};

fn read_until(port: &mut Box<dyn serialport::SerialPort>, byte: u8) -> anyhow::Result<Vec<u8>> {
    let mut readed = vec![];
    let mut serial_buf = [0];
    loop {
        if port.read(&mut serial_buf)? == 0 {
            return Err(anyhow::anyhow!("No data"));
        }
        readed.push(serial_buf[0]);
        if serial_buf[0] == byte {
            break;
        }
    }
    Ok(readed)
}

fn com_process<'a>(
    exit_required: Arc<AtomicBool>,
    port: impl Into<std::borrow::Cow<'a, str>>,
) -> anyhow::Result<()> {
    let mut port = serialport::new(port, 115_200)
        .timeout(Duration::from_millis(1000))
        .open()?;

    while !exit_required.load(Ordering::Acquire) {
        if let Ok(readed) = read_until(&mut port, SM_7_BIT_END_BYTE) {
            debug!("readed: {readed:02X?}");
            match sm7bits::decode(&readed) {
                Ok((controlbits, decoded)) => {
                    debug!("decoded: {decoded:02X?}");
                    match controlbits {
                        sm7bits::SM7BitControlBits::WRC => {
                            debug!("pkt: {:?}", WRCPacket::try_from(decoded));
                        }
                        sm7bits::SM7BitControlBits::USBLocal => {
                            debug!("pkt: {:?}", USBLocalPacket::try_from(decoded));
                        }
                    }
                }
                Err(e) => {
                    error!("Cannot decode: {readed:02X?}, reason: {e}");
                }
            }
        }
    }

    Ok(())
}

pub fn loop_query(exit_required: Arc<AtomicBool>) -> anyhow::Result<()> {
    let mut last_com_threads = HashMap::new();
    let mut com_thread_handles = vec![];

    while !exit_required.load(Ordering::Acquire) {
        let ports = serialport::available_ports()?;
        for p in ports.iter() {
            if !last_com_threads.contains_key(&p.port_name) {
                debug!("New port: {}", p.port_name);
                let port = p.port_name.clone();
                {
                    let exit_required = exit_required.clone();
                    com_thread_handles.push((
                        port.clone(),
                        std::thread::spawn(move || {
                            if let Err(e) = com_process(exit_required, port) {
                                debug!("Error: {e}");
                            }
                        }),
                    ));
                }
            }
        }
        last_com_threads.clear();
        last_com_threads.extend(ports.into_iter().map(|p| (p.port_name, ())));
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }

    for h in com_thread_handles {
        h.1.join()
            .map_err(|_| anyhow::anyhow!(format!("cannot wait thread handle {:?}", h.0)))?;
    }

    Ok(())
}
