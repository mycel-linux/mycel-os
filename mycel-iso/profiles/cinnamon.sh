PROFILE_NAME="Cinnamon"
PROFILE_ENV="cinnamon"
PROFILE_ISO_LABEL="MYCELOS_CINNAMON"
PROFILE_NEEDS_X11=true

install_de_packages() {
    # Xorg
    for pkg in xorg-server xorg-xinit xorg-xrandr xorg-xsetroot \
                xf86-video-fbdev xf86-video-vesa; do
        fetch_arch_pkg "$pkg"
    done

    # Cinnamon
    for pkg in cinnamon cinnamon-translations; do
        fetch_arch_pkg "$pkg"
    done

    # Cinnamon apps
    for pkg in nemo nemo-fileroller nemo-preview gedit \
                xviewer xplayer xreader gnome-calculator \
                file-roller lightdm lightdm-gtk-greeter; do
        fetch_arch_pkg "$pkg"
    done

    # Fonts and theming
    for pkg in noto-fonts noto-fonts-emoji papirus-icon-theme; do
        fetch_arch_pkg "$pkg"
    done

    # Audio
    for pkg in pipewire pipewire-audio pipewire-alsa wireplumber; do
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
environment = "cinnamon"

[services]
enable = [
  "pipewire",
  "wireplumber",
  "NetworkManager",
  "bluetooth",
]
EOF
}
