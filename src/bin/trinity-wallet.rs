// NOTE: This CLI binary has been deprecated/disabled.
// The functionality was split into focused tools: `trinity-wallet-new`,
// `trinity-wallet-backup`, and `trinity-wallet-restore` to avoid ambiguity
// and to enforce explicit wallet usage (no silent default wallets).

fn main() {
    eprintln!("The `trinity-wallet` CLI has been deprecated. Use `trinity-wallet-new`, `trinity-wallet-backup`, or `trinity-wallet-restore` instead.");
    std::process::exit(1);
}
