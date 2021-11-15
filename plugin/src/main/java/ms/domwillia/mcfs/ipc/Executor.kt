package ms.domwillia.mcfs.ipc

import MCFS.*
import MCFS.Common.Vec3
import com.google.flatbuffers.FlatBufferBuilder
import ms.domwillia.mcfs.MinecraftFsMod
import net.minecraft.client.MinecraftClient
import net.minecraft.entity.Entity
import net.minecraft.entity.LivingEntity
import net.minecraft.entity.damage.DamageSource
import net.minecraft.server.MinecraftServer
import net.minecraft.server.network.ServerPlayerEntity
import net.minecraft.util.math.Box
import net.minecraft.util.math.Vec3d
import java.nio.ByteBuffer

class NoGameException : Exception()
class MissingTargetEntityException : Exception()
class NotLivingException : Exception()
class UnknownEntityException(val id: Int) : Exception()
class UnsupportedOperationException : Exception()
class InvalidTypeForWriteException : Exception()


@ExperimentalUnsignedTypes
class Executor(private val responseBuilder: FlatBufferBuilder) {

    fun execute(request: GameRequest): ByteBuffer? {
        responseBuilder.clear();

        val gameResp = when (request.bodyType) {
            GameRequestBody.Command -> {
                val maybeRespBody = try {
                    executeCommand(request.body(Command()) as Command)
                } catch (_: NoGameException) {
                    mkError(Error.NoGame)
                } catch (_: MissingTargetEntityException) {
                    MinecraftFsMod.LOGGER.error("missing target entity")
                    mkError(Error.MalformedRequest)
                } catch (e: UnknownEntityException) {
                    MinecraftFsMod.LOGGER.error("no such entity ${e.id}")
                    mkError(Error.NoSuchEntity)
                } catch (e: Exception) {
                    MinecraftFsMod.LOGGER.catching(e)
                    mkError(Error.Unknown)
                }

                (maybeRespBody as? Int)?.let {
                    GameResponse.createGameResponse(responseBuilder, GameResponseBody.Response, it)
                }
            }
            GameRequestBody.StateRequest -> {
                val respBody = executeStateRequest(request.body(StateRequest()) as StateRequest)
                GameResponse.createGameResponse(responseBuilder, GameResponseBody.StateResponse, respBody)
            }

            else -> {
                throw NullPointerException()
            }
        }

        return gameResp?.let {
            responseBuilder.finish(it)
            responseBuilder.dataBuffer()
        }
    }

    private fun executeCommand(command: Command): Any {
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
                val entity = getTargetLivingEntity(command)
                if (value == null) {
                    mkPosition(entity.pos)
                } else {
                    entity.teleport(value.x, value.y, value.z)
                }
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
        val player = server?.thePlayer

        val entityIds = if (player != null && req.entitiesById) {
            val bounds = -10_000.0;
            val box = Box(Vec3d(-bounds, -bounds, -bounds), Vec3d(bounds, bounds, bounds))
            // TODO specify world name
            val entities = player.world.getOtherEntities(null, box);

            StateResponse.createEntityIdsVector(responseBuilder, entities.map { e -> e.id }.toIntArray())
        } else {
            null
        }

        StateResponse.startStateResponse(responseBuilder);
        if (player != null) {
            StateResponse.addPlayerEntityId(responseBuilder, player.id);
        }

        if (entityIds != null) {
            StateResponse.addEntityIds(responseBuilder, entityIds)
        }
        return StateResponse.endStateResponse(responseBuilder);
    }

    private val theServerOpt: MinecraftServer?
        get() = MinecraftClient.getInstance().server

    private val theServer: MinecraftServer
        get() = theServerOpt ?: throw NoGameException()

    private val MinecraftServer.thePlayer: ServerPlayerEntity
        get() = playerManager?.getPlayer(MinecraftClient.getInstance().session.username)!! // should be present


    private fun mkError(err: Int): Int {
        Response.startResponse(responseBuilder)
        Response.addError(responseBuilder, err)
        return Response.endResponse(responseBuilder)
    }

    private fun mkFloat(float: Float): Int {
        Response.startResponse(responseBuilder)
        Response.addFloat(responseBuilder, float)
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
        Response.addPos(responseBuilder, v)
        return Response.endResponse(responseBuilder)
    }

    // TODO specify world explicitly
    private fun getTargetEntity(command: Command): Entity {
        val id = command.targetEntity ?: throw MissingTargetEntityException();
        val world = theServer.thePlayer.world
        return world.getEntityById(id) ?: throw UnknownEntityException(id)
    }

    private fun getTargetLivingEntity(command: Command): LivingEntity {
        return getTargetEntity(command) as? LivingEntity? ?: throw NotLivingException()
    }

    private fun Command.ro() {
        if (this.write != null) throw UnsupportedOperationException()
    }

    private fun Command.rwFloat(): Float? {
        val writeBody = this.write;
        return if (writeBody != null) {
            writeBody.float ?: throw InvalidTypeForWriteException()
        } else {
            null
        }
    }

    private fun Command.rwPos(): Vec3d? {
        val writeBody = this.write;
        return if (writeBody != null) {
            val vec = writeBody.pos ?: throw InvalidTypeForWriteException()
            Vec3d(vec.x, vec.y, vec.z)
        } else {
            null
        }
    }
}