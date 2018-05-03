extern crate libusb;

use std::net::UdpSocket;
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

fn main() {
    let (sender, receiver) = channel();

    let artnet_thread = thread::spawn(move || {
        let socket = UdpSocket::bind("127.0.0.1:6454").unwrap();

        loop {
            let mut buf = [0; 530];
            let _ = socket.recv_from(&mut buf).unwrap();

            // let universe: u16 = (buf[14] as u16 * 256) + buf[15] as u16;
            let length = (buf[16] as u16 * 256) + buf[17] as u16;

            let slice_end = 18 + length as usize;
            let data = buf[18..slice_end].to_vec();

            sender.send(data).unwrap();
        }
    });

    let udmx_thread = thread::spawn(move|| {
        let context = libusb::Context::new().unwrap();

        let device = context.devices()
            .ok()
            .and_then(|devices| {
                devices
                    .iter()
                    .find(|device| {
                        let device_desc = device.device_descriptor();
                        if device_desc.is_err() {
                            return false;
                        }
                        let device_desc = device_desc.unwrap();
                        device_desc.vendor_id() == 0x16c0 &&
                            device_desc.product_id() == 0x05dc
                    })
            })
            .expect("device not connected");

        let handle = device.open().unwrap();
        loop {
            let data = receiver.recv().unwrap();
            match handle.write_control(64, 2, data.len() as u16, 0, &data, Duration::from_secs(1)) {
                Err(e) => println!("err {:?}", e),
                _ => {}
            }
        }
    });

    udmx_thread.join().unwrap();
    artnet_thread.join().unwrap();
}