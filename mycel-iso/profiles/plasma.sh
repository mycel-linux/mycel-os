PROFILE_NAME="KDE Plasma"
PROFILE_ENV="plasma"
PROFILE_ISO_LABEL="MYCELOS_PLASMA"
PROFILE_NEEDS_X11=false

install_de_packages() {
    # Core Plasma 6 Wayland session. plasma-desktop pulls plasma-workspace,
    # kwin, kscreen, etc. We deliberately avoid plasma-meta (the kitchen sink)
    # to keep the closure and ISO size sane — extra apps can be added later.
    for pkg in plasma-desktop plasma-pa plasma-nm powerdevil \
                kscreen kde-gtk-config; do
        fetch_arch_pkg "$pkg"
    done

    # Essential apps
    for pkg in konsole dolphin kate; do
        fetch_arch_pkg "$pkg"
    done

    # Wayland support + bits a real Plasma session needs that aren't hard deps:
    # Xwayland (X11 apps), xcb-cursor (Qt xcb plugin), GSettings schemas (GTK
    # apps and portals SIGABRT without them), xrdb.
    for pkg in xdg-desktop-portal-kde qt6-wayland xorg-xwayland \
                xcb-util-cursor gsettings-desktop-schemas xorg-xrdb; do
        fetch_arch_pkg "$pkg"
    done

    # Fonts and theming
    for pkg in breeze breeze-icons noto-fonts noto-fonts-emoji; do
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

    # VM guest integration — clipboard sharing + auto-resize under SPICE/QEMU.
    # spice-vdagent ships an /etc/xdg/autostart entry, so Plasma starts it.
    for pkg in spice-vdagent; do
        fetch_arch_pkg "$pkg"
    done
}

profile_desktop_section() {
    cat <<'EOF'
[desktop]
environment = "plasma"

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
