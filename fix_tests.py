import sys
import re
import os

def fix_file(filepath):
    if not os.path.exists(filepath):
        print("File {} not found.".format(filepath))
        return

    with open(filepath, 'r') as f:
        content = f.read()

    # Inject SpatialMap after bounds insertion
    content = re.sub(
        r'(world\.insert_resource\(bounds(?:.clone\(\))?\);)',
        r'\1\n        let mut spatial_map = crate::world::spatial::SpatialMap::new(bounds.chunks_x, bounds.chunks_y);\n        world.insert_resource(spatial_map);',
        content
    )
    content = re.sub(
        r'(world1\.insert_resource\(bounds\.clone\(\)\);)',
        r'\1\n        let mut spatial_map1 = crate::world::spatial::SpatialMap::new(bounds.chunks_x, bounds.chunks_y);\n        world1.insert_resource(spatial_map1);',
        content
    )
    content = re.sub(
        r'(world2\.insert_resource\(bounds\);)',
        r'\1\n        let mut spatial_map2 = crate::world::spatial::SpatialMap::new(bounds.chunks_x, bounds.chunks_y);\n        world2.insert_resource(spatial_map2);',
        content
    )

    # Capture chunk spawns and update SpatialMap
    def replacer(match):
        stmt = match.group(0)
        prefix = match.group(1) # e.g. "world" or "world_b"
        if "ChunkCoord::new" in stmt:
            m = re.search(r'ChunkCoord::new\(\s*(\d+)\s*,\s*(\d+)\s*\)', stmt)
            if m:
                x, y = m.group(1), m.group(2)
                return "let e = {}.id();\n        {}.resource_mut::<crate::world::spatial::SpatialMap>().set(crate::world::coord::ChunkCoord::new({}, {}), e);".format(stmt, prefix, x, y)
        return stmt

    content = re.sub(r'(world\w*)\.spawn\(\([\s\S]*?\)\);', replacer, content)

    # Note: `update_agent_metabolism` might also be used in tests where `bounds` is inserted but we need to ensure the variable `bounds` exists.
    # The regex `world.insert_resource(bounds);` assumes the variable is called `bounds`. This is true for all tests in systems.rs.

    with open(filepath, 'w') as f:
        f.write(content)

for file in [
    "engine/src/agent/systems.rs",
    "engine/src/persistence/io.rs",
    "engine/src/testing/determinism.rs"
]:
    fix_file(file)
