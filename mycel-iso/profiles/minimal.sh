PROFILE_NAME="Minimal (no desktop)"
PROFILE_ENV="none"
PROFILE_ISO_LABEL="MYCELOS_MINIMAL"
PROFILE_NEEDS_X11=false

install_de_packages() {
    # No desktop environment — just useful CLI tools
    for pkg in htop tree ncdu tmux rsync openssh; do
        fetch_arch_pkg "$pkg"
    done
}

profile_desktop_section() {
    cat <<'EOF'
[desktop]
environment = "none"

[services]
enable = [
  "NetworkManager",
  "cronie",
  "sshd",
]
EOF
}
