#!/bin/bash
set -e

echo "Building oxidebar..."
cargo build --release

echo "Installing oxidebar to /usr/local/bin..."
sudo cp target/release/oxidebar /usr/local/bin/

echo "Creating config directory..."
mkdir -p ~/.config/oxidebar

if [ ! -f ~/.config/oxidebar/config.toml ]; then
    echo "No config found, oxidebar will create one on first run."
else
    echo "Config already exists at ~/.config/oxidebar/config.toml"
fi

echo ""
echo "Installation complete!"
echo "Run 'oxidebar' to start the status bar."
echo "Config is at: ~/.config/oxidebar/config.toml"
