use std::net::{SocketAddr, UdpSocket};
use std::sync::Arc;
use std::{io, thread};
use std::time::Duration;
use tokio::spawn;

pub async fn udp(){
    let socket = Arc::new(UdpSocket::bind("0.0.0.0:8080").unwrap());
    socket.set_read_timeout(Some(Duration::from_secs(5))).unwrap();


    let sender_socket = Arc::clone(&socket);
    spawn(async move {
        sender(&sender_socket);
    });


    receiver(&socket);
}

fn sender(socket: &Arc<UdpSocket>) {
    let address : SocketAddr = "127.0.0.1:8080".parse().unwrap();
    let mut count: i32 = 1;
    loop{

        if let Err(e) = socket.send_to(&count.to_be_bytes(), address){
            println!("Failed to send message: {}", e);
            continue;
        }
        count+=1;
        thread::sleep(Duration::from_millis(100));
    }
}

fn receiver(socket: &Arc<UdpSocket>) {
    socket.set_read_timeout(Some(Duration::from_secs(5))).unwrap();

    let mut buffer = [0u8; 1024]; //1MB buffer

    loop {
        match socket.recv_from(&mut buffer) {
            Ok((size, _src)) => {
                println!("Received {} bytes from {:?} {:?}", size, _src, buffer);
            },
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                println!("Would Block");
            }
            Err(e) => {
                println!("Failed to receive message: {}", e);
            },

        }
    }
}