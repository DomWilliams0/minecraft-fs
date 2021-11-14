package ms.domwillia.mcfs.ipc

import MCFS.*
import com.google.flatbuffers.FlatBufferBuilder
import ms.domwillia.mcfs.MinecraftFsMod
import net.minecraft.client.MinecraftClient
import net.minecraft.client.network.ClientPlayerEntity
import net.minecraft.util.math.Box
import net.minecraft.util.math.Vec3d
import java.nio.ByteBuffer

class NoGameException : Exception()

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
            CommandType.PlayerHealth -> mkFloat(thePlayer.health)
            CommandType.PlayerName -> mkString(MinecraftClient.getInstance().session.username)
            CommandType.PlayerPosition -> mkPosition(thePlayer.pos)
            else -> {
                MinecraftFsMod.LOGGER.warn("Unknown command '$command'")
                mkError(Error.UnknownCommand)
            }
        }
    }

    private fun executeStateRequest(req: StateRequest): Int {
        MinecraftFsMod.LOGGER.info("Executing state request")


        val player = MinecraftClient.getInstance().player
        val isInGame = player != null

        val entityIds = if (player != null && req.entitiesById) {
            val bounds = -10_000.0;
            val box = Box(Vec3d(-bounds, -bounds, -bounds), Vec3d(bounds, bounds, bounds))
            val entities = player.world.getOtherEntities(null, box);

            StateResponse.createEntityIdsVector(responseBuilder, entities.map { e -> e.id }.toIntArray())
        } else {
            null
        }

        StateResponse.startStateResponse(responseBuilder);
        StateResponse.addIsInGame(responseBuilder, isInGame);
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
}