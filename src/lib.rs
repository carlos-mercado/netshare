use crossterm::{
    ExecutableCommand,
    cursor::{DisableBlinking, EnableBlinking, Hide, Show},
    event::{self, Event, KeyCode, poll},
    terminal::Clear,
};
use getifaddrs::getifaddrs;
use std::io::stdout;
use std::io::{self, Result, Write};
use std::net::{IpAddr, Ipv4Addr};
use std::net::{SocketAddr, UdpSocket};
use std::sync::{Arc, Mutex};
use std::thread;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio::time::{Duration, sleep};

const START_BYTE: char = '\x1b';
const PORT: i16 = 14953;
const TCP_PORT: i16 = 14952;

// SENDER STUFF -------------------------------------------------------------------
pub async fn connect(user_ip: &Ipv4Addr) -> Result<TcpStream> {
    let remote_address = get_remote_ip(&user_ip).await?;

    Ok(establish_tcp(remote_address.clone()).await?)
}

pub async fn establish_tcp(remote_ip: String) -> Result<TcpStream> {
    let stream = TcpStream::connect(remote_ip + ":" + &TCP_PORT.to_string()).await?;
    Ok(stream)
}

pub async fn listen_tcp(local_ip: &Ipv4Addr) -> Result<TcpStream> {
    //println!("I am now listening for tcp requests at {}:{}", local_ip, TCP_PORT);
    let listener = TcpListener::bind(format!("{local_ip}:{TCP_PORT}")).await?;
    let (stream, _) = listener.accept().await?;

    Ok(stream)
}

pub async fn get_remote_ip(ip: &Ipv4Addr) -> Result<String> {
    stdout().execute(Clear(crossterm::terminal::ClearType::All))?;
    stdout().execute(DisableBlinking)?;
    stdout().execute(Hide)?;

    // Drain any buffered input (e.g. Enter keypress from main menu)
    while event::poll(Duration::from_millis(0))? {
        let _ = event::read();
    }

    let mut selection = 0;
    let listeners: Vec<IpAddr> = Vec::new();
    let m = Arc::new(Mutex::new(listeners));
    let main_mutex_clone = Arc::clone(&m);
    let vec_mutex_clone = Arc::clone(&m);

    let discovery_socket = UdpSocket::bind("0.0.0.0:".to_string() + &PORT.to_string())
        .expect("couldn't bind to address");

    discovery_socket
        .set_nonblocking(true)
        .expect("couldn't set nonblocking");

    discovery_socket
        .set_broadcast(true)
        .expect("set_broadcast call failed");

    let ip_clone = ip.clone();

    // Broadcasts DISC to the network, responds to incoming DISC with our IP,
    // and collects RESP messages into the peer list.
    let _discovery_handle = tokio::spawn(async move {
        let my_netmask: Ipv4Addr = match get_netmask(ip_clone) {
            Some(res) => to_ipv4(res).unwrap(),
            None => Ipv4Addr::new(255, 255, 255, 0),
        };

        let broadcast_addr: Ipv4Addr = find_ipv4_broadcast_address(ip_clone, my_netmask);

        loop {
            let mut buff = [0u8; 64];
            match discovery_socket.recv_from(&mut buff) {
                Ok((_, src_addr)) => {
                    if src_addr.ip() == ip_clone {
                        continue;
                    }

                    if &buff[0..4] == b"DISC" {
                        let response_string = format!("RESP:{}", ip_clone);
                        let reply_addr = SocketAddr::new(src_addr.ip(), PORT as u16);
                        discovery_socket
                            .send_to(response_string.as_bytes(), reply_addr)
                            .unwrap();
                    } else if &buff[0..5] == b"RESP:" {
                        let remote_ip_str = std::str::from_utf8(&buff[5..])
                            .unwrap()
                            .trim_matches(char::from(0));
                        if let Ok(remote_ip) = remote_ip_str.parse::<IpAddr>() {
                            let mut vec = vec_mutex_clone.lock().unwrap();
                            if !vec.contains(&remote_ip) {
                                vec.push(remote_ip);
                            }
                        }
                    }
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
                Err(e) => panic!("discovery recv error: {e}"),
            }

            discovery_socket
                .send_to(b"DISC", broadcast_addr.to_string() + ":" + &PORT.to_string(),)
                .expect("Couldn't send broadcast message");


            sleep(Duration::from_secs(1)).await;
        }
    });

    // this is for windows powershell, does not work without it.
    while event::poll(Duration::from_millis(0))? {
        let _ = event::read();
    }

    loop {
        stdout().execute(crossterm::cursor::MoveTo(0, 0))?;

        let items = {
            let list = main_mutex_clone.lock().unwrap();
            list.clone()
        };

        if items.is_empty() {
            let loading_string = format!("Finding users...\r\n");

            stdout().write_all(loading_string.as_bytes())?;
            stdout().flush()?;
            while event::poll(Duration::from_millis(0))? {
                let _ = event::read();
            }
            sleep(Duration::from_millis(200)).await;
        }

        for (i, item) in items.iter().enumerate() {
            if i == selection {
                stdout().write_all(b"> ")?;
            } else {
                stdout().write_all(b"  ")?;
            }
            stdout().write_all(&item.to_string().as_bytes())?;
            stdout().write_all(b"\r\n")?;
        }
        stdout().flush()?;

        if poll(Duration::from_millis(100))? {
            if let Event::Key(key_event) = event::read()? {
                if key_event.kind != event::KeyEventKind::Press {
                    continue;
                }
                match key_event.code {
                    KeyCode::Char('k') | KeyCode::Up if selection > 0 => selection -= 1,
                    KeyCode::Char('j') | KeyCode::Down if selection < items.len() - 1 => {
                        selection += 1
                    }
                    KeyCode::Enter if !items.is_empty() => break,
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    _ => {}
                }
            }
        }
    }

    stdout().execute(EnableBlinking)?;
    stdout().execute(Show)?;
    let list = main_mutex_clone.lock().unwrap();
    if list.is_empty() {
        return Err(io::Error::new(io::ErrorKind::NotFound, "No peers found"));
    }
    //println!("selected: {}", list[selection].to_string());
    Ok(list[selection].to_string())
}


pub fn find_ipv4_broadcast_address(ip: Ipv4Addr, mask: Ipv4Addr) -> Ipv4Addr {
    let inverted_mask = !mask.to_bits();

    let final_bits = ip.to_bits() | inverted_mask;

    Ipv4Addr::from_bits(final_bits)
}

pub fn to_ipv4(ip: IpAddr) -> Option<Ipv4Addr> {
    match ip {
        IpAddr::V4(ipv4) => Some(ipv4),
        IpAddr::V6(_) => None,
    }
}

pub fn get_netmask(ip: Ipv4Addr) -> Option<IpAddr> {
    for interface in getifaddrs().unwrap() {
        if let Some(ip_addr) = interface.address.ip_addr() {
            if ip_addr == ip {
                if let Some(netmask) = interface.address.netmask() {
                    return Some(netmask);
                }
            }
        }
    }

    None
}

// RECEIVING STUFF -------------------------------------------------------------------

/*
pub async fn receive(ip: &Ipv4Addr) -> Result<TcpStream> {
    listen_tcp(ip).await
}

*/

pub async fn send_message(stream: &mut TcpStream) {
    print_now(&"you> ".to_string());

    io::stdout().flush().unwrap();

    let my_tcp_message = thread::spawn(|| {
        let mut response = String::new();

        std::io::stdin()
            .read_line(&mut response)
            .expect("Failed to read line");

        response
    })
    .join()
    .unwrap();

    stream.write_all(my_tcp_message.as_bytes()).await.unwrap();
}

pub async fn listen_for_message(stream: &mut TcpStream) -> u8 {
    let mut buf = [0u8; 1024];
    let bytes_read = stream.read(&mut buf).await.unwrap();
    if bytes_read == 0 {
        return 1;
    }

    println!("remote> {}", String::from_utf8_lossy(&buf[..bytes_read]));

    0
}

pub fn prompt_user(prompt: String) -> String {
    let mut ret = String::new();
    ret = prompt + &ret;
    print_now(&ret);

    let mut response = String::new();
    io::stdin()
        .read_line(&mut response)
        .expect("Couldn't read the line");
    response
}

pub async fn start_chat(stream: TcpStream) {
    clear_terminal();
    print_now(&clear_terminal());
    print_now(&move_cursor_bottom());

    let (mut reader_stream, mut writer_stream) = stream.into_split();
    let (transmitter, mut receiver) = mpsc::channel(100);
    let transmitter2 = transmitter.clone();

    let _sender_handle = tokio::spawn(async move {
        loop {
            let message = prompt_user(String::from("you> "));

            if message.trim().is_empty() {
                continue;
            }
            if message.trim() == String::from("/q") {
                transmitter.send(String::from("user")).await.unwrap();
                break;
            }
            if let Err(e) = writer_stream.write_all(message.as_bytes()).await {
                eprintln!("Error sending message: {e}");
                break;
            }
        }
    });

    let _receive_handle = tokio::spawn(async move {
        let mut buf = [0u8; 1024];
        loop {
            match reader_stream.read(&mut buf).await {
                Ok(0) => {
                    transmitter2
                        .send(String::from("remote peer"))
                        .await
                        .unwrap();
                    break;
                }
                Ok(bytes_read) => {
                    let msg = String::from_utf8_lossy(&buf[..bytes_read]);
                    print_now(&clear_line());
                    println!("remote> {}", msg.trim());
                    print!("you> ");
                    io::stdout().flush().unwrap();
                }
                Err(e) => {
                    eprintln!("Error reading: {}", e);
                    break;
                }
            }
        }
    });

    let check_signal_handle = tokio::spawn(async move {
        // Check for quit signal
        while let Some(msg) = receiver.recv().await {
            println!("\nChat closed by {msg}.");
            break;
        }
    });

    check_signal_handle.await.unwrap();
}

// TERMINAL CHAT INTERFACE --------------------------------------------------------------

// TODO replace all these functions with crossterm functions
pub fn print_now(s: &String) {
    print!("{s}");
    io::stdout().flush().unwrap();
}

pub fn move_cursor_one_row_down() -> String {
    format!("{}[1;E", START_BYTE)
}

pub fn move_cursor_bottom() -> String {
    format!("{}[999;H", START_BYTE)
}

pub fn clear_terminal() -> String {
    format!("{}[2J", START_BYTE)
}

pub fn clear_line() -> String {
    format!("\r{}[K", START_BYTE)
}
