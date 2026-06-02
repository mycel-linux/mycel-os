{ pkgs ? import <nixpkgs> {} }:

pkgs.buildEnv {
  name = "mycelos-live";
  ignoreCollisions = true;

  paths = with pkgs; [
    # ── Core system ──────────────────────────────────────
    linuxPackages_zen.kernel
    coreutils
    bash
    util-linux
    procps
    shadow
    sudo
    runit
    eudev

    # ── Network ──────────────────────────────────────────
    networkmanager
    iwd
    curl
    wget

    # ── FessusDE compositor stack ─────────────────────────
    sway
    swaybg
    swaylock
    eww
    waybar
    dunst
    wofi
    kitty

    # ── Wayland utilities ─────────────────────────────────
    wl-clipboard
    cliphist
    grim
    slurp
    wf-recorder
    xdg-utils
    xdg-desktop-portal-wlr

    # ── Audio ─────────────────────────────────────────────
    pipewire
    wireplumber

    # ── Seat management ───────────────────────────────────
    seatd

    # ── Live apps ─────────────────────────────────────────
    firefox
    thunar
    mousepad
    mpv
    imv
    zathura
    (zathura.override { useMupdf = true; })
    btop
    fastfetch
    xarchiver
    blueman
    qalculate-gtk

    # ── Fonts ─────────────────────────────────────────────
    inter
    (nerdfonts.override { fonts = [ "JetBrainsMono" ]; })

    # ── Icons and cursors ─────────────────────────────────
    papirus-icon-theme
    bibata-cursors

    # ── Installer ─────────────────────────────────────────
    calamares

    # ── Nix itself ────────────────────────────────────────
    nix
    git
  ];
}
