//! Address book management tool

use trinitychain::addressbook::{self};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_usage();
        std::process::exit(1);
    }

    let command = &args[1];

    match command.as_str() {
        "add" => add_address(&args[2..])?,
        "remove" | "rm" => remove_address(&args[2..])?,
        "list" | "ls" => list_addresses()?,
        "search" => search_addresses(&args[2..])?,
        "get" => get_address(&args[2..])?,
        _ => {
            eprintln!("Unknown command: {}", command);
            print_usage();
            std::process::exit(1);
        }
    }

    Ok(())
}

fn print_usage() {
    println!("📒 Address Book Management\n");
    println!("Usage: trinity-addressbook <command> [arguments]\n");
    println!("Commands:");
    println!("  add <label> <address> [notes]   Add a new address");
    println!("  remove <label>                   Remove an address");
    println!("  list                             List all addresses");
    println!("  search <query>                   Search addresses");
    println!("  get <label>                      Get specific address");
    println!("\nExamples:");
    println!("  trinity-addressbook add Alice abc123... \"My friend\"");
    println!("  trinity-addressbook list");
    println!("  trinity-addressbook search Alice");
}

fn add_address(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.len() < 2 {
        eprintln!("Usage: trinity-addressbook add <label> <address> [notes]");
        std::process::exit(1);
    }

    let label = &args[0];
    let address = &args[1];
    let notes = if args.len() > 2 {
        Some(args[2..].join(" "))
    } else {
        None
    };

    let mut book = addressbook::load_default()?;
    book.add(label.clone(), address.clone(), notes.clone())?;
    addressbook::save_default(&book)?;

    println!("✅ Address added successfully!");
    println!("📌 Label: {}", label);
    println!("📍 Address: {}...", &address[..42.min(address.len())]);
    if let Some(n) = notes {
        println!("📝 Notes: {}", n);
    }

    Ok(())
}

fn remove_address(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.is_empty() {
        eprintln!("Usage: trinity-addressbook remove <label>");
        std::process::exit(1);
    }

    let label = &args[0];

    let mut book = addressbook::load_default()?;
    let entry = book.remove(label)?;
    addressbook::save_default(&book)?;

    println!("✅ Address removed successfully!");
    println!("📌 Label: {}", entry.label);
    println!(
        "📍 Address: {}...",
        &entry.address[..42.min(entry.address.len())]
    );

    Ok(())
}

fn list_addresses() -> Result<(), Box<dyn std::error::Error>> {
    let book = addressbook::load_default()?;
    let entries = book.list();

    if entries.is_empty() {
        println!("📒 Address book is empty");
        println!("\nAdd an address with: trinity-addressbook add <label> <address>");
        return Ok(());
    }

    println!("📒 Address Book ({} entries)\n", entries.len());

    for entry in entries {
        println!("─────────────────────────────────────────");
        println!("📌 Label: {}", entry.label);
        println!("📍 Address: {}", entry.address);
        if let Some(notes) = &entry.notes {
            println!("📝 Notes: {}", notes);
        }
        println!("📅 Added: {}", entry.added);
    }

    println!("─────────────────────────────────────────");

    Ok(())
}

fn search_addresses(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.is_empty() {
        eprintln!("Usage: trinity-addressbook search <query>");
        std::process::exit(1);
    }

    let query = args.join(" ");

    let book = addressbook::load_default()?;
    let results = book.search(&query);

    if results.is_empty() {
        println!("❌ No addresses found matching '{}'", query);
        return Ok(());
    }

    println!("🔍 Search Results ({} found)\n", results.len());

    for entry in results {
        println!("─────────────────────────────────────────");
        println!("📌 Label: {}", entry.label);
        println!("📍 Address: {}", entry.address);
        if let Some(notes) = &entry.notes {
            println!("📝 Notes: {}", notes);
        }
    }

    println!("─────────────────────────────────────────");

    Ok(())
}

fn get_address(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if args.is_empty() {
        eprintln!("Usage: trinity-addressbook get <label>");
        std::process::exit(1);
    }

    let label = &args[0];

    let book = addressbook::load_default()?;

    match book.get(label) {
        Some(entry) => {
            println!("📌 Label: {}", entry.label);
            println!("📍 Address: {}", entry.address);
            if let Some(notes) = &entry.notes {
                println!("📝 Notes: {}", notes);
            }
            println!("📅 Added: {}", entry.added);
        }
        None => {
            eprintln!("❌ Address not found: {}", label);
            std::process::exit(1);
        }
    }

    Ok(())
}
