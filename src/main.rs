/*
The gameplan:
- Get the network address
- For every possible host on the network, see if there is a listener
  listening on port 14953.
- If they are listening `connect()`.
- If the user is listening for data, they will be given data by
  the sender who is listening for receivers on the network. 
 */


use std::io;
use std::io::Result;
use std::net::UdpSocket;
use std::net::{ IpAddr, Ipv4Addr };
use local_ip_address::local_ip;
use getifaddrs::getifaddrs;


const PORT : i16 = 14953;

fn main() -> Result<()>
{

    // what's my IP
    let my_ipv4 : Ipv4Addr = to_ipv4(local_ip().unwrap())
        .unwrap();

    // what's the network's netmask?
    let my_netmask : Ipv4Addr = to_ipv4(get_netmask(my_ipv4).unwrap())
        .unwrap();

    // what is the broadcast address on the network?
    let broadcast_addr = find_ipv4_broadcast_address(my_ipv4, my_netmask);

    let out_sock = UdpSocket::bind(my_ipv4.to_string() + ":" + &PORT.to_string())?;

    // can't send packets to the broadcast address without this.
    out_sock.set_broadcast(true).expect("set_broadcast call failed");


    out_sock.connect(broadcast_addr.to_string() + ":" + &PORT.to_string()).expect("Couldn't connect");

    let data = b"Hello there server";
    out_sock.send(data).expect("Couldn't send message");

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

fn get_netmask(ip: Ipv4Addr) -> io::Result<IpAddr>
{
    for interface in getifaddrs()? 
    {
        if let Some(ip_addr) = interface.address.ip_addr()
        {
            if ip_addr == ip
            {
                if let Some(netmask) = interface.address.netmask()
                {
                    return Ok(netmask);
                }
            }

        }
    }

    Ok(IpAddr::V4(Ipv4Addr::new(127,0,0,1)))
}
