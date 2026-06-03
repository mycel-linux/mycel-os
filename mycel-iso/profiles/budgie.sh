PROFILE_NAME="Budgie"
PROFILE_ENV="budgie"
PROFILE_ISO_LABEL="MYCELOS_BUDGIE"
PROFILE_NEEDS_X11=false

install_de_packages() {
    # Budgie (Wayland via wlroots)
    for pkg in budgie-desktop budgie-extras; do
        fetch_arch_pkg "$pkg"
    done

    # Apps
    for pkg in nautilus gedit eog evince \
                gnome-calculator file-roller; do
        fetch_arch_pkg "$pkg"
    done

    # Wayland portal
    for pkg in xdg-desktop-portal-gtk; do
        fetch_arch_pkg "$pkg"
    done

    # Fonts and theming
    for pkg in noto-fonts noto-fonts-emoji papirus-icon-theme; do
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
environment = "budgie"

[services]
enable = [
  "pipewire",
  "wireplumber",
  "NetworkManager",
  "bluetooth",
]
EOF
}
