# Space Travel

En este repositorio se encuentra nuestra implementación de un renderer que muestra un sistema solar con varios planetas hecho en Rust. El proyecto permite moverse con una nave sobre el plano eclíptico, acercarse a los planetas, rodearlos y explorar el sistema en general.

## Integrantes del grupo

- Santiago Cordero
- Diego Gudiel
- Adaír Velásquez

## Video de demostración

**TODO**
El video demostrativo de la ejecución de nuestro proyecto se puede ver [en este enlace]()


## Capturas

**TODO**


## Controles

| Tecla | Función |
|---|---|
| `W` / `S` | Avanzar / retroceder |
| `A` / `D` | Desplazarse hacia la izquierda / derecha |
| `←` / `→` | Girar orientar la nave |


## Shaders implementados

**TODO** hacer descripción de cada shader

### Nave

Para la nave implementamos un shader de casco con paneles, uniones, desgaste, iluminación envolvente, reflejos suaves, rim lighting y franjas cian animadas.

### Neutron star

Implementamos una estrella de neutrones mediante un shader emisivo animado que combina FBM y ruido celular para producir turbulencia y variación sobre la superficie.


## Ejecución del Proyecto

El proyecto, como fue mencionado anteriormente, fue realizado con Rust, por lo que las únicas dependencias son rust y cargo. Para correr el proyecto, solo se debe de ejecutar esto desde la raíz del repositorio:

```bash
cargo run --release
```

Es recomendado ejecutar el código con --release para tener mejor performance por las optimizaciones que hace cargo con esa flag activa.

## Estructura del proyecto

El proyecto está dividido en la siguiente estructura:

```bash
.
├── assets
│   └── obj
│       ├── nave.obj
│       ├── planet1.obj
│       ├── planeta_morado.mtl
│       ├── planeta_morado.obj
│       ├── planeta_rick.obj
│       ├── planeta_verde.mtl
│       ├── planeta_verde.obj
│       └── sphere.obj
├── Cargo.lock
├── Cargo.toml
├── README.md
└── src
    ├── algorithms
    │   ├── barycentric.rs
    │   ├── mod.rs
    │   ├── scanline.rs
    │   ├── triangle.rs
    │   └── wireframe.rs
    ├── camera.rs
    ├── framebuffer.rs
    ├── line.rs
    ├── main.rs
    ├── model.rs
    ├── parallelism
    │   ├── mod.rs
    │   └── thread.rs
    ├── procedural.rs
    ├── shaders.rs
    └── worldobjects.rs

6 directories, 26 files
```

- `./assets/obj`: contiene los modelos utilizados para tanto los planetas como la nave (modelo que fue realizado por Santiago Cordero para el laboratorio #4)
- `./src/algorithms`: contiene la rasterización de triángulos, interpolación de fragmentos y algoritmos gráficos del renderer
- `./src/parallelism`: divide el framebuffer en bandas horizontales que se renderizan con threads
- `./src/main.rs`: construye la escena, actualiza las órbitas y transformaciones, y coordina el proceso de renderizado
- `./src/camera.rs`, `model.rs` y `framebuffer.rs`: implementan la cámara, la carga de modelos y los buffers de color, profundidad y emisión
- `./src/shaders.rs` y `procedural.rs`: contienen los shaders de fragmentos y las funciones de ruido utilizadas para generar sus efectos
- `./src/worldobjects.rs`: guarda la información de los planetas (tanto su mesh como sus propiedades en sí como objetos del mundo, tales como su velocidad de rotación, posición, etc...)
