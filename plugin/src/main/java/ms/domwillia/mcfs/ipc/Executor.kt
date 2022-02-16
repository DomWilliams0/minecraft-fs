package ms.domwillia.mcfs.ipc

import MCFS.*
import com.google.flatbuffers.FlatBufferBuilder
import ms.domwillia.mcfs.MinecraftFsMod
import net.minecraft.block.Block
import net.minecraft.client.MinecraftClient
import net.minecraft.client.network.ClientPlayerEntity
import net.minecraft.entity.Entity
import net.minecraft.entity.LivingEntity
import net.minecraft.entity.damage.DamageSource
import net.minecraft.server.MinecraftServer
import net.minecraft.server.network.ServerPlayerEntity
import net.minecraft.server.world.ServerWorld
import net.minecraft.util.Identifier
import net.minecraft.util.math.BlockPos
import net.minecraft.util.math.Box
import net.minecraft.util.math.Vec3d
import net.minecraft.util.registry.Registry
import net.minecraft.util.registry.SimpleRegistry
import net.minecraft.world.World
import java.nio.ByteBuffer

class NoGameException : Exception()
class MissingTargetException : Exception()
class NotLivingException : Exception()
class BadBlockException(val block: String) : Exception()
class UnknownEntityException(val id: Int) : Exception()
class UnsupportedOperationException : Exception()
class InvalidTypeForWriteException : Exception()


@ExperimentalUnsignedTypes
class Executor(private val responseBuilder: FlatBufferBuilder) {

    fun execute(request: GameRequest): ByteBuffer {
        responseBuilder.clear()

        val gameResp = when (request.bodyType) {
            GameRequestBody.Command -> {
                val maybeRespBody = try {
                    executeCommand(request.body(Command()) as Command)
                } catch (_: NoGameException) {
                    mkError(Error.NoGame)
                } catch (_: MissingTargetException) {
                    MinecraftFsMod.LOGGER.error("Missing target info")
                    mkError(Error.MalformedRequest)
                } catch (e: UnknownEntityException) {
                    MinecraftFsMod.LOGGER.error("No such entity ${e.id}")
                    mkError(Error.NoSuchEntity)
                } catch (e: BadBlockException) {
                    MinecraftFsMod.LOGGER.error("Bad block: ${e.block}")
                    mkError(Error.NoSuchBlock)
                } catch (e: Exception) {
                    MinecraftFsMod.LOGGER.catching(e)
                    mkError(Error.Unknown)
                }

                val respBody = when (maybeRespBody) {
                    is Int -> maybeRespBody
                    null -> {
                        Response.startResponse(responseBuilder)
                        Response.endResponse(responseBuilder)
                    }
                    else -> {
                        MinecraftFsMod.LOGGER.error("Bad response returned from executor")
                        mkError(Error.Unknown)
                    }
                }

                GameResponse.createGameResponse(responseBuilder, GameResponseBody.Response, respBody)
            }
            GameRequestBody.StateRequest -> {
                val respBody = executeStateRequest(request.body(StateRequest()) as StateRequest)
                GameResponse.createGameResponse(responseBuilder, GameResponseBody.StateResponse, respBody)
            }

            else -> {
                MinecraftFsMod.LOGGER.error("Invalid request type");
                throw NullPointerException()
            }
        }

        responseBuilder.finish(gameResp)
        return responseBuilder.dataBuffer()
    }

    @Suppress("RedundantNullableReturnType")
    private fun executeCommand(command: Command): Any? {
        MinecraftFsMod.LOGGER.info("Executing command '${CommandType.name(command.cmd)}'")
        return when (command.cmd) {
            CommandType.PlayerName -> {
                command.ro()
                mkString(MinecraftClient.getInstance().session.username)
            }
            CommandType.EntityType -> {
                command.ro()
                mkString(getTargetEntity(command).type.toString())
            }
            CommandType.EntityHealth -> {
                val value = command.rwFloat()
                val entity = getTargetLivingEntity(command)
                if (value == null) {
                    mkFloat(entity.health)
                } else {
                    if (value < entity.health) {
                        entity.damage(DamageSource.OUT_OF_WORLD, entity.health - value)
                    } else {
                        entity.health = value
                    }
                }
            }
            CommandType.EntityPosition -> {
                val value = command.rwPos()
                val entity = getTargetEntity(command)
                if (value == null) {
                    mkPosition(entity.pos)
                } else {
                    entity.teleport(value.x, value.y, value.z)
                }
            }
            CommandType.WorldTime -> {
                val value = command.rwInt()
                val world = getTargetWorld(command)
                if (value == null) {
                    mkInt(world.timeOfDay.toInt())
                } else {
                    world.timeOfDay = value.toLong()
                }
            }

            CommandType.BlockType -> {
                val value = command.rwString();
                val pos = getTargetBlockPos(command)
                val world = getTargetWorld(command)
                if (value == null) {
                    val state = world.getBlockState(pos)
                    val id = Registry.BLOCK.getId(state.block);
                    mkString(id.toString())
                } else {
                    val toParse = value.lowercase()
                    try {
                        val id = if (toParse.contains(':')) {
                            Identifier(value)
                        } else {
                            Identifier("minecraft", toParse)
                        }

                        val block = (Registry.BLOCK as SimpleRegistry<Block>).get(id)!!
                        val state = block.defaultState

                        world.setBlockState(pos, state)

                    } catch (e: Exception) {
                        MinecraftFsMod.LOGGER.catching(e);
                        throw BadBlockException(toParse)
                    }
                }
            }

            CommandType.ControlSay -> {
                val value = command.woString()
                val player = MinecraftClient.getInstance().player ?: throw NoGameException()
                player.sendChatMessage(value)
            }

            CommandType.ControlJump -> {
                theClientPlayer.jump()
            }

            CommandType.ControlMove -> {
                val vec = command.woVec()
                theClientPlayer.travel(vec)
            }

            else -> {
                MinecraftFsMod.LOGGER.warn("Unknown command '$command'")
                mkError(Error.UnknownCommand)
            }
        }
    }

    private fun executeStateRequest(req: StateRequest): Int {
        MinecraftFsMod.LOGGER.info("Executing state request")

        // null if not in game
        val server = theServerOpt
        val player = server?.thePlayerOpt
        val world = req.targetWorld?.let(this::resolveWorld)

        val entities = if (world != null && req.entitiesById) {
            val bounds = -100_000.0
            val box = Box(Vec3d(-bounds, -bounds, -bounds), Vec3d(bounds, bounds, bounds))
            val entities = world.getOtherEntities(null, box)

            StateResponse.startEntitiesVector(responseBuilder, entities.size)
            for (e in entities) {
                EntityDetails.createEntityDetails(responseBuilder, e.id, e.isLiving, e.isAlive)
            }
            responseBuilder.endVector()
        } else {
            null
        }

        val block = if (world != null && req.targetBlock != null) {
            val tgt = req.targetBlock!!
            val state = world.getBlockState(BlockPos(tgt.x, tgt.y, tgt.z))

            BlockDetails.createBlockDetails(responseBuilder, hasColor = state.material != null)
        } else {
            null
        }

        StateResponse.startStateResponse(responseBuilder)
        if (player != null) {
            StateResponse.addPlayerEntityId(responseBuilder, player.id)
            StateResponse.addPlayerWorld(
                responseBuilder, when (player.world.registryKey) {
                    World.OVERWORLD -> Dimension.Overworld
                    World.NETHER -> Dimension.Nether
                    World.END -> Dimension.End
                    else -> throw IllegalArgumentException("unknown dimension")
                }
            )
        }

        if (entities != null) {
            StateResponse.addEntities(responseBuilder, entities)
        }

        if (block != null) {
            StateResponse.addBlock(responseBuilder, block)
        }

        return StateResponse.endStateResponse(responseBuilder)
    }

    private val theServerOpt: MinecraftServer?
        get() = MinecraftClient.getInstance().server

    private val theServer: MinecraftServer
        get() = theServerOpt ?: throw NoGameException()

    private val MinecraftServer.thePlayerOpt: ServerPlayerEntity?
        get() = playerManager?.getPlayer(MinecraftClient.getInstance().session.username)

    private val MinecraftServer.thePlayer: ServerPlayerEntity
        get() = thePlayerOpt ?: throw NoGameException()

    private val theClientPlayer: ClientPlayerEntity
        get() = MinecraftClient.getInstance().player ?: throw NoGameException()

    private fun mkError(err: Int): Int {
        responseBuilder.clear()

        Response.startResponse(responseBuilder)
        Response.addError(responseBuilder, err)
        return Response.endResponse(responseBuilder)
    }

    private fun mkFloat(float: Float): Int {
        Response.startResponse(responseBuilder)
        Response.addFloat(responseBuilder, float)
        return Response.endResponse(responseBuilder)
    }

    private fun mkInt(int: Int): Int {
        Response.startResponse(responseBuilder)
        Response.addInt(responseBuilder, int)
        return Response.endResponse(responseBuilder)
    }

    private fun mkString(string: String): Int {
        val s = responseBuilder.createString(string)
        Response.startResponse(responseBuilder)
        Response.addString(responseBuilder, s)
        return Response.endResponse(responseBuilder)
    }

    private fun mkPosition(pos: Vec3d): Int {
        val v = Vec3.createVec3(responseBuilder, pos.x, pos.y, pos.z)
        Response.startResponse(responseBuilder)
        Response.addVec(responseBuilder, v)
        return Response.endResponse(responseBuilder)
    }

    private fun getTargetEntity(command: Command): Entity {
        val id = command.targetEntity
        return if (id != null) {
            val world = getTargetWorld(command)
            world.getEntityById(id) ?: throw UnknownEntityException(id)
        } else if (command.targetPlayerEntity) {
            theServer.thePlayer
        } else {
            throw MissingTargetException()
        }
    }

    private fun getTargetLivingEntity(command: Command): LivingEntity {
        return getTargetEntity(command) as? LivingEntity? ?: throw NotLivingException()
    }

    private fun getTargetWorld(command: Command): ServerWorld {
        val dim = command.targetWorld ?: throw MissingTargetException()
        return resolveWorld(dim) ?: throw IllegalArgumentException("world not found")
    }

    private fun getTargetBlockPos(command: Command): BlockPos {
        val block = command.targetBlock ?: throw MissingTargetException()
        return BlockPos(block.x, block.y, block.z)
    }

    private fun resolveWorld(dim: UByte): ServerWorld? {
        val server = theServer
        return when (dim) {
            Dimension.Overworld -> server.getWorld(World.OVERWORLD)
            Dimension.Nether -> server.getWorld(World.NETHER)
            Dimension.End -> server.getWorld(World.END)
            else -> null
        }
    }


    private fun Command.ro() {
        if (this.write != null) throw UnsupportedOperationException()
    }

    private fun Command.rwFloat(): Float? {
        val writeBody = this.write
        return if (writeBody != null) {
            writeBody.float ?: throw InvalidTypeForWriteException()
        } else {
            null
        }
    }

    private fun Command.rwInt(): Int? {
        val writeBody = this.write
        return if (writeBody != null) {
            writeBody.int ?: throw InvalidTypeForWriteException()
        } else {
            null
        }
    }

    private fun Command.rwString(): String? {
        val writeBody = this.write
        return if (writeBody != null) {
            writeBody.string ?: throw InvalidTypeForWriteException()
        } else {
            null
        }
    }

    private fun Command.rwPos(): Vec3d? {
        val writeBody = this.write
        return if (writeBody != null) {
            val vec = writeBody.vec ?: throw InvalidTypeForWriteException()
            Vec3d(vec.x, vec.y, vec.z)
        } else {
            null
        }
    }

    private fun Command.woString(): String {
        val writeBody = this.write ?: throw UnsupportedOperationException()
        return writeBody.string ?: throw InvalidTypeForWriteException()
    }

    private fun Command.woVec(): Vec3d {
        val writeBody = this.write ?: throw UnsupportedOperationException()
        val vec = writeBody.vec ?: throw InvalidTypeForWriteException()
        return Vec3d(vec.x, vec.y, vec.z)
    }
}