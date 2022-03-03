import sys

from common import Minecraft, IoException, BlockPos


def demo_tpall():
    player = mc.player()
    if player is None:
        return

    target = player.position
    for e in mc.iter_entities():
        try:
            e.teleport(target)
            print(f"teleported {e.id}")
        except IoException as exc:
            print(f"failed to tp {e.id}: {exc}")
            raise exc


def demo_killall():
    player = mc.player()
    if player is None:
        return

    for e in mc.iter_entities(living_filter=True):
        if e.id != player.id:
            try:
                e.kill()
                print(f"killed {e.id} ({e.entity_type})")
            except Exception as exc:
                print(f"failed to kill {e.id}: {exc}")


def demo_blocks():
    player = mc.player()
    if player is None:
        return

    pos = player.position.to_block_pos()
    pos.y -= 1

    sz = 3
    for dx in range(-sz, sz):
        for dz in range(-sz, sz):
            block_pos = BlockPos(pos.x + dx, pos.y, pos.z + dz)
            block = mc.block(player.world, block_pos)
            block.block_type = "cobblestone"


if __name__ == '__main__':
    mc = Minecraft.from_args()

    def list_demos():
        return [name[len("demo_"):] for name in globals().keys() if name.startswith("demo_")]

    try:
        what = sys.argv[2]
    except IndexError:
        print("missing demo name, which can be one of {}".format(list_demos()))
        exit(1)

    try:
        name = "demo_{}".format(what.lower())
        globals()[name]()
    except KeyError:
        print("invalid demo name, must be one of {}".format(list_demos()))
        exit(1)
