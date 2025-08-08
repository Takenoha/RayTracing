# Raytracing Simulation Engine

This is a simple raytracing simulation engine written in Rust. It reads a scene configuration from a `simulation.toml` file, simulates the paths of rays through the scene, and outputs the results to CSV files. It can also optionally render the scene using the Bevy game engine.

## How to Run

1.  Make sure you have Rust and Cargo installed.
2.  Clone this repository.
3.  Run the simulation from the root of the project:
    ```bash
    cargo run --release
    ```
4.  The simulation will read the `simulation.toml` file in the root directory.
5.  The output will be saved as a series of `.csv` files in the `./dist/` directory.

## Output Format

The program will create a `dist` directory if it doesn't exist. For each ray defined in the scene, a corresponding CSV file will be generated (e.g., `path_0.csv`, `path_1.csv`, ...).

Each CSV file contains the 3D coordinates (`x, y, z`) of the points along the ray's path as it bounces through the scene.

```
x,y,z
-15.0,0.0,0.0
-1.388...,-1.388...,-1.388...
...
```

## Configuration (`simulation.toml`)

The entire simulation is controlled by the `simulation.toml` file. Here is a breakdown of its structure.

### `[simulation_settings]`

This section controls the general parameters of the simulation.

-   `infinity_distance` (float): The distance at which a ray is considered to have traveled to infinity.
-   `max_bounces` (integer): The maximum number of times a ray can bounce before the simulation for that ray is stopped.

**Example:**

```toml
[simulation_settings]
infinity_distance = 50.0
max_bounces = 10
```

### `[scene]`

This section defines all the elements within the scene: the objects and the light rays. The scene can contain any combination of the following:
-   `[[scene.objects]]`: Individual objects.
-   `[[scene.object_generators]]`: Generators for creating grids of objects.
-   `[[scene.rays]]`: Individual light rays.
-   `[[scene.ray_generators]]`: Generators for creating groups of rays.

---

### Object Definition (`[[scene.objects]]`)

You can define individual objects in the scene by adding `[[scene.objects]]` tables. Each object is composed of a shape, a material, and a transform.

-   **`shape`**: Defines the geometry of the object.
    -   `{ type = "Sphere", radius = 1.5 }`
    -   `{ type = "Plane", normal = [0.0, 1.0, 0.0] }` (normal vector)
-   **`material`**: Defines the physical properties of the object's surface.
    -   `{ type = "Glass", ior = 1.5 }` (ior = index of refraction)
    -   `{ type = "Mirror" }`
    -   `{ type = "Lambertian" }` (diffuse surface)
-   **`transform`**: Defines the position and orientation of the object.
    -   `position = [x, y, z]`
    -   `rotation_y_deg = angle` (rotation around the Y axis in degrees)

**Example:**

```toml
[[scene.objects]]
# A glass sphere
shape = { type = "Sphere", radius = 1.5 }
material = { type = "Glass", ior = 1.5 }
transform = { position = [0.0, 0.0, 0.0], rotation_y_deg = 0.0 }

[[scene.objects]]
# A floor plane
shape = { type = "Plane", normal = [0.0, 1.0, 0.0] }
material = { type = "Mirror" }
transform = { position = [0.0, -10.0, 0.0], rotation_y_deg = 0.0 }
```

### Object Generator (`[[scene.object_generators]]`)

To create a grid of identical objects, you can use an object generator.

-   `type = "ObjectGrid"`
-   `count_x`, `count_z`: Number of objects in the X and Z directions.
-   `position_start`: The `[x, y, z]` coordinate of the first object in the grid.
-   `step_x`, `step_z`: The `[x, y, z]` vector to step between objects in each direction.
-   `template`: An object definition (shape, material, transform) that will be used for every object in the grid. The `position` in the template's transform is used as an offset from the calculated grid position.

**Example:**

```toml
[[scene.object_generators]]
type = "ObjectGrid"
count_x = 5
count_z = 5
position_start = [-10.0, 0.0, 10.0]
step_x = [5.0, 0.0, 0.0]
step_z = [0.0, 0.0, -5.0]
template.shape = { type = "Sphere", radius = 1.5 }
template.material = { type = "Glass", ior = 1.5 }
template.transform = { position = [0.0, 0.0, 0.0], rotation_y_deg = 0.0 }
```

### Ray Definition (`[[scene.rays]]`)

You can define individual rays.

- `origin`: The `[x, y, z]` starting point of the ray.
- `direction`: The `[x, y, z]` vector of the ray's direction.
- `current_ior`: The index of refraction of the medium the ray starts in (e.g., 1.0 for air).

**Example:**
```toml
[[scene.rays]]
origin = [-15.0, 0.0, 0.0]
direction = [1.0, 0.0, 0.0]
current_ior = 1.0
```

### Ray Generator (`[[scene.ray_generators]]`)

Generators can create patterns of rays.

#### `type = "Projector"`

Simulates rays from a single point towards a rectangular target.

- `origin`: The `[x, y, z]` starting point for all rays.
- `target_corner`: The `[x, y, z]` coordinate of a corner of the target rectangle.
- `target_u`, `target_v`: The `[x, y, z]` vectors defining the sides of the target rectangle from the corner.
- `count_u`, `count_v`: The number of rays to cast along each side of the target rectangle.
- `current_ior`: The starting index of refraction.

**Example:**

```toml
[[scene.ray_generators]]
type = "Projector"
origin = [-15.0, 0.0, 0.0]
target_corner = [-5.0, -5.0, -5.0]
target_u = [0.0, 10.0, 0.0]
target_v = [0.0, 0.0, 10.0]
count_u = 5
count_v = 5
current_ior = 1.0
```

#### `type = "ParallelGrid"`

Creates a grid of parallel rays.

- `origin_corner`: The `[x, y, z]` coordinate of the corner of the starting grid.
- `vec_u`, `vec_v`: The `[x, y, z]` vectors defining the sides of the grid.
- `count_u`, `count_v`: The number of rays along each side of the grid.
- `direction`: The single `[x, y, z]` direction vector for all rays.
- `current_ior`: The starting index of refraction.

**Example:**

```toml
[[scene.ray_generators]]
type = "ParallelGrid"
origin_corner = [-10.0, -10.0, -10.0]
vec_u = [20.0, 0.0, 0.0]
vec_v = [0.0, 20.0, 0.0]
count_u = 10
count_v = 10
direction = [0.0, 0.0, 1.0]
current_ior = 1.0
```

---

## Full Example `simulation.toml`

Here is a complete example file that you can use as a starting point.

```toml
# General simulation settings
[simulation_settings]
infinity_distance = 50.0
max_bounces = 10

# === Define Ray Sources ===

# A projector light source
[[scene.ray_generators]]
type = "Projector"
origin = [-15.0, 0.0, 0.0]         # Point source of light
target_corner = [-5.0, -5.0, -5.0] # Top-left corner of the target area
target_u = [0.0, 10.0, 0.0]        # "Width" of the target area
target_v = [0.0, 0.0, 10.0]        # "Height" of the target area
count_u = 10                       # Number of rays horizontally
count_v = 10                       # Number of rays vertically
current_ior = 1.0                  # Starting medium is air

# === Define Scene Objects ===

# A grid of glass spheres
[[scene.object_generators]]
type = "ObjectGrid"
count_x = 5
count_z = 5
position_start = [-10.0, 0.0, 10.0] # Starting position of the grid
step_x = [5.0, 0.0, 0.0]            # Distance between columns
step_z = [0.0, 0.0, -5.0]           # Distance between rows

# Template for the objects in the grid
template.shape = { type = "Sphere", radius = 1.5 }
template.material = { type = "Glass", ior = 1.5 }
template.transform = { position = [0.0, 0.0, 0.0], rotation_y_deg = 0.0 }

# A reflective floor plane
[[scene.objects]]
shape = { type = "Plane", normal = [0.0, 1.0, 0.0] }
material = { type = "Mirror" }
transform = { position = [0.0, -10.0, 0.0], rotation_y_deg = 0.0 }
```
