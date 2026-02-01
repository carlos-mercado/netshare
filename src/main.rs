/*
RECEIVER:
- Listen for UDP connections at port 14953.
- You will get a message asking for your ip.
- Respond with ip via UDP.
- After this the sender will estabish a tcp connection
- The sender will send you a message
- Chat.
- Fin.

SENDER:
- Send the prompt message to the network broadcast address *DONE*
- If you get a response, it will be the remote machine's ip address *DONE*
- Establish a tcp connection with that machine. *DONE*
- Send messages back and forth with the remote machine *DONE?*.


*/

use core::{panic};
use std::io::{self, Read, Result, Write};
use std::net::{ SocketAddr, TcpListener, TcpStream, UdpSocket };
use std::net::{ IpAddr, Ipv4Addr };
use std::str::{from_utf8, FromStr};
use local_ip_address::local_ip;
use getifaddrs::getifaddrs;


const PORT : i16 = 14953;

fn main() -> Result<()>
{
    // what's my IP
    let my_ipv4 : Ipv4Addr = to_ipv4(local_ip().unwrap())
        .unwrap();

    let mut mode = String::new();

    println!("
        ███╗   ██╗ ███████╗ ████████╗ ███████╗ ███████╗ ███╗   ██╗ ██████╗ 
        ████╗  ██║ ██╔════╝ ╚══██╔══╝ ██╔════╝ ██╔════╝ ████╗  ██║ ██╔══██╗
        ██╔██╗ ██║ █████╗      ██║    ███████╗ █████╗   ██╔██╗ ██║ ██║  ██║
        ██║╚██╗██║ ██╔══╝      ██║    ╚════██║ ██╔══╝   ██║╚██╗██║ ██║  ██║
        ██║ ╚████║ ███████╗    ██║    ███████║ ███████╗ ██║ ╚████║ ██████╔╝
        ╚═╝  ╚═══╝ ╚══════╝    ╚═╝    ╚══════╝ ╚══════╝ ╚═╝  ╚═══╝ ╚═════╝ ");

    print!("\n\nEnter 'listen' for listening mode, or 'send' for sending mode: ");
    io::stdout().flush().unwrap();
    io::stdin()
        .read_line(&mut mode)
        .expect("Failed to read line");

    match mode.trim() 
    {
        "listen" => 
        {
            println!("Entering listening mode...");
            receive(&my_ipv4)?;

        },
        "send" => 
        {
            println!("Entering sending mode...");
            sender(&my_ipv4);
        },
        _ => panic!("Bad input.\n"),
    }


    Ok(())
}

// SENDER STUFF -------------------------------------------------------------------

fn sender(user_ip: &Ipv4Addr)
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

fn estabish_tcp(remote_ip: &Ipv4Addr) -> Result<()>
{
    let mut stream : TcpStream = TcpStream::connect(remote_ip.to_string() + ":" + &PORT.to_string())?;

    clearscreen::clear().expect("Failed to clear screen");
    println!("Connected with *user*!\n");

    loop
    {
        send_message(&mut stream);

        if listen_for_message(&mut stream) == 1
        {
            break;
        }
    }


    Ok(())
}

fn get_remote_ip(ip: &Ipv4Addr) -> Option<String>
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


fn broadcast_init_message(ip: &Ipv4Addr) -> Result<()>
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


fn find_ipv4_broadcast_address(ip: Ipv4Addr, mask: Ipv4Addr) -> Ipv4Addr
{
    let inverted_mask = !mask.to_bits();

    let final_bits = ip.to_bits() | inverted_mask;

    Ipv4Addr::from_bits(final_bits)
}

fn to_ipv4(ip: IpAddr) -> Option<Ipv4Addr>
{
    match ip
    {
        IpAddr::V4(ipv4) => Some(ipv4),
        IpAddr::V6(_) => None,
    }
}

fn get_netmask(ip: Ipv4Addr) -> Option<IpAddr>
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

fn receive(ip: &Ipv4Addr) -> Result<()>
{
    match listen_and_respond(ip)
    {
        Ok(_) => println!("Listen Success"),
        Err(_) => println!("Listen Failure"),
    }

    Ok(())
}

fn listen_and_respond(ip: &Ipv4Addr) -> Result<()>
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
fn listen_tcp(local_ip: &Ipv4Addr) -> Result<()>
{
    let listener = TcpListener::bind(format!("{local_ip}:{PORT}"))?;
    let (mut stream, _) = listener.accept()?;

    clearscreen::clear().expect("Failed to clear screen");

    loop
    {
        send_message(&mut stream);

        let s = listen_for_message(&mut stream); if s == 1 { break; }
    }

    Ok(())
}

fn send_message(stream : &mut TcpStream)
{
    let mut my_tcp_message : String = String::new();

    print!("you> ");
    io::stdout().flush().unwrap();
    io::stdin()
        .read_line(&mut my_tcp_message)
        .expect("Failed to read line");

    stream.write(&my_tcp_message[..].as_bytes()).unwrap();
}


fn listen_for_message(stream : &mut TcpStream) -> u8
{

    loop
    {
        let mut buf = [0u8; 1024];
        let bytes_read = stream.read(&mut buf).unwrap();

        if bytes_read == 0
        {
            break;
        }

        println!("remote> {}", String::from_utf8_lossy(&buf[..bytes_read]));
    }

    1
}
