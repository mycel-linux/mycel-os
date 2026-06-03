PROFILE_NAME="GNOME"
PROFILE_ENV="gnome"
PROFILE_ISO_LABEL="MYCELOS_GNOME"
PROFILE_NEEDS_X11=false

install_de_packages() {
    # GNOME core (Wayland)
    for pkg in gnome gnome-tweaks gnome-shell-extensions; do
        fetch_arch_pkg "$pkg"
    done

    # GNOME apps
    for pkg in gnome-terminal nautilus gedit eog evince \
                gnome-calculator gnome-disk-utility file-roller \
                gnome-clocks gnome-weather gnome-maps; do
        fetch_arch_pkg "$pkg"
    done

    # Wayland portal
    for pkg in xdg-desktop-portal-gnome; do
        fetch_arch_pkg "$pkg"
    done

    # Fonts
    for pkg in cantarell-fonts noto-fonts noto-fonts-emoji; do
        fetch_arch_pkg "$pkg"
    done

    # Audio
    for pkg in pipewire pipewire-audio wireplumber; do
        fetch_arch_pkg "$pkg"
    done

    # Browser
    for pkg in firefox; do
        fetch_arch_pkg "$pkg"
    done
}

profile_desktop_section() {
    cat <<'EOF'
[desktop]
environment = "gnome"

[services]
enable = [
  "pipewire",
  "wireplumber",
  "NetworkManager",
  "bluetooth",
  "cronie",
]
EOF
}
