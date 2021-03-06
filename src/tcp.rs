use std::thread;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::net::{TcpListener, TcpStream};
use std::io::{Error, Read};
use std::str::from_utf8;
use actions::{Action, Message};

pub fn listen() -> Result<(Sender<Action>, Receiver<Action>), Error> {
    let (tx, rx) = channel();
    let tx_clone = tx.clone();
    let listener = TcpListener::bind("127.0.0.1:7033")?;

    thread::spawn(move || {
        loop {
            let (socket, address) = listener.accept().unwrap();

            let action = Action {
                domain: String::from("client"),
                invocation: String::from("add"),
                message: Message::Client(Arc::new(Mutex::new(socket))),
                sender: format!("{}", address),
            };

            match tx.send(action) {
                Ok(()) => (),
                Err(err) => {
                    println!("{:?}", err);
                    break;
                },
            };
        }
    });

    Ok((tx_clone, rx))
}

pub fn receive(socket : Arc<Mutex<TcpStream>>, tx : Sender<Action>, sender : String) -> Result<(), Error> {
    thread::spawn(move || {
        let mut data = String::new();

        loop {
            let mut buffer = [0; 128];

            match socket.lock().unwrap().read(&mut buffer[..]) {
                Ok(0) => (),
                Ok(size) => {
                    data.push_str(from_utf8(&buffer[..size]).expect("failed to convert tcp data to string"));

                    let data_clone = data.clone();
                    let chunks : Vec<&str> = data_clone.split("\n").collect();
                    let (&rest, messages) = chunks.split_last().unwrap();

                    for &message in messages {
                        let action = Action {
                            domain: String::from("client"),
                            invocation: String::from("receive"),
                            message: Message::Raw(String::from(message)),
                            sender: sender.clone(),
                        };

                        tx.send(action).unwrap();
                    }

                    data.truncate(0);
                    data.push_str(rest);
                },
                Err(err) => {
                    println!("{:?}", err);
                    break;
                }
            }
        }
    });

    Ok(())
}
