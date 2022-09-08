# mitto
A file upload server compatible with ShareX.


# Debian package

1. `cargo install cargo-deb`
2. `rustup target add x86_64-unknown-linux-musl` (while on Windows; musl is more portable)
3. `cargo deb --target x86_64-unknown-linux-musl`
4. `dpkg -i target/debian/*.deb`
5. `dpkg -e target/debian/*.deb` to inspect the systemd scripts
6. Update the config in "/home/sandrohc/.config/mitto" and restart the service via `systemctl restart mitto.service`