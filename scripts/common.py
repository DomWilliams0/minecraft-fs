from __future__ import annotations

import re
import sys
from dataclasses import dataclass
from pathlib import Path
from signal import signal, SIGPIPE, SIG_DFL
from typing import ClassVar, Optional


class IoException(Exception):
    def __init__(self, mc, path, exc, op):
        self.path = path.relative_to(mc.mnt)
        self.exc = exc
        self.message = f"Failed to {op} '{self.path}': {exc}"

    def __str__(self):
        return self.message


class Minecraft:
    def __init__(self, mnt: Path):
        if not mnt.is_dir():
            raise RuntimeError("Mount dir is invalid")

        # mild validation
        if not (mnt / "player").is_dir():
            raise RuntimeError("Mount dir missing player dir, sure it's a mount point?")

        # make it work nice with pipes such as `head`
        signal(SIGPIPE, SIG_DFL)

        self.mnt = mnt

    @classmethod
    def from_args(cls) -> Minecraft:
        args = sys.argv
        try:
            mnt_dir = Path(args[1])
        except IndexError:
            print("expected mnt directory as first arg")
            exit(1)

        try:
            return Minecraft(mnt_dir)
        except Exception as e:
            print(f"error: {e}")
            exit(1)

    def player(self) -> Optional[EntityProxy]:
        try:
            entity = (self.mnt / "player" / "entity").resolve()
            world_name = (self.mnt / "player" / "world").readlink().name

            entity_id = int(entity.name)
            return EntityProxy(self, world_name, entity_id)
        except Exception:
            return None

    def entity(self, entity_id: int, world=None) -> Optional[EntityProxy]:
        if world is not None:
            world_name = world
        else:
            world_name = (self.mnt / "player" / "world").readlink().name

        return EntityProxy(self, world_name, entity_id)

    def iter_entities(self, living_filter=None, world=None):
        if world is not None:
            world_name = world
            world = self.mnt / "worlds" / world
        else:
            world = self.mnt / "player" / "world"
            world_name = world.readlink().name

        entities = world / "entities" / "by-id"
        use_living_filter = isinstance(living_filter, bool)

        for entity in entities.glob("*"):
            try:
                living = self._exists(entity / "living")
                if use_living_filter and living != living_filter:
                    continue

                entity_id = int(entity.name)
                yield EntityProxy(self, world_name, entity_id)
            except IoException:
                pass

    def block(self, world: str, pos: BlockPos) -> BlockProxy:
        return BlockProxy(self, world, pos)

    def _read(self, p: Path) -> str:
        try:
            return p.read_text()
        except OSError as e:
            raise IoException(self, p, e, "read")

    def _write(self, p: Path, text: str, truncate=False):
        try:
            with p.open("w" if truncate else "a") as f:
                f.write(text)
        except OSError as e:
            raise IoException(self, p, e, "write")

    def _exists(self, p: Path) -> bool:
        try:
            return p.exists()
        except OSError as e:
            raise IoException(self, p, e, "check existence of")


@dataclass
class Position:
    x: float
    y: float
    z: float

    SPLIT_PATTERN: ClassVar[re.Pattern] = re.compile(r"\s|,")

    @classmethod
    def from_string(cls, s: str) -> Optional[Position]:
        [x, y, z] = cls.SPLIT_PATTERN.split(s, 2)
        try:
            return Position(float(x), float(y), float(z))
        except ValueError:
            raise RuntimeError(f"invalid position '{s}'")

    def to_block_pos(self) -> BlockPos:
        return BlockPos(int(self.x), int(self.y), int(self.z))

    def __repr__(self):
        return f"{self.x},{self.y},{self.z}"


@dataclass
class BlockPos:
    x: int
    y: int
    z: int

    SPLIT_PATTERN: ClassVar[re.Pattern] = re.compile(r"\s|,")

    @classmethod
    def from_string(cls, s: str) -> Optional[Position]:
        [x, y, z] = cls.SPLIT_PATTERN.split(s, 2)
        try:
            return Position(int(x), int(y), int(z))
        except ValueError:
            raise RuntimeError(f"invalid blockpos '{s}'")

    def __repr__(self):
        return f"{self.x},{self.y},{self.z}"


class EntityProxy:
    def __init__(self, mc: Minecraft, world: str, id: int) -> EntityProxy:
        print(f"entity with id {id}")
        self._world = world
        self._id = id
        self._mc = mc
        self._path = mc.mnt / "worlds" / world / "entities" / "by-id" / str(self._id)

    @property
    def id(self) -> int:
        return self._id

    @property
    def entity_type(self) -> int:
        path = self._path / "type"
        return self._mc._read(path)

    @property
    def position(self) -> Position:
        print(f"get pos for {self.id}")
        path = self._path / "position"
        return Position.from_string(self._mc._read(path))

    @position.setter
    def position(self, value: Position):
        """
        Position is treated as if it's in the same world
        """
        print(f"set pos for {self.id}")
        path = self._path / "position"
        self._mc._write(path, repr(value))

    @property
    def health(self) -> float:
        path = self._path / "health"
        return float(self._mc._read(path))

    @position.setter
    def health(self, value: float):
        path = self._path / "health"
        self._mc._write(path, str(value))

    def teleport(self, target: Position):
        self.position = target

    def kill(self):
        self.health = 0


class BlockProxy:
    def __init__(self, mc: Minecraft, world: str, pos: BlockPos) -> BlockProxy:
        self._world = world
        self._pos = pos
        self._mc = mc
        self._path = mc.mnt / "worlds" / world / "blocks" / repr(self._pos)

    @property
    def pos(self):
        return self._pos

    @property
    def block_type(self) -> str:
        path = self._path / "type"
        return self._mc._read(path)

    @block_type.setter
    def block_type(self, value: str):
        path = self._path / "type"
        self._mc._write(path, value)
