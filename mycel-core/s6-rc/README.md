# s6-rc service tree

This tree is **generated**, not hand-written.

The source of truth is the declarative service definitions in
`../services/*.toml`. At ISO build time, `bootstrap.sh` runs
**mycel-compose** to weave those declarations into a complete s6-rc source
tree, then `s6-rc-compile` compiles it.

To regenerate locally for inspection:

    mycel-compose --services ../services --out /tmp/s6-rc-source
    s6-rc-compile /tmp/compiled /tmp/s6-rc-source

To add or change a service, edit a `.toml` in `../services/` — never write
s6-rc run scripts or `dependencies`/`contents` files by hand.
