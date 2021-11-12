package ms.domwillia.mcfs.ipc

import MCFS.*
import com.google.flatbuffers.FlatBufferBuilder
import ms.domwillia.mcfs.MinecraftFsMod
import net.minecraft.client.MinecraftClient
import net.minecraft.client.network.ClientPlayerEntity
import net.minecraft.util.math.Vec3d
import net.minecraft.util.math.Vec3f
import java.nio.ByteBuffer

class NoGameException : Exception()

@ExperimentalUnsignedTypes
class CommandExecutor(private val responseBuilder: FlatBufferBuilder) {

    fun execute(command: Command): ByteBuffer {
        responseBuilder.clear();

        val resp = try {
            dewIt(command)
        } catch (_: NoGameException) {
            mkError(Error.NoGame)
        } catch (e: Exception) {
            MinecraftFsMod.LOGGER.catching(e)
            mkError(Error.Unknown)
        }

        responseBuilder.finish(resp)
        return responseBuilder.dataBuffer()
    }

    private fun dewIt(command: Command): Int {
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