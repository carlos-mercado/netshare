use crossterm::{
    ExecutableCommand, 
    cursor::{ DisableBlinking, EnableBlinking, Hide, Show }, 
    event::{self, Event, KeyCode, poll}, 
    terminal::{ Clear, disable_raw_mode, enable_raw_mode }
};
use std::{ io::stdout };
use std::io::{ self, Read, Result, Write };
use std::net::{ SocketAddr, TcpListener, TcpStream, UdpSocket };
use std::net::{ IpAddr, Ipv4Addr };
use std::sync::{ mpsc, Arc, Mutex };
use std::thread;
use getifaddrs::getifaddrs;
use std::time::Duration;

const START_BYTE : char = '\x1b';
const PORT : i16 = 14953;

// SENDER STUFF -------------------------------------------------------------------
pub fn sender(user_ip: &Ipv4Addr)
{
    let remote_address =  get_remote_ip(&user_ip).unwrap();

    match estabish_tcp(remote_address) {
        Ok(_) => println!("Successfully established TCP connection with remote IP"),
        Err(e) => panic!("Could not get establish TCP connection with remote IP. Error {e}"),
    }
}

pub fn get_remote_ip(ip: &Ipv4Addr) -> std::io::Result<String>
{
    enable_raw_mode()?; // Enter raw mode
    stdout().execute(Clear(crossterm::terminal::ClearType::All))?;
    stdout().execute(DisableBlinking)?;
    stdout().execute(Hide)?;

    let mut selection = 0;
    let listeners: Vec<SocketAddr> = Vec::new();
    let m = Arc::new(Mutex::new(listeners));
    let main_mutex_clone = Arc::clone(&m);
    let vec_mutex_clone = Arc::clone(&m);

    let in_sock : UdpSocket = UdpSocket::bind(ip.to_string() + ":" + &PORT.to_string())
        .expect("couldn't bind to address");
    let in_socket = Arc::new(in_sock);
    let listener_clone = Arc::clone(&in_socket);
    let broadcaster_clone = Arc::clone(&in_socket);
    let ip_clone = ip.clone();

    let _rebroadcaster_handle = thread::spawn(move || {
        let my_netmask : Ipv4Addr = match get_netmask(ip_clone) {
            Some(res) => to_ipv4(res).unwrap(),
            None => Ipv4Addr::new(255, 255, 255, 0),
        };
        let broadcast_addr : Ipv4Addr = find_ipv4_broadcast_address(ip_clone, my_netmask);
        broadcaster_clone.set_broadcast(true)
            .expect("set_broadcast call failed");

        loop {
            broadcaster_clone.send_to(b"Hey there client!, mind sending me your ip?", broadcast_addr.to_string() + ":" + &PORT.to_string())
                .expect("Couldn't send broadcast message");
            std::thread::sleep(std::time::Duration::from_secs(2));
        }
    });

    let _listener_handle = thread::spawn(move || {
        let my_addr = listener_clone.local_addr().unwrap();
        loop {
            // might need to make this buff bigger for windows
            let mut buff = [0; 64];
            let (_, src_addr) = listener_clone.recv_from(&mut buff)
                .expect("Didn't receive data");
            if src_addr != my_addr {
                (vec_mutex_clone.lock().unwrap()).push(src_addr);
            }
        }
    });


    // this is for windows powershell, does not work without it.
    while event::poll(Duration::from_millis(0))? { let _ = event::read(); }
    loop {
        stdout().execute(crossterm::cursor::MoveTo(0, 0))?;

        let items = {
            let list = main_mutex_clone.lock().unwrap();
            list.clone()
        };

        if items.is_empty()
        {
            let loading_string = format!("Finding users...\r\n");

            stdout().write_all(loading_string.as_bytes())?;
            stdout().flush()?;
            while event::poll(Duration::from_millis(0))? { let _ = event::read(); }
            thread::sleep(Duration::from_millis(200));
        }

        for (i, item) in items.iter().enumerate() {
            if i == selection {
                stdout().write_all(b"> ")?;
            }
            else {
                stdout().write_all(b"  ")?;
            }
            stdout().write_all(&item.to_string().as_bytes())?;
            stdout().write_all(b"\r\n")?;
        }
        stdout().flush()?;

        if poll(Duration::from_millis(100))? {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Char('k') | KeyCode::Up if selection > 0 => selection -= 1,
                    KeyCode::Char('j') | KeyCode::Down if selection < items.len() - 1 => selection += 1,
                    KeyCode::Enter => break,
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    _ => {}
                }
            }
        }
    }

    let selected = &main_mutex_clone.lock().unwrap()[selection];
    stdout().execute(EnableBlinking)?;
    stdout().execute(Show)?;
    disable_raw_mode()?; // Revert to original terminal mode on exit
    Ok(selected.to_string())
}

pub fn estabish_tcp(remote_ip: String) -> Result<()>
{
    let stream = TcpStream::connect(&remote_ip).unwrap();

    println!("Connected with *{remote_ip}*!\n");

    start_chat(stream);

    Ok(())
}

pub fn find_ipv4_broadcast_address(ip: Ipv4Addr, mask: Ipv4Addr) -> Ipv4Addr
{
    let inverted_mask = !mask.to_bits();

    let final_bits = ip.to_bits() | inverted_mask;

    Ipv4Addr::from_bits(final_bits)
}

pub fn to_ipv4(ip: IpAddr) -> Option<Ipv4Addr>
{
    match ip
    {
        IpAddr::V4(ipv4) => Some(ipv4),
        IpAddr::V6(_) => None,
    }
}

pub fn get_netmask(ip: Ipv4Addr) -> Option<IpAddr>
{
    for interface in getifaddrs().unwrap()
    {
        if let Some(ip_addr) = interface.address.ip_addr()
        {
            if ip_addr == ip
            {
                if let Some(netmask) = interface.address.netmask()
                {
                    return Some(netmask);
                }
            }

        }
    }

    None
}

// RECEIVING STUFF -------------------------------------------------------------------

pub fn receive(ip: &Ipv4Addr) -> Result<()>
{
    match listen_and_respond(ip)
    {
        Ok(_) => println!("Listen Success"),
        Err(e) => println!("Listen Failure: {e}"),
    }

    Ok(())
}

pub fn listen_and_respond(ip: &Ipv4Addr) -> Result<()>
{
    let listener = UdpSocket::bind("0.0.0.0:".to_string() + &PORT.to_string())?;
    listener.set_nonblocking(true)
        .expect("couldn't set listener socket to non-blocking");

    let mut buf = [0; 128];

    loop {
        match listener.recv_from(&mut buf) {
            Ok((_, src_addr)) => {
                let ip_string = ip.to_string();
                let ip_message: &[u8] = ip_string.as_bytes();

                listener.send_to(&ip_message, src_addr)?;

                listen_tcp(ip)?;

                break;
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(50));
            }
            Err(e) => return Err(e.into()),
        }
    }


    Ok(())
}

pub fn listen_tcp(local_ip: &Ipv4Addr) -> Result<()>
{
    let listener = TcpListener::bind(format!("{local_ip}:{PORT}"))?;
    let (stream, _) = listener.accept()?;


    start_chat(stream);

    Ok(())
}


pub fn send_message(stream : &mut TcpStream)
{
    let mut my_tcp_message : String = String::new();

    print!("you> ");
    io::stdout().flush().unwrap();
    io::stdin()
        .read_line(&mut my_tcp_message)
        .expect("Failed to read line");

    stream.write(&my_tcp_message[..].as_bytes()).unwrap();
}


pub fn listen_for_message(stream : &mut TcpStream) -> u8
{

    let mut buf = [0u8; 1024];
    let bytes_read = stream.read(&mut buf).unwrap();

    if bytes_read == 0
    {
        return 1;
    }

    println!("remote> {}", String::from_utf8_lossy(&buf[..bytes_read]));

    0
}



pub fn prompt_user(prompt: String) -> String
{
    let mut ret = String::new();

    // prompt consumed here
    ret = prompt + &ret;

    print_now(&ret);

    let mut response = String::new();

    io::stdin()
        .read_line(&mut response)
        .expect("Coudn't read the line");

    response
}

pub fn start_chat(stream: TcpStream)
{
    clear_terminal();
    print_now(&clear_terminal());
    print_now(&move_cursor_bottom());

    let mut reader_stream = stream.try_clone().expect("Failed to clone stream");
    let mut writer_stream = stream;

    reader_stream.set_read_timeout(Some(Duration::from_millis(200))).unwrap();


    let (transmitter, receiver) = mpsc::channel();

    let _send_handle = thread::spawn(move || {
        loop 
        {
            let message = prompt_user(String::from("you> "));

            if message.trim().is_empty() { continue; }

            if message.trim() == String::from("/q")
            {
                transmitter.send(String::from("Quit")).unwrap();
                break;
            }

            if let Err(e) = writer_stream.write_all(message.as_bytes())
            {
                eprintln!("Error sending message: {e}");
                break;
            }
        }

    });

    let receive_handle = thread::spawn(move || {
        let mut buf = [0u8; 1024];
        loop {
            // Check for quit signal
            if let Ok(msg) = receiver.try_recv() 
            {
                if msg == "Quit" 
                {
                    println!("\nChat closed by user.");
                    break;
                }
            }

            match reader_stream.read(&mut buf) {
                Ok(0) => {
                    println!("\nConnection closed by remote peer.");
                    break;
                }
                Ok(bytes_read) => {
                    let msg = String::from_utf8_lossy(&buf[..bytes_read]);
                    print_now(&clear_line());
                    println!("remote> {}", msg.trim());
                    print!("you> ");
                    io::stdout().flush().unwrap();
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock || e.kind() == io::ErrorKind::TimedOut => {
                    // Timeout, loop again to check channel
                    continue;
                }
                Err(e) => {
                    eprintln!("Error reading: {}", e);
                    break;
                }
            }
        }
    });

    receive_handle.join().unwrap();
}

// TERMINAL CHAT INTERFACE --------------------------------------------------------------

pub fn print_now(s: &String) { print!("{s}"); io::stdout().flush().unwrap(); }

pub fn move_cursor_one_row_down() -> String { format!("{}[1;E", START_BYTE) }

pub fn move_cursor_bottom() -> String { format!("{}[999;H", START_BYTE) }

pub fn clear_terminal() -> String { format!("{}[2J", START_BYTE) }

pub fn clear_line() -> String { format!("\r{}[K", START_BYTE) }
