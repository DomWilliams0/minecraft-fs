package ms.domwillia.mcfs.ipc

import MCFS.*
import com.google.flatbuffers.FlatBufferBuilder
import ms.domwillia.mcfs.MinecraftFsMod
import net.minecraft.client.MinecraftClient
import net.minecraft.client.network.ClientPlayerEntity
import net.minecraft.entity.Entity
import net.minecraft.entity.LivingEntity
import net.minecraft.util.math.Box
import net.minecraft.util.math.Vec3d
import java.nio.ByteBuffer

class NoGameException : Exception()
class MissingTargetEntityException : Exception()
class NotLivingException : Exception()
class UnknownEntityException(val id: Int) : Exception()


@ExperimentalUnsignedTypes
class Executor(private val responseBuilder: FlatBufferBuilder) {

    fun execute(request: GameRequest): ByteBuffer {
        responseBuilder.clear();


        val gameResp = when (request.bodyType) {
            GameRequestBody.Command -> {
                val respBody = try {
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

                GameResponse.createGameResponse(responseBuilder, GameResponseBody.Response, respBody)
            }
            GameRequestBody.StateRequest -> {
                val respBody = executeStateRequest(request.body(StateRequest()) as StateRequest)
                GameResponse.createGameResponse(responseBuilder, GameResponseBody.StateResponse, respBody)
            }

            else -> {
                throw NullPointerException()
            }
        }

        responseBuilder.finish(gameResp)
        return responseBuilder.dataBuffer()
    }

    private fun executeCommand(command: Command): Int {
        MinecraftFsMod.LOGGER.info("Executing command '${CommandType.name(command.cmd)}'")
        return when (command.cmd) {
            CommandType.PlayerName -> mkString(MinecraftClient.getInstance().session.username)
            CommandType.EntityType -> mkString(getTargetEntity(command).type.toString())
            CommandType.EntityHealth -> mkFloat(getTargetLivingEntity(command).health)
            CommandType.EntityPosition -> mkPosition(getTargetEntity(command).pos)
            else -> {
                MinecraftFsMod.LOGGER.warn("Unknown command '$command'")
                mkError(Error.UnknownCommand)
            }
        }
    }

    private fun executeStateRequest(req: StateRequest): Int {
        MinecraftFsMod.LOGGER.info("Executing state request")


        // null if not in game
        val player = MinecraftClient.getInstance().player

        val entityIds = if (player != null && req.entitiesById) {
            val bounds = -10_000.0;
            val box = Box(Vec3d(-bounds, -bounds, -bounds), Vec3d(bounds, bounds, bounds))
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

    private val thePlayer: ClientPlayerEntity
        get() = MinecraftClient.getInstance().player ?: throw NoGameException()


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

    private fun getTargetEntity(command: Command): Entity {
        val id = command.targetEntity ?: throw  MissingTargetEntityException();
        val world = MinecraftClient.getInstance().world ?: throw NoGameException()
        return world.getEntityById(id) ?: throw UnknownEntityException(id)
    }

    private fun getTargetLivingEntity(command: Command): LivingEntity {
        return getTargetEntity(command) as? LivingEntity? ?: throw NotLivingException()
    }
}