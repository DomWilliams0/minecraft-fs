# minecraft-fs

[![Build](https://github.com/DomWilliams0/minecraft-fs/actions/workflows/build.yml/badge.svg)](https://github.com/DomWilliams0/minecraft-fs/actions/workflows/build.yml)
![Version](https://img.shields.io/badge/version-1.0-brightgreen)
![MC Version](https://img.shields.io/badge/minecraft-1.18.2-blue)
[![Lines](https://tokei.rs/b1/github/DomWilliams0/minecraft-fs)](https://github.com/XAMPPRocky/tokei)

A FUSE filesystem for querying and controlling Minecraft, as a universal mod platform (but mainly
for fun).

*Warning: don't get your hopes too high, this is still WIP!*

* * *

* [Examples GIFs](#examples)
* [Installation](#installation)
* [Usage](#usage)
* [Directory structure](#structure)


# What?

This plugin makes it possible to control your game through the filesystem, and therefore with common
Unix tools like `cat`, `find`, `grep` etc. This means you can easily write Minecraft mods with 
languages like bash and Python without needing to touch Java, gradle or Fabric.

# Why?

For fun, to learn about FUSE, but most importantly - why not?

# Examples <a id="examples"/>

## Controlling the player
<img src=".gifs/control.gif" />

## Teleporting
<img src=".gifs/teleport.gif" />

## Teleporting others to the player
<img src=".gifs/teleport-all.gif" />

## Setting health
<img src=".gifs/health.gif" />

## Setting blocks
<img src=".gifs/set-block.gif" />

# Scripting

In [./scripts](./scripts) you can find some python that encapsulates the filesystem structure and
makes for a nicer scripting experience. See [the demo script](./scripts/demo.py) for some examples.

```python
import common
mc = Minecraft.from_args()

player = mc.player()
print(f"{player.name} is at {player.position}")

player.kill()
```

# Installation <a id="installation"/>

* Download [latest release](https://github.com/DomWilliams0/minecraft-fs/releases), or build it yourself
    * Build FUSE filesystem with `cargo build --bin minecraft-fs --release`
    * Build Minecraft mod with `cd plugin; ./gradlew build`, which will build the jar file to
        `build/libs`
* Install Minecraft mod
    * Client version: **1.18.2**
    * Dependencies: [Fabric Loader](https://fabricmc.net/use/installer/),
[Fabric API](https://www.curseforge.com/minecraft/mc-mods/fabric-api) and
[Fabric Language Kotlin](https://www.curseforge.com/minecraft/mc-mods/fabric-language-kotlin)
    * Install MCFS via mod manager/putting mod jar in `mods/`


# Usage <a id="usage"/>

* Install as above
* Start Minecraft
* Mount the FUSE filesystem over an empty directory
    * `mkdir mnt; ./minecraft-fs ./mnt`
* Join a **single player** world - there's currently no support for multiplayer

Your mountpoint should contain something like the following:

```bash
$ cd mnt
$ ls
player  version  worlds

$ ls -l player
drwxr-xr-x   - dom 21 Feb 20:27 control
lrwxr-xr-x   0 dom 21 Feb 20:27 entity -> world/entities/by-id/135
.rwxr-xr-x 256 dom 21 Feb 20:27 health
.rwxr-xr-x 256 dom 21 Feb 20:27 name
.rwxr-xr-x 256 dom 21 Feb 20:27 position
lrwxr-xr-x   0 dom 21 Feb 20:27 world -> ../worlds/overworld
```

Congratulations, you can now manipulate the game through reading and writing to these special files.

## Directory structure <a id="structure"/>

```asm
; wo=write only, ro=read only, rw=read and write
├── command       ; wo, executes a command as the player
├── player
│   ├── control    ; all the files here are write-only
│   │   ├── jump   ; causes the player to jump on any input
│   │   ├── move   ; applies the given x,y,z force to the player
│   │   └── say    ; makes the player chat
│   ├── health     ; rw, the player's health
│   ├── name       ; ro, the player's name
│   ├── position   ; rw, the player's position
│   ├── gamemode   ; rw, the player's gamemode
│   ├── hunger     ; rw, the player's hunger
│   ├── exhaustion ; rw, the player's exhaustion
│   ├── saturation ; rw, the player's food saturation
│   ├── target     ; wo, a position to look at
│   ├── entity -> world/entities/by-id/135  ; symlink to player entity
│   └── world -> ../worlds/overworld  ; symlink to player world
└── worlds
    ├── overworld
    │   ├── blocks
    │   │   ├── 100,64,250
    │   │   │   ├── adjacent  ; dir of symlinks to adjacent blocks
    │   │   │   │   ├── above -> ../../100,65,250
    │   │   │   │   ├── below -> ../../100,63,250
    │   │   │   │   ├── east -> ../../101,64,250
    │   │   │   │   ├── north -> ../../100,64,249
    │   │   │   │   ├── south -> ../../100,64,251
    │   │   │   │   └── west -> ../../99,64,250
    │   │   │   ├── pos    ; ro, this block's position
    │   │   │   └── type   ; rw, the block's type
    │   │   ├── 100.2 64.555 250.1223  ; this works too
    │   │   │   └── ...
    │   │   └── README  ; ro, explains the dir structure
    │   ├── entities
    │   │   ├── by-id
    │   │   │   ├── 107  ; entity id
    │   │   │   │   ├── health     ; rw, the entity's health (if living)
    │   │   │   │   ├── living     ; inaccessible, exists to indicate living
    │   │   │   │   ├── position   ; rw, the entity's position
    │   │   │   │   ├── target     ; wo, a position to look at
    │   │   │   │   └── type       ; ro, the entity's type
    │   │   │   ├── 108
    │   │   │   │   ├── health
    │   │   │   │   ├── living
    │   │   │   │   ├── position
    │   │   │   │   ├── target
    │   │   │   │   └── type
    │   │   │   ...
    │   │   └── spawn ; rw, spawns an entity, read file for help
    │   └── time      ; rw, the world's time
    ├── nether
    │   ├── blocks
    │   │   └── ...
    │   ├── entities
    │   │   └── ...
    │   └── time
    └── end
        ├── blocks
        │   └── ...
        ├── entities
        │   └── ...
        └── time
```

# TODOs

* More endpoints
    * [X] player gamemode
    * [X] entity hunger
    * [ ] better player movement
    * [ ] entity looking direction (yaw,pitch,roll)
    * [X] entity target pos
    * [ ] symlink to entity vehicle
* Inventory management
    * [ ] individual slots
    * [ ] symlink to current slot, armour, other hand
    * [ ] give/spawn items
* More block control
*   * [ ] orientation
    * [ ] nbt tags
* [X] Entity spawning
* More entity filters than `by-id`
    * [ ] by-type
    * [ ] by-proximity-to a position and radius
* Server settings
    * [ ] game rules
    * [ ] pvp
    * [ ] difficulty
    * [ ] weather
* Event file for reacting to events
    * [ ] `tail`able file of events such as player chat
* Client specific things
    * [ ] pause/unpause game
    * [ ] load into world, stop server
* Multiplayer support
    * [ ] install as a server mod, control the server world
    * [ ] install as a client mod and join an unmodded server, at least control the player
