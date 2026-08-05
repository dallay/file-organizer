# Sequence Diagrams

## (a) File organization with `[[categories]]` supplement + `[extensions]` override

Config:
```toml
[[categories]]
name = "Text"
extensions = ["foo"]            # supplement (no `replace`)
[extensions]
md = "Docs"                     # last-write override
```

```
load_config(path)
   |
   +-- toml::from_str -> Config
   |     categories: [CategoryRule { name: "Text", ext: ["foo"], replace: false }]
   |     extensions:  { "md" -> "Docs" }
   |
   +-- validate_config -> crate::categories::validate_categories(..) -> Ok
   |
   +-- apply_categories(config)              <-- one-shot composition
   |     built-ins  : { "txt"->"Text", "md"->"Text", "jpg"->"Image", ... }
   |     supplement : + { "foo"->"Text" }
   |     ext override: { "md"->"Docs" }      <-- wins last
   |     result map : { "txt"->"Text", "foo"->"Text", "md"->"Docs", ... }
   |
run(config)
   |
   +-- Lock::acquire            (skipped if --dry-run)
   +-- Logger::new(cfg.log_file)
   +-- canonical_log = fs::canonicalize(cfg.log_file)?
   |
   +-- for root in source_directories:
   |     +-- if !root.is_dir()      -> log "La carpeta no existe"; failures++; continue
   |     +-- canonical_root = fs::canonicalize(root)
   |     |
   |     +-- files = collect_files(canonical_root, config, skip_paths={canonical_log})
   |     |     walker = WalkDir::new(root).min_depth(1).filter_entry(should_visit)
   |     |     result: Vec<PathBuf> of FILE entries
   |     |
   |     +-- dirs = collect_top_level_directories(canonical_root, config, skip_paths)
   |     |     walker = max_depth(1).min_depth(1).filter_entry(should_visit)
   |     |     in-place pre-classify: skip generated / empty / symlink / hidden
   |     |     result: Vec<(PathBuf /*source*/, PathBuf /*dest*/)>
   |     |
   |     +-- for entry in (dirs then files):      <-- interleaved for shared logger
   |           |
   |           +-- if is_recent: maybe log; continue
   |           +-- category_for(entry) => str_name      <-- O(1) lookup in composed map
   |           +-- match (dest.exists(), on_conflict):
   |           |     (false, _)              => dest
   |           |     (true,  Skip)           => log "Omitido por conflicto"; continue
   |           |     (true,  Rename)         => unique_destination(dest)
   |           |     (true,  Overwrite && is_dir) => log "dest es carpeta"; continue
   |           |     (true,  Overwrite)      => dest
   |           +-- logger.line("Se movería"|"Movido", src, category)
   |           +-- if dry_run -> continue       (no create_dir_all, no rename)
   |           +-- fs::create_dir_all(dest.parent)
   |           +-- fs::rename(src, dest)?       (cross-vol => Err)
   |           +-- on Err: logger.line("ERROR: ..."); failures++
   |
   +-- drop(lock); if failures > 0 -> bail
```

The composed map is built once; every `category_for` lookup is an O(1)
hash get. `replace = true` was consumed during composition.

## (b) Top-level directory move with conflict policy + dry-run parity

```
run(config) [extract from (a)]:

collect_top_level_directories(root, config, skip_paths):
   |
   +-- for entry in WalkDir::new(root).max_depth(1).min_depth(1):
   |     |
   |     +-- depth 0 always admitted
   |     |     (test_entry.depth() == 0)
   |     |
   |     +-- PRE-CLASSIFY in place:
   |     |     if entry.file_type().is_symlink()                       -> drop   (test_16)
   |     |     if config.ignore_hidden && is_hidden(entry.path())       -> drop   (test_17)
   |     |     if entry.file_type().is_dir() &&
   |     |        is_generated_category(entry.path(), root, config)    -> drop   (test_13, test_14)
   |     |     if fs::read_dir(entry.path())?.count() == 0             -> drop   (test_15)
   |     |
   |     +-- canonical_src = fs::canonicalize(entry.path())
   |     +-- if skip_paths.contains(&canonical_src)                    -> drop   (log-collision)
   |     |
   |     +-- dest = root.join("Other").join(entry.file_name())
   |     +-- yield (canonical_src, dest)
   |
   +-- return Vec<(PathBuf, PathBuf)>

move_dir((src, dest_root), config, options, logger):
   |
   +-- requested_dest = dest_root.join(src.file_name())
   +-- match (requested_dest.exists(), options.conflict_policy):
   |     (false, _)              => requested_dest
   |     (true,  Skip)           => log "Omitido por conflicto"; return Ok
   |     (true,  Rename)         => final = unique_destination(requested_dest)
   |     (true,  Overwrite && is_dir) => log "dest es carpeta"; return Ok
   |     (true,  Overwrite)      => final = requested_dest
   |
   +-- logger.line("Se movería" if dry_run else "Movido", src, "Other")
   +-- if dry_run: return Ok              <-- PARITY with move_file (src/lib.rs:255-263)
   +-- fs::create_dir_all(dest_root)      <-- only on real run
   +-- if Overwrite && final.exists() && !is_dir: fs::remove_file(final)
   +-- fs::rename(src, final)             <-- atomic; cross-vol => Err context
```

### test_14 guarantee

`Other` is in `is_generated_category_set`. The pre-classify at the top of
`collect_top_level_directories` rejects a top-level `Other/` as a MOVING
SOURCE but accepts `Other/` as the only valid DESTINATION PARENT. A root
that already contains `Other/` with files AND a `Loose/` directory
recursively produces:
- `Other/` files untouched (generated-category guard in `should_visit`).
- `Loose/` → `Other/Loose/` (path built at dest time).
