# File Organizer en Rust

Motor multiplataforma para macOS, Linux y Windows. La lógica de clasificación vive en un binario Rust; los schedulers de cada sistema operativo solo lo ejecutan.

## Compilar

Requiere Rust estable:

```bash
cargo test
cargo build --release
```

El binario queda en `target/release/file-organizer` (`file-organizer.exe` en Windows).

## Configurar

```bash
mkdir -p "$HOME/.config/file-organizer"
cp config.example.toml "$HOME/.config/file-organizer/config.toml"
```

En Windows, copia el archivo a `%APPDATA%\\file-organizer\\config.toml`. Edita las carpetas y ejecuta:

```bash
file-organizer --config ~/.config/file-organizer/config.toml validate-config
```

## Ejecutar

```bash
file-organizer run --dry-run
file-organizer run
file-organizer run --verbose ~/Downloads
file-organizer run --config ./config.toml --log /dev/null
```

Las carpetas indicadas al final sustituyen a `source_directories`. El comportamiento predeterminado espera 60 segundos, ignora ocultos y renombra conflictos (`archivo (1).pdf`).

## Reglas integradas

Incluye imágenes, documentos, hojas de cálculo, presentaciones, vídeo, audio, comprimidos, instaladores y código. Se pueden ampliar o sobrescribir desde `[extensions]` en TOML.

## Automatización por plataforma

- **macOS:** usa Shortcuts o `launchd` para ejecutar `file-organizer run`.
- **Linux:** usa el `systemd` user timer incluido en `platform/linux/`.
- **Windows:** usa Task Scheduler con `schtasks`.

El lock se crea con `create_dir`, por lo que no depende de `flock` y funciona en los tres sistemas.

### macOS con launchd

```bash
mkdir -p "$HOME/.local/bin" "$HOME/Library/LaunchAgents"
cp target/release/file-organizer "$HOME/.local/bin/"
sed "s#TU_USUARIO#$(whoami)#" platform/macos/com.file-organizer.plist.example \
  > "$HOME/Library/LaunchAgents/com.file-organizer.plist"
launchctl bootstrap "gui/$(id -u)" "$HOME/Library/LaunchAgents/com.file-organizer.plist"
```

Para detenerlo: `launchctl bootout "gui/$(id -u)" "$HOME/Library/LaunchAgents/com.file-organizer.plist"`.

### Linux con systemd user

```bash
mkdir -p "$HOME/.local/bin" "$HOME/.config/systemd/user"
cp target/release/file-organizer "$HOME/.local/bin/"
cp platform/linux/file-organizer.service platform/linux/file-organizer.timer \
  "$HOME/.config/systemd/user/"
systemctl --user daemon-reload
systemctl --user enable --now file-organizer.timer
```

### Windows con Task Scheduler

Después de copiar `file-organizer.exe` a una ruta permanente y crear el TOML en `%APPDATA%\\file-organizer\\config.toml`:

```powershell
schtasks /Create /TN "File Organizer" /SC MINUTE /MO 5 `
  /TR "C:\\Users\\TU_USUARIO\\.local\\bin\\file-organizer.exe run" /F
```

## Alcance actual

El movimiento utiliza `rename`, que es atómico dentro del mismo volumen. Si el origen y destino están en volúmenes distintos, se informa del error en vez de borrar o copiar parcialmente el archivo.
