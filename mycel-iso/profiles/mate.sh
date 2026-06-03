PROFILE_NAME="MATE"
PROFILE_ENV="mate"
PROFILE_ISO_LABEL="MYCELOS_MATE"
PROFILE_NEEDS_X11=true

install_de_packages() {
    # Xorg
    for pkg in xorg-server xorg-xinit xorg-xrandr \
                xf86-video-fbdev xf86-video-vesa; do
        fetch_arch_pkg "$pkg"
    done

    # MATE
    for pkg in mate mate-extra; do
        fetch_arch_pkg "$pkg"
    done

    # Apps
    for pkg in caja pluma eom engrampa mate-calc; do
        fetch_arch_pkg "$pkg"
    done

    # Fonts and theming
    for pkg in noto-fonts noto-fonts-emoji papirus-icon-theme; do
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
environment = "mate"

[services]
enable = [
  "pipewire",
  "wireplumber",
  "NetworkManager",
  "bluetooth",
]
EOF
}
