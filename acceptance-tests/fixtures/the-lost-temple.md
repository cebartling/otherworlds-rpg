---
title: "The Lost Temple"
description: "An adventure into the ruins of an ancient temple"
min_engine_version: 1
---

# Scene: entrance

The ancient temple entrance looms before you. Crumbling stone pillars
frame a darkened archway, and the faint smell of incense lingers in
the still air. A weathered guard stands watch nearby.

## Choices
- [Enter the temple](scene:inner_hall)
- [Search the perimeter](scene:perimeter)

## NPCs
- guard_captain

# Scene: inner_hall

The inner hall is dark and musty. Faded murals stretch across the
walls, depicting scenes of forgotten rituals. A narrow corridor
leads deeper into the temple, while the entrance glows faintly
behind you.

## Choices
- [Go back](scene:entrance)
- [Descend deeper](scene:sanctum)

# Scene: perimeter

You circle the exterior of the temple. Moss-covered stones and
tangled vines conceal what might once have been windows. Near the
rear wall, you notice a partially collapsed section that could
serve as an alternate way in.

## Choices
- [Return to the entrance](scene:entrance)
- [Squeeze through the gap](scene:sanctum)

# Scene: sanctum

The sanctum is bathed in an eerie blue glow emanating from a
crystal altar at its centre. The air hums with latent energy.
Whatever power this temple once held, it has not entirely faded.

## Choices
- [Return to the inner hall](scene:inner_hall)

## NPCs
- temple_keeper

# NPC: guard_captain

- name: Captain Theron
- disposition: neutral

# NPC: temple_keeper

- name: Elara the Keeper
- disposition: friendly
