# Mitto

A file upload server compatible with ShareX.

![Mitto configuration on ShareX](files/mitto.png?raw=true)

## Usage

Start by installing Mitto on your server and configuring the file "/home/$USER/.config/mitto/mitto.toml" to your liking. Then open ShareX and go to "Destinations > Custom uploader
settings... > Import > From URL...". On the URL field, place "https://your-site.com/sharex" and press OK. On the Headers section, update the "Authorization" header to read "Basic {base64:USER:PASS}" replacing "USER" with any name and "PASS" with the value defined on the `auth_token` key in the `mitto.toml` configuration file.

## Building for Debian

1. `cargo install cargo-deb`
2. `rustup target add x86_64-unknown-linux-musl` (while on Windows; musl is more portable)
3. `cargo deb --target x86_64-unknown-linux-musl`
4. Install the package with `dpkg -i target/debian/*.deb`
5. Inspect the package with `dpkg -e target/debian/*.deb` to inspect the systemd scripts
6. Update the config file "/home/$USER/.config/mitto/mitto.toml" and restart the service via `systemctl restart mitto.service`
7. And enable the service if not already enabled: `systemctl enable mitto.service`. This will start the service on host startup.

### Useful links

1. https://www.ebbflow.io/blog/vending-linux-1
2. https://www.ebbflow.io/blog/vending-linux-2