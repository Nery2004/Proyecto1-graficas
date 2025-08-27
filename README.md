Proyecto 1 — Pacman Horror (Rust + raylib)
=========================================

Descripción
-----------
Juego experimental tipo Pacman con estética y mecánicas de terror, implementado en Rust usando los bindings de raylib.
El proyecto combina un render 3D estilo raycast (paredes proyectadas por slices), minimapa 2D, sprites en 3D (billboards) y sonido por raylib.

Características principales
--------------------------
- Menú principal con pantalla `home.png`.
- Pantalla de "warning" antes de iniciar la partida.
- Fantasmas con IA básica: patrullan y persiguen al jugador cuando lo ven.
- Efecto "screamer" al ser alcanzado por un fantasma (imagen + sonido), seguido de pantalla de Game Over.
- "You Win" cuando se recogen todas las pastillas con pantalla `win.png` y retorno automático al menú.
- Minimapa con indicadores de pastillas y fantasmas.
- Texturas de pared dinámicas: cuando un fantasma te mira las paredes parpadean a `wall_run.jpg` cada 0.5s.
- Sonidos: pasos, persecución, comer pastilla, screamer. Se detienen/gestionan según estado del juego.
- Ajustes visuales: control de estiramiento de paredes al acercarse, proyección con clamp.
- Controles: ratón (rotación mientras se mantiene botón derecho), teclado (WASD o flechas para mover, M para cambiar vista 2D/3D).

Estructura del proyecto
-----------------------
- `src/` - código fuente en Rust
  - `main.rs` - bucle principal, estados, audio, lógica de juego
  - `framebuffer.rs` - buffer de píxeles y funciones de render / swap
  - otros módulos: `player.rs`, `maze.rs`, `caster.rs`, `line.rs`...
- `assets/` - imágenes y audio (sprites, texturas y sonidos)
  - `assets/sprites/` - `home.png`, `warning.png`, `win.png`, `wall.jpg`, `wall_run.jpg`, sprites de fantasmas, pastillas, etc.
  - `assets/audio/` - `foot_steps.wav`, `perseguir.wav`, `eat.wav`, `screamer.wav`, etc.
- `maze.txt` - mapa del nivel (caracteres para muros, pastillas y spawns)

Cómo compilar y ejecutar
------------------------
Requisitos:
- Rust toolchain (rustc + cargo)
- Compilación contra la plataforma con raylib instalada o usando las bindings preconfiguradas.

Desde la raíz del proyecto:

```powershell
cargo build --release
# o en modo debug
cargo build
# ejecutar
cd target\debug
.\computer-graphics-v3.exe
```

Controles
---------
- Movimiento: W/A/S/D o flechas
- Rotar cámara: mantener botón derecho del ratón y mover horizontalmente
- M: cambiar vista 2D/3D
- Enter: en menús o reiniciar tras Game Over
- Esc: salir

Configuración y ajustes rápidos
------------------------------
- Cambiar velocidad de los fantasmas: editar los valores `speed` al crear `Ghost` en `src/main.rs`.
- Ajustar duración del "warning" o del "win": buscar `warning_timer` y `win_timer` en `src/main.rs`.
- Modificar color del flash (cuando un ghost te ve): `flash_color` en `src/main.rs`.

Notas
-----
- El juego asume la existencia de ciertos assets; si faltan, el código usa comprobaciones para evitar panic, pero la experiencia puede verse limitada.
- Para cambiar las texturas (por ejemplo `wall_run.jpg`), coloca el archivo en `assets/sprites/`.

Licencia
--------
Código y assets: revisa las fuentes de los assets y respeta las licencias correspondientes. Este repositorio no incluye una licencia por defecto.

Contacto
--------
Repositorio/local: trabaja en `c:\Users\Neryyy\Documents\Graficas\Proyecto1-graficas`.

---
Generado automáticamente: README básico describiendo el proyecto y cómo trabajar con él. Si quieres que añada screenshots, ejemplos de parámetros o un archivo de configuración, dime qué prefieres y lo incluyo.
