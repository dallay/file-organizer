# Sequence Diagrams

## (a) Release pipeline — release-please → jobs → publish order

```mermaid
sequenceDiagram
    autonumber
    participant GH as GitHub (push to main)
    participant RP as release-please
    participant BB as build-binaries (8 targets)
    participant UA as upload-assets
    participant NB as publish-npm-binaries (6 pkgs)
    participant NBB as publish-npm-base
    participant CR as publish-crates
    participant DK as publish-docker
    participant RS as release-summary

    GH->>RP: push to main (conventional commits)
    RP->>RP: bump version (manifest + rust type), create release PR, tag vX.Y.Z, GitHub Release
    Note over RP: outputs: new_release_created, release_version, release_git_tag
    RP-->>BB: version + tag (if new_release_created == 'true')
    RP-->>UA: tag_name
    RP-->>NB: version
    RP-->>CR: version
    RP-->>DK: version
    BB->>BB: 8× (cargo build | cross build) → organiza-{v}-{target}.{tar.gz|zip} + .sha256
    BB-->>UA: upload-artifact (8 archives)
    BB-->>NB: upload-artifact (per target)
    UA->>UA: softprops/action-gh-release → attach 8 archives to the Release
    NB->>NB: envsubst < package.json.tmpl → publish @dallay/organiza-{os}-{arch}@X (--provenance)
    NB-->>NBB: result == 'success' (gate)
    NBB->>NBB: npm install; tsc build; publish @dallay/organiza@X (--provenance)
    CR->>CR: cargo publish --locked (organiza crate, CARGO_REGISTRY_TOKEN)
    DK->>DK: buildx multi-arch (linux/amd64,linux/arm64) → yacosta738/organiza + ghcr.io/dallay/organiza
    RS->>RS: $GITHUB_STEP_SUMMARY (install commands per channel)
```

Key gating facts: every publisher `needs: [release-please, build-binaries]`; `publish-npm-base` additionally gates on the binaries job's `result == 'success'`; `release-summary` runs `if: always() && new_release_created == 'true'` and `needs` all jobs. `workflow_dispatch.dry_run` short-circuits because `new_release_created` is false.

## (b) npm wrapper runtime — user invokes `organiza`

```mermaid
sequenceDiagram
    autonumber
    participant U as User shell
    participant N as node (bin/organiza → lib/index.js)
    participant R as require.resolve
    participant P as @dallay/organiza-{os}-{arch}
    participant B as organiza binary

    U->>N: organiza run --dry-run ~/Downloads
    N->>N: key = process.platform + "-" + process.arch
    alt key not in PLATFORMS
        N-->>U: "Unsupported platform: <key>" + supported list; exit 1
    end
    N->>R: require.resolve("<pkg>/package.json")
    alt resolve fails
        N->>N: fallback: join(__dirname, "..", "node_modules", pkg, "bin", bin)
    end
    R-->>N: node_modules/<pkg>/package.json
    N->>N: binaryPath = join(pkgPath, "..", "bin", "organiza" + (.exe if win32/cygwin))
    alt !existsSync(binaryPath)
        N-->>U: "Could not find organiza binary … reinstall with npm install @dallay/organiza"; exit 1
    end
    N->>B: spawnSync(binaryPath, process.argv.slice(2), { stdio: "inherit", env: process.env })
    B-->>N: exit status
    N-->>U: process.exit(status ?? 1)
```

The wrapper is a pure proxy: identical args, inherited stdio, propagated exit code. The `cygwin` aliases in `PLATFORMS` map to the `windows-*` packages so Git-Bash/Cygwin hosts resolve the `.exe`.

## (c) npm install-time — platform package selection by os/cpu

```mermaid
sequenceDiagram
    autonumber
    participant U as User
    participant NPM as npm i -g @dallay/organiza@X
    participant REG as npm registry
    participant P as matching @dallay/organiza-{os}-{arch}@X
    participant O as non-matching platform pkgs (5)

    U->>NPM: npm install -g @dallay/organiza@X
    NPM->>REG: fetch @dallay/organiza@X metadata
    REG-->>NPM: optionalDependencies: 6 × @dallay/organiza-* = X (exact)
    NPM->>REG: resolve each optional dep; filter by package "os" + "cpu" fields vs host (process.platform/arch)
    NPM->>P: install ONLY @dallay/organiza-{host-os}-{host-arch}@X
    REG-->>P: tarball containing bin/organiza (preferUnplugged → no postinstall)
    Note over P: template set os:[<host os>], cpu:[<host arch>], bin.organiza
    NPM->>O: skip non-matching (optional → absent on unsupported hosts)
    NPM->>NPM: bin-link organiza → lib/index.js (base wrapper)
    NPM-->>U: organiza available on PATH
```

Why it works: npm filters optionalDependencies by the package's `os`/`cpu` fields before install, so exactly one platform package lands in `node_modules` per host. A Linux ARM user gets `@dallay/organiza-linux-arm64`; a macOS Intel user gets `darwin-x64`. The wrapper's `require.resolve` then finds that single installed package. If no package matches (e.g., future exotic arch), the wrapper's PLATFORMS map throws with the supported list — a deliberate fail-loud, since optional deps never error the install itself.
