# Contributing a community overlay

A MycelOS overlay is a GitHub repository that provides extra packages installable via `mycel.toml`.

## Creating an overlay

Your repo needs this structure:

```
your-overlay/
  overlay.toml        # metadata
  packages/
    my-app.toml       # one file per package
```

**overlay.toml:**
```toml
name        = "your-overlay"
description = "what this overlay provides"
maintainer  = "your GitHub username"
```

**packages/my-app.toml:**
```toml
[package]
name        = "my-app"
version     = "1.0.0"
description = "does a thing"

[source]
type   = "github-release"
repo   = "username/my-app"
tag    = "v1.0.0"
binary = "my-app-linux-x86_64"
```

## Submitting to the index

Once your overlay repo is ready:

1. Fork `mycel-linux/mycel-os`
2. Add an entry to `community/index.toml`:

```toml
[[overlays]]
name        = "your-overlay"
repo        = "github:username/your-overlay"
description = "short description"
maintainer  = "username"
verified    = false
```

3. Open a pull request

A maintainer will review that the overlay structure is valid and the packages work. Once merged, users can add your overlay to their `mycel.toml` immediately.

## Guidelines

- One package per file in `packages/`
- Packages must be open source or freeware
- No malware, no telemetry injectors, no cryptominers
- Keep your overlay maintained — broken overlays get removed
