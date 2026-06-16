import os

def fix_file(filepath):
    if not os.path.exists(filepath):
        return

    with open(filepath, 'r') as f:
        content = f.read()

    content = content.replace(
        "world.insert_resource(bounds.clone());\n        let mut spatial_map = crate::world::spatial::SpatialMap::new(bounds.chunks_x, bounds.chunks_y);",
        "let mut spatial_map = crate::world::spatial::SpatialMap::new(bounds.chunks_x, bounds.chunks_y);\n        world.insert_resource(bounds.clone());"
    )
    content = content.replace(
        "world.insert_resource(bounds);\n        let mut spatial_map = crate::world::spatial::SpatialMap::new(bounds.chunks_x, bounds.chunks_y);",
        "let mut spatial_map = crate::world::spatial::SpatialMap::new(bounds.chunks_x, bounds.chunks_y);\n        world.insert_resource(bounds);"
    )
    content = content.replace(
        "world1.insert_resource(bounds.clone());\n        let mut spatial_map1 = crate::world::spatial::SpatialMap::new(bounds.chunks_x, bounds.chunks_y);",
        "let mut spatial_map1 = crate::world::spatial::SpatialMap::new(bounds.chunks_x, bounds.chunks_y);\n        world1.insert_resource(bounds.clone());"
    )
    content = content.replace(
        "world2.insert_resource(bounds);\n        let mut spatial_map2 = crate::world::spatial::SpatialMap::new(bounds.chunks_x, bounds.chunks_y);",
        "let mut spatial_map2 = crate::world::spatial::SpatialMap::new(bounds.chunks_x, bounds.chunks_y);\n        world2.insert_resource(bounds);"
    )

    with open(filepath, 'w') as f:
        f.write(content)

for file in [
    "engine/src/agent/systems.rs",
    "engine/src/persistence/io.rs",
    "engine/src/testing/determinism.rs"
]:
    fix_file(file)
