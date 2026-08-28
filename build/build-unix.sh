#!/usr/bin/env bash
sudo -v || { echo "Could not complete build: not enough permissions."; exit 1; }
cd "$(dirname "$0")/.." || exit 1

cargo build --release
sudo cp ./target/release/rs-eval /usr/local/bin/

# Install shell completions for Unix shells (Linux & macOS)
BIN="./target/release/rs-eval"

# Bash completions
for bash_dir in "/usr/share/bash-completion/completions" "/usr/local/share/bash-completion/completions" "/etc/bash_completion.d" "/usr/local/etc/bash_completion.d" "/opt/homebrew/etc/bash_completion.d"; do
    if [ -d "$bash_dir" ]; then
        "$BIN" --generate-completion bash | sudo tee "$bash_dir/rs-eval" > /dev/null
        break
    fi
done

# Zsh completions
for zsh_dir in "/usr/local/share/zsh/site-functions" "/usr/share/zsh/site-functions" "/usr/share/zsh/vendor-completions" "/opt/homebrew/share/zsh/site-functions"; do
    if [ -d "$zsh_dir" ]; then
        "$BIN" --generate-completion zsh | sudo tee "$zsh_dir/_rs-eval" > /dev/null
        break
    fi
done

# Fish completions
for fish_dir in "/usr/share/fish/vendor_completions.d" "/usr/local/share/fish/vendor_completions.d" "/opt/homebrew/share/fish/vendor_completions.d"; do
    if [ -d "$fish_dir" ]; then
        "$BIN" --generate-completion fish | sudo tee "$fish_dir/rs-eval.fish" > /dev/null
        break
    fi
done
