PROFILE_NAME="FessusDE"
PROFILE_ENV="fessus"
PROFILE_ISO_LABEL="MYCELOS"
PROFILE_NEEDS_X11=false

install_de_packages() {
    # Sway + FessusDE stack
    for pkg in sway swaybg swaylock wlroots libwayland-client \
                waybar dunst wofi eww; do
        fetch_arch_pkg "$pkg"
    done

    # Hyprland (bundled in fessus ISO so users can switch)
    for pkg in hyprland hyprpaper hyprlock hypridle \
                xdg-desktop-portal-hyprland; do
        fetch_arch_pkg "$pkg"
    done

    # Wayland utilities
    for pkg in wl-clipboard cliphist grim slurp wf-recorder \
                xdg-desktop-portal xdg-desktop-portal-wlr xdg-utils; do
        fetch_arch_pkg "$pkg"
    done

    # GUI apps
    for pkg in firefox thunar mousepad mpv imv \
                zathura zathura-pdf-mupdf xarchiver \
                blueman qalculate-gtk; do
        fetch_arch_pkg "$pkg"
    done

    for pkg in inter-font papirus-icon-theme; do
        fetch_arch_pkg "$pkg"
    done
}

profile_desktop_section() {
    cat <<'EOF'
[desktop]
environment = "fessus"

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
