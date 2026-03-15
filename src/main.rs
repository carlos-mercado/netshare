use crossterm::{
    ExecutableCommand, 
    cursor::{ EnableBlinking, Hide },
    event::{self, Event, KeyCode, poll},
    style::{Attribute, Color, Stylize},
    terminal::{ Clear, disable_raw_mode, enable_raw_mode }
};

use netshare::*;
use std::io::{Result, Write, stdout};
use local_ip_address::local_ip;
use std::net::{ Ipv4Addr };
use std::time::Duration;

fn main() -> Result<()>
{
    // what's my IP
    let my_ipv4 : Ipv4Addr = to_ipv4(local_ip().unwrap())
        .unwrap();

    enable_raw_mode()?; // Enter raw mode
    stdout().execute(Clear(crossterm::terminal::ClearType::All))?;
    stdout().execute(Hide)?;

    let logo = "NETSHARE"
        .with(Color::Yellow)
        .on(Color::Blue)
        .attribute(Attribute::Bold);

    let menu_items = vec![
        "Listen", 
        "Send",
    ];
    let mut selection = 0;

    stdout().execute(crossterm::cursor::MoveTo(0, 0))?;
    stdout().write_all(format!("{}\r\n", logo).as_bytes())?;
    loop 
    {
        stdout().execute(crossterm::cursor::MoveTo(0, 2))?;

        for (i, item) in menu_items.iter().enumerate() 
        {
            if i == selection 
            {
                stdout().write_all(b"> ")?;
            } 
            else 
            {
                stdout().write_all(b"  ")?;
            }
            stdout().write_all(item.as_bytes())?;
            stdout().write_all(b"\r\n")?;
        }
        stdout().flush()?;

        if poll(Duration::from_millis(100))? 
        {
            if let Event::Key(key_event) = event::read()? 
            {
                match key_event.code 
                {
                    KeyCode::Char('k') if selection > 0 => selection -= 1,
                    KeyCode::Char('j') if selection < menu_items.len() - 1 => selection += 1,
                    KeyCode::Enter => break,
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    _ => {}
                }
            }
        }
    }

    stdout().execute(EnableBlinking)?;
    stdout().execute(Hide)?;
    disable_raw_mode()?; // Revert to original terminal mode on exit


    match menu_items[selection]
    {
        "Listen" => 
        {
            println!("Entering listening mode...");
            receive(&my_ipv4)?;

        },
        "Send" => 
        {
            println!("Entering sending mode...");
            sender(&my_ipv4);
        },
        _ => panic!("Bad input.\n"),
    }


    Ok(())
}

/*
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
 * */
