//! Trinity Address Book CLI - Simple version without clap
//!
//! Command-line interface for managing TrinityChain address book

use trinitychain::addressbook::{self, AddressBook};
use trinitychain::error::ChainError;
use std::env;

fn main() -> Result<(), ChainError> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        return Ok(());
    }

    let command = &args[1];

    match command.as_str() {
        "add" => {
            if args.len() < 4 {
                println!("Usage: trinity-addressbook add <label> <address> [notes]");
                return Ok(());
            }
            let label = args[2].clone();
            let address = args[3].clone();
            let notes = if args.len() > 4 {
                Some(args[4..].join(" "))
            } else {
                None
            };

            let book = addressbook::load_default()?;
            book.add(label.clone(), address, notes)?;
            addressbook::save_default(&book)?;
            println!("✅ Added address with label '{}'", label);
        }

        "remove" | "rm" => {
            if args.len() < 3 {
                println!("Usage: trinity-addressbook remove <label>");
                return Ok(());
            }
            let label = &args[2];

            let book = addressbook::load_default()?;
            let entry = book.remove(label)?;
            addressbook::save_default(&book)?;
            println!("✅ Removed address: {} ({})", entry.label, entry.address);
        }

        "update" => {
            if args.len() < 3 {
                println!("Usage: trinity-addressbook update <label> [--address <addr>] [--notes <notes>]");
                return Ok(());
            }
            let label = &args[2];
            
            let mut new_address = None;
            let mut new_notes = None;
            
            let mut i = 3;
            while i < args.len() {
                match args[i].as_str() {
                    "--address" | "-a" => {
                        if i + 1 < args.len() {
                            new_address = Some(args[i + 1].clone());
                            i += 2;
                        } else {
                            println!("Error: --address requires a value");
                            return Ok(());
                        }
                    }
                    "--notes" | "-n" => {
                        if i + 1 < args.len() {
                            new_notes = Some(args[i + 1..].join(" "));
                            break;
                        } else {
                            println!("Error: --notes requires a value");
                            return Ok(());
                        }
                    }
                    _ => {
                        println!("Unknown option: {}", args[i]);
                        return Ok(());
                    }
                }
            }

            let book = addressbook::load_default()?;
            book.update(label, new_address, new_notes)?;
            addressbook::save_default(&book)?;
            println!("✅ Updated address with label '{}'", label);
        }

        "get" => {
            if args.len() < 3 {
                println!("Usage: trinity-addressbook get <label>");
                return Ok(());
            }
            let label = &args[2];

            let book = addressbook::load_default()?;
            match book.get(label) {
                Some(entry) => print_entry(&entry),
                None => println!("❌ Address with label '{}' not found", label),
            }
        }

        "search" => {
            if args.len() < 3 {
                println!("Usage: trinity-addressbook search <query>");
                return Ok(());
            }
            let query = args[2..].join(" ");

            let book = addressbook::load_default()?;
            let results = book.search(&query);
            
            if results.is_empty() {
                println!("No addresses found matching '{}'", query);
            } else {
                println!("Found {} result(s):\n", results.len());
                for entry in results {
                    print_entry(&entry);
                    println!(); // blank line between entries
                }
            }
        }

        "list" | "ls" => {
            let book = addressbook::load_default()?;
            let entries = book.list();
            
            if entries.is_empty() {
                println!("Address book is empty");
            } else {
                println!("📖 Address Book ({} entries):\n", entries.len());
                for entry in entries {
                    print_entry_compact(&entry);
                }
            }
        }

        "export" => {
            if args.len() < 3 {
                println!("Usage: trinity-addressbook export <path.csv>");
                return Ok(());
            }
            let path = &args[2];

            let book = addressbook::load_default()?;
            let export_path = std::path::Path::new(path);
            book.export_csv(export_path)?;
            println!("✅ Exported {} entries to {}", book.len(), path);
        }

        "stats" => {
            let book = addressbook::load_default()?;
            print_stats(&book);
        }

        "help" | "--help" | "-h" => {
            print_usage();
        }

        _ => {
            println!("Unknown command: {}", command);
            println!();
            print_usage();
        }
    }

    Ok(())
}

fn print_usage() {
    println!("Trinity Address Book - Manage TrinityChain addresses");
    println!();
    println!("USAGE:");
    println!("    trinity-addressbook <COMMAND> [OPTIONS]");
    println!();
    println!("COMMANDS:");
    println!("    add <label> <address> [notes]     Add a new address");
    println!("    remove <label>                    Remove an address (alias: rm)");
    println!("    update <label> [options]          Update an existing address");
    println!("        --address <addr>              Set new address");
    println!("        --notes <text>                Set new notes");
    println!("    get <label>                       Get address by label");
    println!("    search <query>                    Search addresses");
    println!("    list                              List all addresses (alias: ls)");
    println!("    export <path.csv>                 Export to CSV file");
    println!("    stats                             Show statistics");
    println!("    help                              Show this help message");
    println!();
    println!("EXAMPLES:");
    println!("    trinity-addressbook add Alice abc123 My friend");
    println!("    trinity-addressbook get Alice");
    println!("    trinity-addressbook update Alice --notes Updated info");
    println!("    trinity-addressbook search friend");
    println!("    trinity-addressbook list");
    println!("    trinity-addressbook export backup.csv");
}

fn print_entry(entry: &trinitychain::addressbook::AddressEntry) {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🏷️  Label: {}", entry.label);
    println!("📬 Address: {}", entry.address);
    if let Some(notes) = &entry.notes {
        println!("📝 Notes: {}", notes);
    }
    println!("📅 Created: {}", entry.created_at);
    println!("📅 Updated: {}", entry.updated_at);
    println!("🔢 Version: {}", entry.version);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
}

fn print_entry_compact(entry: &trinitychain::addressbook::AddressEntry) {
    let notes_preview = entry.notes.as_ref().map(|n| {
        if n.len() > 30 {
            format!(" - {}...", &n[..30])
        } else {
            format!(" - {}", n)
        }
    }).unwrap_or_default();
    
    println!("  {} → {}{}", 
        entry.label, 
        entry.address,
        notes_preview
    );
}

fn print_stats(book: &AddressBook) {
    println!("📊 Address Book Statistics");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📦 Total Entries: {}", book.len());
    
    let entries = book.list();
    let with_notes = entries.iter().filter(|e| e.notes.is_some()).count();
    
    println!("📝 Entries with notes: {}", with_notes);
    println!("📭 Entries without notes: {}", entries.len() - with_notes);
    
    if !entries.is_empty() {
        // Find most recently added/updated
        let newest = entries.iter().max_by(|a, b| a.created_at.cmp(&b.created_at));
        let most_updated = entries.iter().max_by(|a, b| a.updated_at.cmp(&b.updated_at));
        
        if let Some(entry) = newest {
            println!("🆕 Newest entry: {} ({})", entry.label, entry.created_at);
        }
        
        if let Some(entry) = most_updated {
            println!("🔄 Most recently updated: {} (v{}, {})", 
                entry.label, entry.version, entry.updated_at);
        }
        
        // Find entry with most updates
        let most_versions = entries.iter().max_by_key(|e| e.version);
        if let Some(entry) = most_versions {
            if entry.version > 1 {
                println!("🏆 Most updated entry: {} ({} versions)", 
                    entry.label, entry.version);
            }
        }
    }
    
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
}
