from common import Minecraft, IoException

if __name__ == '__main__':
    mc = Minecraft.from_args()
    pos = None
    for e in mc.iter_entities(living_filter=True):
        if pos is None:
            pos = e.pos
        else:
            try:
                e.teleport(pos)
                print(f"teleported {e.id}")
            except IoException as exc:
                print(f"failed to tp {e.id}: exc")
