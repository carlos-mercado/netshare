use core::{panic};
use std::io::{self, Read, Result, Write};
use std::net::{ SocketAddr, TcpListener, TcpStream, UdpSocket };
use std::net::{ IpAddr, Ipv4Addr };
use std::str::{from_utf8, FromStr};
use std::sync::{Mutex, Arc};
use std::thread;
use getifaddrs::getifaddrs;


const PORT : i16 = 14953;
const START_BYTE : char = '\x1b';

// SENDER STUFF -------------------------------------------------------------------

pub fn sender(user_ip: &Ipv4Addr)
{
    match broadcast_init_message(&user_ip)
    {
        Ok(_) => println!("Successfully broadcasted message."),
        Err(e) => panic!("{e}"),
    }

    let remote_addres =  get_remote_ip(&user_ip).unwrap();
match estabish_tcp(&Ipv4Addr::from_str(&remote_addres).unwrap())
    {
        Ok(_) => println!("Successfully established TCP connection with remote IP"),
        Err(_) => panic!("Could not get establish TCP connection with remote IP."),
    }
}

pub fn estabish_tcp(remote_ip: &Ipv4Addr) -> Result<()>
{
    let stream : TcpStream = TcpStream::connect(remote_ip.to_string() + ":" + &PORT.to_string())?;

    println!("Connected with *user*!\n");

    start_chat(stream);


    Ok(())
}

pub fn get_remote_ip(ip: &Ipv4Addr) -> Option<String>
{
    let mut listeners: Vec<SocketAddr> = Vec::new();

    let in_sock : UdpSocket = UdpSocket::bind(ip.to_string() + ":" + &PORT.to_string())
        .expect("couldn't bind to address");

    let mut buff = [0; 16];
    let (number_of_bytes, src_addr) = in_sock.recv_from(&mut buff)
                                            .expect("Didn't receive data");

    listeners.push(src_addr);

    let filled = &mut buff[..number_of_bytes];

    Some(from_utf8(filled).unwrap().to_string())
}


pub fn broadcast_init_message(ip: &Ipv4Addr) -> Result<()>
{
    // what's the network's netmask?
    let my_netmask : Ipv4Addr = to_ipv4(get_netmask(*ip).unwrap())
        .unwrap();

    // what is the broadcast address on the network?
    let broadcast_addr : Ipv4Addr = find_ipv4_broadcast_address(*ip, my_netmask);

    let out_sock : UdpSocket = UdpSocket::bind(ip.to_string() + ":" + &PORT.to_string())?;

    // can't send packets to the broadcast address without this.
    out_sock.set_broadcast(true).expect("set_broadcast call failed");

    out_sock.connect(broadcast_addr.to_string() + ":" + &PORT.to_string()).expect("Couldn't connect");
    out_sock.send(b"Hey there client!, mind sending me your ip?").expect("Couldn't send message");


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
        Err(_) => println!("Listen Failure"),
    }

    Ok(())
}

pub fn listen_and_respond(ip: &Ipv4Addr) -> Result<()>
{
    let listener = UdpSocket::bind("0.0.0.0:".to_string() + &PORT.to_string())?;

    let mut buf = [0; 128];

    let (_, src_addr) = listener.recv_from(&mut buf)
                                                .expect("Did not receive data!");

    let ip_string = ip.to_string();
    let ip_message: &[u8] = ip_string.as_bytes();

    listener.send_to(&ip_message, src_addr)?;

    listen_tcp(ip)?;

    Ok(())
}

// TODO listening and submitting should happen simultaneously (multithreaded)
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

    let mut b = String::new();
    b += &move_cursor_bottom();
    print_now(&b);

    let protected_stream = Arc::new(Mutex::new(stream));

    let sender_stream = Arc::clone(&protected_stream);
    let send_handle = thread::spawn(move || {
        loop 
        {
            let message = prompt_user(String::from("you> "));
            sender_stream.lock().unwrap().write(&message[..].as_bytes()).unwrap();
        }

    });

    let receiver_stream = Arc::clone(&protected_stream);
    let receive_handle = thread::spawn(move || {
        loop
        {
            let mut buf = [0u8; 1024];
            let bytes_read = receiver_stream.lock().unwrap().read(&mut buf).unwrap();

            if bytes_read == 0
            {
                return 1;
            }

            println!("remote> {}", String::from_utf8_lossy(&buf[..bytes_read]));
        }
    });

    send_handle.join().unwrap();
    receive_handle.join().unwrap();
}

// TERMINAL CHAT INTERFACE --------------------------------------------------------------

pub fn print_now(s: &String) { print!("{s}"); io::stdout().flush().unwrap(); }

pub fn move_cursor_one_row_down() -> String { format!("{}[1;E", START_BYTE) }

pub fn move_cursor_bottom() -> String { format!("{}[999;H", START_BYTE) }

pub fn clear_terminal() -> String { format!("{}[2J", START_BYTE) }
