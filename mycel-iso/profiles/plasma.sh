PROFILE_NAME="KDE Plasma"
PROFILE_ENV="plasma"
PROFILE_ISO_LABEL="MYCELOS_PLASMA"
PROFILE_NEEDS_X11=false

install_de_packages() {
    # KDE Plasma 6 (Wayland)
    for pkg in plasma-meta plasma-wayland-session; do
        fetch_arch_pkg "$pkg"
    done

    # KDE essential apps
    for pkg in konsole dolphin kate ark gwenview okular \
                spectacle kdeconnect plasma-browser-integration \
                packagekit-qt6 discover; do
        fetch_arch_pkg "$pkg"
    done

    # Wayland support
    for pkg in xdg-desktop-portal-kde qt6-wayland; do
        fetch_arch_pkg "$pkg"
    done

    # Fonts and theming
    for pkg in breeze-icons noto-fonts noto-fonts-emoji; do
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
