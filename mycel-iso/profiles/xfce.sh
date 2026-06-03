PROFILE_NAME="XFCE"
PROFILE_ENV="xfce"
PROFILE_ISO_LABEL="MYCELOS_XFCE"
PROFILE_NEEDS_X11=true

install_de_packages() {
    # Xorg
    for pkg in xorg-server xorg-xinit xorg-xrandr \
                xf86-video-fbdev xf86-video-vesa; do
        fetch_arch_pkg "$pkg"
    done

    # XFCE core
    for pkg in xfce4 xfce4-goodies; do
        fetch_arch_pkg "$pkg"
    done

    # Apps
    for pkg in thunar mousepad ristretto parole \
                file-roller galculator; do
        fetch_arch_pkg "$pkg"
    done

    # Fonts and theming
    for pkg in noto-fonts noto-fonts-emoji papirus-icon-theme \
                arc-gtk-theme; do
        fetch_arch_pkg "$pkg"
    done

    # Audio
    for pkg in pipewire pipewire-audio pipewire-alsa wireplumber \
                pavucontrol; do
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
environment = "xfce"

[services]
enable = [
  "pipewire",
  "wireplumber",
  "NetworkManager",
  "bluetooth",
]
EOF
}
